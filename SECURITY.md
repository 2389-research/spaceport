# Security Policy

Spaceport is a development fork with signed macOS previews.
Report suspected Spaceport vulnerabilities privately through
[Spaceport's security reporting form](https://github.com/2389-research/spaceport/security/advisories/new).
The report goes to this repository's maintainers.
Please do not open a public issue for a suspected vulnerability.

## Reporting a vulnerability

Include the affected version or full commit, operating system, reproduction steps,
and what an attacker could gain. Remove credentials, private terminal output,
and personal data from examples. The maintainers may ask for a smaller reproduction.

This project has no guaranteed response schedule or supported release series.
Fixes land on `main`; check the release notes for preview availability.

## Scope

Spaceport connects to separately installed Herdr 1 and Herdr Core runtimes.
The runtimes own terminal processes and sessions. Closing Spaceport leaves them running.

Relevant reports include:

- Malformed daemon messages that bypass framing, decoding, or resource limits.
- Incorrect socket access, host validation, or session targeting.
- Terminal content that causes unintended execution, clipboard writes, or file access.
- Access-control failures in local control or agent attach endpoints.
- Update verification failures or release artifact substitution.

Herdr runtime bugs belong to its maintainers. If ownership is unclear, use the private
Spaceport form first. Do not disclose a suspected vulnerability publicly to route it.
Reports about dependencies are welcome when they affect Spaceport.

## Preview integrity

The macOS preview route uses Apple signing and notarization.
Installed macOS previews use Sparkle updates with signed update archives.
Development and worktree builds do not participate in that update route.
See [macOS preview verification](scripts/release/README.md#spaceport-macos-previews)
for the artifact checks and publishing process.

Broader platform packaging and update qualification remain release gates.
The inherited production release workflow does not establish Spaceport release support.
See the [Spaceport specification](docs/spaceport/START_HERE.md) for current scope.
