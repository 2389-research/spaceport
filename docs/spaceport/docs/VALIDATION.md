# Validation report — Spaceport specification 1.0

**Scope: specification artifacts and their local tools only. No Spaceport application was forked, built or exercised in this task. No remote repository was changed.**

## Executed checks

| Check | Result |
|---|---|
| User decision ledger | 57 unique decision IDs |
| Normative requirement coverage | 26 requirements, each assigned to tasks and acceptance scenarios |
| Task dependency graph | 13 tasks; references valid; no cycles |
| Application scenario registry | 55 planned scenarios; all `not_run` |
| Capability/removal/gap references | 10 capability entries, 19 disposition entries, 6 open gates |
| Source references | 12 source entries with exact commit pins and read-scope notes |
| Authored JSON parsing | Passed; duplicate keys and non-finite numbers refused |
| Control JSON Schema and synthetic examples | Passed; 2 examples validated |
| Local Markdown file links and generated decision/task views | Passed; anchors and external URLs not validated by the tool |
| Specification-tool unit tests | 21 passed; no failures, errors or skips |
| Task-packet bounds | All 13 fit the 65,536-byte ceiling; largest 49,037 UTF-8 bytes |
| Input-change detection | SHA-256 input manifest included; local environments/build trees excluded |

Runtime used for these checks: Python 3.13.5, jsonschema 4.26.0. These versions describe specification validation, not the application toolchain.

Machine-readable reports: `evidence/validation.json` and `evidence/tool-tests.json`. Full tooling test output: `evidence/spec-tool-tests.txt`. Input hashes provide change detection, not proof of approval or actual runtime behavior. The validator checks evidence-reference presence, not authenticity or sufficiency.

## Not executed

**Application acceptance scenarios executed: 0.** Source review is not product conformance. No native macOS/Windows/Linux build or UI test, real Herdr startup/attach/chat/forwarding test, tailnet test, permission race test, product performance measurement, package/signing/update test or production deployment occurred.

The upstream audit is limited to the exact source sections recorded in `plan/sources.json`; it is not a whole-repository dependency audit or a guarantee that candidate deletion paths have no surviving consumers. T00 must establish that baseline in the implementation environment.

G-CHAT, G-PORT, G-FILE, G-OBS, G-RELEASE and G-PLATFORM remain open. Specification checks do not close them. Every task starts `not_started`; no task is represented as completed implementation.

## Reproduce

From the package root, after installing `tools/requirements.txt`:

```sh
python tools/spec.py render
python tools/spec.py check --write-report
python tools/test_spec.py
python tools/spec.py ready
python tools/spec.py context T00
```

The generated combined reading copy is refreshed by `render`. The registries and component documents remain authoritative.
