# Specification tools

These local tools validate and navigate **this specification package**. They do not clone a repository, execute product code, contact Herdr, authenticate to a service, or verify desktop behavior.

Use Python 3.10 or newer in an isolated environment:

```sh
python -m venv .venv
# Activate the environment using the command for your shell.
python -m pip install -r tools/requirements.txt
python tools/spec.py render
python tools/spec.py check --write-report
python tools/test_spec.py
python tools/spec.py ready
python tools/spec.py context T00
```

The context command emits a task-specific packet with the applicable requirements, source revisions, open gaps, planned acceptance scenarios and hashes of all authored inputs. Its default ceiling is 65,536 UTF-8 bytes. It refuses overflow rather than silently dropping instructions. This is a byte limit, not a token estimate. A deliberately reviewed larger budget can be passed with `--max-bytes`.

`check --skip-schema` explicitly leaves JSON Schema validation **not run**; it is not an equivalent full check. The validator checks local link targets, not external URLs or Markdown anchor validity. Hashes detect changes, not approval or correctness. Local `.venv`, Git metadata, build output and dependency trees are pruned; authored-input symlinks are refused. Keep the package separate from the application source tree as described in `START_HERE.md`.

## Repository gate

The application repository runs `just check-spec` in `just ci`, and CI runs the same two commands on every pull request. That gate runs the tool tests and `check --skip-schema`, so cross-file references, evidence paths and generated views cannot drift unnoticed. It does not run JSON Schema validation and it does not rewrite `evidence/validation.json`, which stays a dated record of the full check someone actually ran.

## Task status and gates

`plan/tasks.json` is the sole task-status registry. `gaps` identifies issues the task must read/investigate. `must_close_gaps` identifies issues that must be resolved before that task can be marked `done`. An investigation can finish with an evidenced finding that a capability is still missing. A parity/release task cannot declare success while its required gate remains unresolved.

`plan/acceptance.json` holds scenario outcomes separately from task progress. Initially every scenario is `not_run`. Each `passed` or `failed` entry requires evidence. `plan/gaps.json` records the substantive resolution and evidence before its status can become `resolved`.

These tools check structural consistency and that each evidence reference names a file in the package, **not the authenticity or sufficiency of a human's or agent's evidence**. Reviewers must inspect commands, exact code revisions, output and native runtime behavior independently. A forged evidence path or manually edited status is not verification.

## Reproducibility

The pinned requirement records the JSON Schema validator version used to check the package. It is not an application dependency. The standard-library tests include negative fixtures for cycles, duplicate IDs/keys, non-finite JSON, path escape, missing evidence, unresolved completion gates and unsafe outcome claims.

Generated views are `docs/DECISIONS.md`, `docs/IMPLEMENTATION_TASKS.md` and `READING_COPY.md`. Edit their source documents/registries, not both representations. The combined reading copy is for human review; a coding agent should generally use the smaller task context.
