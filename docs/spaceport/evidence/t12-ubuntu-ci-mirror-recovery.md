# Ubuntu CI mirror recovery

On November 11, exact-main CI for `4a2604b4e` passed macOS/Windows checks but
failed Linux dependency installation before compilation. Logs report temporary
DNS failures for Canonical archive/security/ports hosts. GitHub's AMD64 mirror
list falls back among the same affected domains. The incident also appears in
[runner-images #15296](https://github.com/actions/runner-images/issues/15296).

CI explicitly opts into a temporary source list. Only Ubuntu URI fields change.
Canonical archive/ports are primary; [MIT's mirrors](https://mirrors.mit.edu/)
provide independent Ubuntu/Ubuntu-ports fallback after a failed metadata refresh.
Both preserve noble, updates, backports and security suites. The first attempt
used kernel.org for AMD64 and MIT for ARM64; kernel.org later became unreachable
from one runner. Canonical reported recovery and its HTTPS endpoints responded.

The helper preserves suites, components, comments, architecture fields and the
runner's `/usr/share/keyrings/ubuntu-archive-keyring.gpg` setting. Unsupported
origins, keys, suites, fields and architectures fail before APT runs. No trusted
key, authentication flag, required package, test or release gate changes. Host
source/key files remain untouched; ordinary developer invocation stays unchanged.
The temporary public configuration permits APT's normal `_apt` sandbox reads.

Local verification passed:

- Six real CLI regression tests, including eleven invalid-input subcases.
- ShellCheck, Bash syntax, actionlint and `git diff --check`.
- File-size, workflow-security and release-gate contract checks.

Hosted run [43066971913](https://github.com/harperreed/spaceport/actions/runs/43066971913)
completed AMD64/ARM64 dependency installation and default/all-feature Linux
checks. Logs show noble, updates, backports and security InRelease downloads
from the selected mirrors, with normal APT authentication and no GPG,
unsigned-repository, unauthenticated-package or unsandboxed-download warning.
macOS checks/builds, Windows checks/package and Windows ARM native checks also
passed. Windows x64 native GUI timed out returning to its original tab; the
overall gate failed and remains required. The follow-up adds bounded focus/
shell diagnostics without changing behavior, deadlines or retries.

Unknown future Ubuntu source formats fail with diagnostics and need an explicit
review before support is expanded.

The next run, `43072871856`, exposed an AMD64 mirror connection timeout. Plain
APT update reported warnings and returned success, so install then failed from
missing indexes. The correction requires `update --error-on=any`, disables
internal retries, bounds transport timeouts and tries at most two metadata
sources. Package installation runs exactly once after successful metadata.
Failed attempts retain named full logs. No package or authentication gate changes.

Real APT tests ran in an exact task-owned disposable amd64 Ubuntu 24.04 container,
official image digest
`sha256:fe81560990fae86dc4743202b2cb4d3df98d0bcf949173cecef15ce1127ef852`.
Primary success used one update/one install; blocked primary used two updates/
one install through MIT; both blocked used two updates/zero installs, even with
warm indexes. Source/keyring SHA-256 stayed unchanged. Test-only unreachable
HTTPS proxies caused real network failures; no fake APT or trust override ran.
The minimal image's package-owned omitted-manpage warnings remain in full logs.
Task containers/scratch were removed after copying artifacts into ignored
`target/apt-mirror-fallback/spaceport-apt-fallback.fhMQSZ/` in the fallback
worktree. ARM package installation and the combined source remain for fresh CI.
