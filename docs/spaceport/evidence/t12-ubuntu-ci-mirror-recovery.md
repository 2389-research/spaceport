# Ubuntu CI mirror recovery

On November 11, exact-main CI for `4a2604b4e` passed macOS/Windows checks but
failed Linux dependency installation before compilation. Logs report temporary
DNS failures for Canonical archive/security/ports hosts. GitHub's AMD64 mirror
list falls back among the same affected domains. The incident also appears in
[runner-images #15296](https://github.com/actions/runner-images/issues/15296).

CI now explicitly opts into a temporary source list. Only Ubuntu URI fields
change: AMD64 uses the [kernel.org Ubuntu mirror](https://mirrors.edge.kernel.org/ubuntu/),
and ARM64 uses [MIT's Ubuntu ports mirror](https://mirrors.mit.edu/ubuntu-ports/).
Both served noble, noble-updates and noble-security metadata during investigation.
HTTPS responses alone do not prove archive authenticity; real APT verification
remains required on each architecture.

The helper preserves suites, components, comments, architecture fields and the
runner's `/usr/share/keyrings/ubuntu-archive-keyring.gpg` setting. Unsupported
origins, keys, suites, fields and architectures fail before APT runs. No trusted
key, authentication flag, required package, test or release gate changes. Host
source/key files remain untouched; ordinary developer invocation stays unchanged.
The temporary public configuration permits APT's normal `_apt` sandbox reads.

Local verification passed:

- Five real CLI regression tests, including eleven invalid-input subcases.
- ShellCheck, Bash syntax, actionlint and `git diff --check`.
- File-size, workflow-security and release-gate contract checks.

Hosted AMD64/ARM64 APT signature checks, real dependency installation and full
platform/build CI remain pending. Unknown future Ubuntu source formats fail
with diagnostics and need an explicit review before support is expanded.
