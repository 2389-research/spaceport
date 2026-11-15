#!/usr/bin/env python3
"""Validate and navigate the Spaceport specification, never the application runtime.

Commands are local-only. Generated views have one canonical input; task packets
carry input hashes and reject overflow rather than silently losing constraints.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
GENERATED = {
    'READING_COPY.md', 'docs/DECISIONS.md', 'docs/IMPLEMENTATION_TASKS.md',
    'docs/VALIDATION.md', 'evidence/validation.json',
}
READ_ORDER = [
    'START_HERE.md', 'AGENTS.md', 'SPEC.md', 'SYSTEM.md',
    'docs/01-herdr-contract.md', 'docs/02-agent-interface.md',
    'docs/03-fork-and-deletion.md', 'docs/04-delivery-plan.md',
    'docs/05-verification.md', 'docs/06-security-and-resources.md',
    'docs/07-reconciliation.md', 'docs/08-agent-journeys.md',
    'docs/DECISIONS.md', 'docs/IMPLEMENTATION_TASKS.md',
    'docs/SOURCES.md', 'docs/VALIDATION.md', 'BOOTSTRAP_PROMPT.md',
]
REQ_RE = re.compile(r'^### (SP-\d{3}) — (.+)$', re.MULTILINE)


def reject_constant(value: str) -> None:
    """JSON NaN/Infinity are not portable contract values."""
    raise ValueError(f'Non-finite JSON constant: {value}')


def unique_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    """Reject duplicate JSON keys rather than silently taking the last one."""
    out: dict[str, Any] = {}
    for key, value in pairs:
        if key in out:
            raise ValueError(f'Duplicate JSON key: {key}')
        out[key] = value
    return out


def load(path: Path) -> Any:
    return json.loads(path.read_text(encoding='utf-8'),
                      object_pairs_hook=unique_object,
                      parse_constant=reject_constant)


def text(root: Path, relative: str) -> str:
    """Task paths may not escape the specification root."""
    resolved = (root / relative).resolve()
    if not resolved.is_relative_to(root.resolve()):
        raise ValueError(f'Path outside specification: {relative}')
    return resolved.read_text(encoding='utf-8')


def validate_evidence(root: Path, item: dict[str, Any]) -> None:
    """Evidence names a file in the package; presence only, not sufficiency."""
    for relative in item.get('evidence', []):
        resolved = (root / relative).resolve()
        if not resolved.is_relative_to(root.resolve()) or not resolved.is_file():
            raise ValueError(f"{item['id']}: evidence is not a file in the specification: {relative}")


def index(items: list[dict[str, Any]], kind: str) -> dict[str, dict[str, Any]]:
    result: dict[str, dict[str, Any]] = {}
    for item in items:
        key = item.get('id')
        if not isinstance(key, str) or not key:
            raise ValueError(f'{kind}: missing stable ID')
        if key in result:
            raise ValueError(f'{kind}: duplicate ID {key}')
        result[key] = item
    return result


def requirements(root: Path) -> dict[str, str]:
    """Extract each normative requirement without duplicating its authored text."""
    body = text(root, 'SPEC.md')
    matches = list(REQ_RE.finditer(body))
    result: dict[str, str] = {}
    for i, match in enumerate(matches):
        key = match.group(1)
        if key in result:
            raise ValueError(f'Duplicate requirement {key}')
        end = matches[i + 1].start() if i + 1 < len(matches) else len(body)
        result[key] = body[match.start():end].strip()
    if not result:
        raise ValueError('No normative requirements found')
    return result


def registries(root: Path) -> dict[str, Any]:
    return {
        'decisions': index(load(root / 'plan/decisions.json')['decisions'], 'decision'),
        'tasks': index(load(root / 'plan/tasks.json')['tasks'], 'task'),
        'acceptance': index(load(root / 'plan/acceptance.json')['scenarios'], 'acceptance'),
        'gaps': index(load(root / 'plan/gaps.json')['gaps'], 'gap'),
        'sources': index(load(root / 'plan/sources.json')['sources'], 'source'),
        'capabilities': index(load(root / 'plan/capabilities.json')['capabilities'], 'capability'),
        'removals': index(load(root / 'plan/removals.json')['items'], 'removal'),
        'requirements': requirements(root),
    }


def validate_graph(tasks: dict[str, dict[str, Any]]) -> None:
    """Check both missing prerequisites and cycles; do not execute any task."""
    visiting: set[str] = set()
    visited: set[str] = set()

    def visit(key: str) -> None:
        if key not in tasks:
            raise ValueError(f'Unknown prerequisite {key}')
        if key in visiting:
            raise ValueError(f'Task dependency cycle at {key}')
        if key in visited:
            return
        visiting.add(key)
        for dependency in tasks[key]['depends_on']:
            visit(dependency)
        visiting.remove(key)
        visited.add(key)

    for task_id in tasks:
        visit(task_id)


def validate_task_completion(task: dict[str, Any], gaps: dict[str, Any]) -> None:
    """Investigation can finish with findings; parity tasks must close their gates.

    Evidence references are required but are not independently authenticated here.
    A passed integrity check never establishes that cited tests actually ran.
    """
    if not set(task['must_close_gaps']).issubset(task['gaps']):
        raise ValueError(f"Completion gate missing from task context: {task['id']}")
    if task['status'] != 'done':
        return
    if not task['evidence']:
        raise ValueError(f"Done task has no evidence: {task['id']}")
    blocked = [g for g in task['must_close_gaps'] if gaps[g]['status'] != 'resolved']
    if blocked:
        raise ValueError(f"Done task has unresolved completion gates: {task['id']} {blocked}")


def package_files(root: Path) -> list[Path]:
    """Prune local environments/build output; never follow links outside the pack."""
    ignored = {'.git', '.venv', 'venv', '__pycache__', '.pytest_cache',
               '.mypy_cache', '.ruff_cache', 'target', 'node_modules', '.DS_Store'}
    files: list[Path] = []
    for directory, children, names in os.walk(root, followlinks=False):
        children[:] = sorted(d for d in children if d not in ignored)
        for child in children:
            if (Path(directory) / child).is_symlink():
                raise ValueError(f'Symlink in authored package: {child}')
        for name in sorted(names):
            path = Path(directory) / name
            if name in ignored or path.suffix == '.pyc':
                continue
            if path.is_symlink():
                raise ValueError(f'Symlink in authored package: {path.relative_to(root)}')
            files.append(path)
    return sorted(files)


def input_hashes(root: Path) -> dict[str, str]:
    """Hash all authored pack inputs, not environments or a claimed perfect impact graph."""
    result: dict[str, str] = {}
    for path in package_files(root):
        relative = path.relative_to(root).as_posix()
        if relative in GENERATED or relative.startswith('evidence/'):
            continue
        result[relative] = hashlib.sha256(path.read_bytes()).hexdigest()
    return result


def render_views(root: Path) -> dict[str, str]:
    r = registries(root)
    lines = ['# User decision ledger', '',
             '**Generated from `plan/decisions.json`; edit the JSON, not this view.**', '',
             '| ID | Decision | Status |', '|---|---|---|']
    for value in r['decisions'].values():
        lines.append(f"| {value['id']} | **{value['title']}** — {value['decision']} | {value['status']} |")
    tasks = ['# Implementation task graph', '',
             '**Generated from `plan/tasks.json`; this is not a second progress tracker.**', '']
    for value in r['tasks'].values():
        tasks += [f"## {value['id']} — {value['title']}", '',
                  f"Status: **{value['status']}** · Prerequisites: {', '.join(value['depends_on']) or 'none'}", '',
                  value['deliverable'], '',
                  f"Requirements: {', '.join(value['requirements'])}", '',
                  f"Acceptance: {', '.join(value['acceptance'])}", '',
                  f"Gates to investigate: {', '.join(value['gaps']) or 'none additional'}", '',
                  f"Gates this task must close: {', '.join(value['must_close_gaps']) or 'none; record findings without claiming full parity'}", '']
    return {'docs/DECISIONS.md': '\n'.join(lines) + '\n',
            'docs/IMPLEMENTATION_TASKS.md': '\n'.join(tasks).rstrip() + '\n'}


def rewrite_links(body: str, origin: str) -> str:
    """In the combined root-level copy, make relative document links root-relative."""
    def replace(match: re.Match[str]) -> str:
        target = match.group(2)
        if re.match(r'^[a-zA-Z]+:', target) or target.startswith('#'):
            return match.group(0)
        p, sep, anchor = target.partition('#')
        full = Path(origin).parent / p
        return f'[{match.group(1)}]({full.as_posix()}{sep}{anchor})'
    return re.sub(r'\[([^\]]+)\]\(([^)]+)\)', replace, body)


def render(root: Path) -> None:
    for relative, body in render_views(root).items():
        (root / relative).write_text(body, encoding='utf-8')
    parts = ['# Spaceport — complete specification reading copy', '',
             '**Generated. Edit the component documents and registries, then run `python tools/spec.py render`.**', '',
             'This is a specification and handoff, not an application implementation or runtime test report.', '']
    for relative in READ_ORDER:
        if (root / relative).exists():
            parts += ['---', '', f'<!-- Source: {relative} -->', '',
                      rewrite_links(text(root, relative), relative), '']
    (root / 'READING_COPY.md').write_text('\n'.join(parts).rstrip()+'\n', encoding='utf-8')


def validate(root: Path, require_schema: bool = True) -> dict[str, Any]:
    r = registries(root)
    # Parse all JSON strictly, including examples, not only indexed registries.
    json_files = [p for p in package_files(root) if p.suffix == '.json' and 'evidence' not in p.relative_to(root).parts]
    for path in json_files:
        load(path)
    validate_graph(r['tasks'])

    def refs(values: list[str], pool: dict[str, Any], where: str) -> None:
        missing = set(values) - set(pool)
        if missing:
            raise ValueError(f'{where}: unknown references {sorted(missing)}')

    assigned: set[str] = set()
    required: set[str] = set()
    for task in r['tasks'].values():
        for field, pool in [('requirements','requirements'), ('acceptance','acceptance'),
                            ('sources','sources'), ('gaps','gaps'), ('must_close_gaps','gaps')]:
            refs(task[field], r[pool], task['id'])
        if task['status'] not in {'not_started','in_progress','blocked','done'}:
            raise ValueError(f"Invalid task status: {task['id']}")
        validate_task_completion(task, r['gaps'])
        validate_evidence(root, task)
        for relative in task['read']:
            text(root, relative)
        assigned.update(task['acceptance'])
        required.update(task['requirements'])
    if assigned != set(r['acceptance']):
        raise ValueError(f'Unassigned acceptance: {sorted(set(r["acceptance"])-assigned)}')
    if required != set(r['requirements']):
        raise ValueError('Some normative requirements have no task')
    covered: set[str] = set()
    for scenario in r['acceptance'].values():
        refs(scenario['requirements'], r['requirements'], scenario['id'])
        if scenario['status'] not in {'not_run','passed','failed','blocked'}:
            raise ValueError(f"Invalid acceptance status: {scenario['id']}")
        if scenario['status'] in {'passed','failed'} and not scenario['evidence']:
            raise ValueError(f"Executed scenario lacks evidence: {scenario['id']}")
        validate_evidence(root, scenario)
        covered.update(scenario['requirements'])
    if covered != set(r['requirements']):
        raise ValueError(f'Uncovered requirements: {sorted(set(r["requirements"])-covered)}')
    for collection in ['gaps','capabilities','removals']:
        for item in r[collection].values():
            refs(item.get('sources',[]), r['sources'], item['id'])
            refs(item.get('gaps',[]), r['gaps'], item['id'])
            refs(item.get('decisions',[]), r['decisions'], item['id'])
    for source in r['sources'].values():
        if not re.fullmatch(r'[0-9a-f]{40}', source['commit']):
            raise ValueError(f"Unpinned source: {source['id']}")
    # Check local file links; external links and anchor content are not fetched.
    links_checked = 0
    for path in (p for p in package_files(root) if p.suffix == '.md'):
        for target in re.findall(r'\[[^\]]+\]\(([^)]+)\)',path.read_text(encoding='utf-8')):
            if re.match(r'^[a-zA-Z]+:',target) or target.startswith('#'):
                continue
            relative = target.split('#',1)[0]
            resolved = (path.parent / relative).resolve()
            if not resolved.is_relative_to(root.resolve()) or not resolved.exists():
                raise ValueError(f'Broken local link in {path.relative_to(root)}: {target}')
            links_checked += 1
    for relative, expected in render_views(root).items():
        if not (root / relative).exists() or text(root, relative) != expected:
            raise ValueError(f'Stale generated view: {relative}; run render')
    schema_status = 'not_run'
    schema_examples = 0
    if require_schema:
        try:
            from jsonschema import Draft202012Validator, SchemaError, ValidationError
        except ImportError as exc:
            raise ValueError('Install tools/requirements.txt to perform JSON Schema validation') from exc
        try:
            schema = load(root / 'contracts/control.schema.json')
            Draft202012Validator.check_schema(schema)
            validator = Draft202012Validator(schema)
            for example in sorted((root / 'examples').glob('*.json')):
                validator.validate(load(example))
                schema_examples += 1
        except (SchemaError, ValidationError) as exc:
            raise ValueError(f'JSON Schema validation failed: {exc.message}') from exc
        schema_status = 'passed'
    return {'scope':'Specification-package integrity only; no application conformance',
            'result':'passed', 'counts':{k:len(v) for k,v in r.items()},
            'strict_json_files':len(json_files),'local_file_links_checked':links_checked,
            'schema_validation':schema_status,'schema_examples':schema_examples,
            'application_scenarios_executed':sum(x['status'] in {'passed','failed'} for x in r['acceptance'].values()),
            'input_sha256':input_hashes(root)}


def task_packet(root: Path, task_id: str, max_bytes: int = 65536) -> str:
    r = registries(root)
    if task_id not in r['tasks']:
        raise ValueError(f'Unknown task {task_id}')
    task = r['tasks'][task_id]
    payload = {'task':task,
               'sources':[r['sources'][s] for s in task['sources']],
               'gaps':[r['gaps'][g] for g in task['gaps']],
               'acceptance':[r['acceptance'][a] for a in task['acceptance']],
               'input_sha256':input_hashes(root)}
    sections = [f'# Spaceport task context: {task_id}',
                'Specification requirements and recorded statuses follow. Source facts and spec checks do not prove application behavior.',
                '## Current task and traceability', '```json',json.dumps(payload,indent=2,ensure_ascii=False),'```',
                '## Always-on agent instructions',text(root,'AGENTS.md'),
                '## System boundaries',text(root,'SYSTEM.md'),
                '## Applicable normative requirements']
    sections += [r['requirements'][i] for i in task['requirements']]
    for relative in task['read']:
        sections += [f'## Read: {relative}',text(root,relative)]
    out = '\n\n'.join(sections)+'\n'
    size = len(out.encode('utf-8'))
    if size > max_bytes:
        raise ValueError(f'Packet is {size} bytes, exceeding explicit limit {max_bytes}; choose a reviewed larger budget or narrow task reads. Nothing was truncated.')
    return out


def ready(root: Path) -> list[dict[str, Any]]:
    tasks = registries(root)['tasks']
    return [t for t in tasks.values() if t['status']=='not_started'
            and all(tasks[d]['status']=='done' for d in t['depends_on'])]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--root',type=Path,default=ROOT)
    sub = parser.add_subparsers(dest='command',required=True)
    check = sub.add_parser('check',help='Validate specification artifacts, not the product')
    check.add_argument('--write-report',action='store_true')
    check.add_argument('--skip-schema',action='store_true',help='Explicitly report schema validation not run')
    sub.add_parser('render',help='Regenerate decision/task views and combined reading copy')
    sub.add_parser('ready',help='List dependency-ready, not-started tasks')
    packet = sub.add_parser('context',help='Emit one bounded, hash-bearing task packet')
    packet.add_argument('task_id')
    packet.add_argument('--max-bytes',type=int,default=65536)
    args = parser.parse_args()
    try:
        root = args.root.resolve()
        if args.command=='render':
            render(root)
            print('Rendered generated specification views.')
        elif args.command=='ready':
            print(json.dumps([{'id':t['id'],'title':t['title'],'gaps':t['gaps']} for t in ready(root)],indent=2))
        elif args.command=='context':
            if args.max_bytes <= 0:
                raise ValueError('--max-bytes must be positive')
            print(task_packet(root,args.task_id,args.max_bytes),end='')
        else:
            result = validate(root,not args.skip_schema)
            result['checked_at_utc']=datetime.now(timezone.utc).isoformat()
            if args.write_report:
                (root/'evidence/validation.json').write_text(json.dumps(result,indent=2)+'\n',encoding='utf-8')
            # Hashes remain in the evidence file; keep ordinary console output small.
            print(json.dumps({k:v for k,v in result.items() if k!='input_sha256'},indent=2))
        return 0
    except (ValueError, OSError, KeyError) as exc:
        print(f'Specification error: {exc}',file=sys.stderr)
        return 1


if __name__ == '__main__':
    raise SystemExit(main())
