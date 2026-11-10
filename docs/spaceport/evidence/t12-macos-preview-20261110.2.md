# Nebula artwork preview 20261110.2

This records the initial unsigned build and historical access failures.
The same build is now signed and notarized; see
[the signed qualification](t12-macos-preview-20261110.2-signed.md).

Doctor Biz clarified that shipping the approved branding includes rebuilding
the application. This candidate was built from
`8f94bef21e6e47d682b4b8e317b1c852d013e9f5` with the selected nebula artwork.
It is an **unsigned arm64 debug preview**, not a notarized or public release.

## Actual build and package checks

The existing `scripts/release/build-macos.sh 20261110.2 debug arm64 unsigned`
route succeeded with `RUSTUP_TOOLCHAIN=1.99.0`, locked dependencies and the
existing cargo-about 0.9.2. The shell initially selected Rust 1.97.1 despite
the repository pin; the explicit selection fixed the build environment.
The inherited `block v0.1.6` future-incompatibility warning remains.

Compilation, canonical CLI smoke, actual third-party notice generation and
macOS assembly passed. The executable's compiled preview identity is
`20261110.2` / debug / `ai.2389.spaceport`. The packaged ICNS and Assets.car
are byte-identical to the generated development variants; the new development
PNG is also embedded in the rebuilt executable.

The unsigned ZIP is retained in the integration checkout:

`target/test-builds/20261110.2-unsigned/Spaceport-20261110.2-arm64-apple-darwin-unsigned.zip`

- Size: 50,680,360 bytes.
- SHA-256: `19f7ec7b94dc54d7e9765065a8356af1aca9995f70948cf5801c71bc129424db`.
- ZIP integrity passed. Its executable, ICNS and catalog match the assembled
  candidate and approved generated resources exactly.
- Adjacent `qualification.json` records the source, bytes and actual checks.

## Signing and native checks remain blocked

The installed, previously approved 2389 Developer ID identity was selected.
`sign-macos.sh` failed on the first helper signature with
`errSecInternalComponent`, before notarization. No app/DMG submission or receipt
was created, so no uncertain Apple-side operation needs retrying. No keychain
was unlocked or reconfigured, and no private key was uploaded or committed.
That failure initially required restored keychain access or a matching P12
password file. A later check on 2026-11-10 found the default keychain unlocked,
readable and writable; a scratch copy of the candidate helper then signed with
the approved 2389 identity and passed strict verification. Private-key access
is no longer the blocker, and the earlier unlock/password request is obsolete.

The resumed signing wrapper subsequently blocked opening the notarization
credential metadata in Downloads, before starting the signer or creating its
log. Stack sampling showed the blocked file open; a separate bounded file read
also timed out. The cause of that file-access failure is not established. The
exact wrapper was stopped and reaped, with no Apple submission or receipt.
The current remaining step is restoring access to the approved notarization
metadata, then signing the existing unsigned app without rebuilding it.
Preserve receipt checks before any future submission.

A throwaway copy used a distinct `ai.2389.spaceport.iconqa` identity, ad-hoc
signature, private HOME/XDG state and an explicitly absent test socket. It ran
and logged only failed attachment to that private socket; no personal daemon
or conversation was selected. The computer-use service could not find a
visible window (`cgWindowNotFound`), so Dock/About/native visual QA did not pass.
The exact test process was stopped and its exit observed. The existing live
Spaceport instance was left untouched. This test copy is not the deliverable.

Earlier signed previews remain intact. No application was installed, no public
release was published, and no updater or broader acceptance gate was enabled.
