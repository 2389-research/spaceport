# Signed macOS preview 20261110.1

D61 preview policy, identity and credential separation remain as documented in
[the signing workflow evidence](t12-macos-signed-preview.md). This later preview
contains the Teleport/credential tails and desktop chat lifecycle removals.
Source is `9bbc81c70889f1f6da29ed2cecee493e8c3afe35`; subsequent documentation commits do not
change these built bytes. It is a local arm64 debug preview, not a public release.

| Observation | Actual result |
| --- | --- |
| App identity | `ai.2389.spaceport`, Team `HD9NM9NSMK` |
| Version / profile / architecture | `20261110.1` / debug / arm64 |
| App notarization | `01c8c9b2-9101-47f1-9e22-786b425f4df5`, Accepted |
| DMG notarization | `ddde4339-ac71-4b0e-ae7e-1e3e12f690da`, Accepted |
| DMG | `Spaceport-20261110.1-arm64-apple-darwin.dmg`, 53,766,420 bytes |
| DMG SHA-256 | `4a3fba4df4f545a01ab6232e5343ecdecf9694431493411d0164d6f2836c580b` |
| Shipped app executable SHA-256 | `ceeda2745f506d794c0b79d76674fdfc64857bea62232ff77fec6f6dd43d193f` |
| Shipped adapter SHA-256 | `69e8c5d98b834497e12f436eb3fbd04d55ebf9b51172e6f4e5259ffdb7533c00` |
| App and DMG verification | Strict codesign, stapled-ticket validation and Gatekeeper assessment passed; Notarized Developer ID |
| Actual shipped executable | Compiled preview identity and canonical `scripts/release/smoke-cli.py` passed from a read-only DMG mount |
| Actual shipped signed adapter | Both private Core component fixtures passed; same adapter hash before/after |

The unsigned build used Rust 1.99.0, locked dependencies, CARGO_INCREMENTAL=0,
`build-macos.sh 20261110.1 debug arm64 unsigned` and task-local cargo-about 0.9.2.
It compiled, generated actual third-party notices and passed CLI smoke before
assembly. `sign-macos.sh` then used the approved matching P12 in an isolated
temporary keychain. Public certificate fingerprint and exact identity/Team were
checked. Each app/DMG had one submit; persistent accepted receipts remain at
`target/notarization/ai.2389.spaceport-20261110.1-arm64/{app,dmg}/`.
The user keychain search list matches its exact pre-run bytes, temporary signing
state was removed, and the login keychain was not unlocked or reconfigured.

Delivery is `target/test-builds/signed/20261110.1/`: DMG, metadata, structured
`qualification.json` and the signed app copied from the read-only image. The
copy passed signature, ticket and Gatekeeper checks again; both executables
are arm64. Only `spaceport` and `sheprd-acp` are in Contents/MacOS. Required
notices remain; no signing key or Core runtime is packaged. The adjacent
assembler app under target/distribution is unsigned, not the delivered app.
The inspection canonicalized its private mount path and detached only its own
image device. Inherited hdiutil create/detach deprecation warnings remain.

The signed helper then ran the existing isolated Core fixtures via explicit
HERDR_CORE_TEST_BINARY and SHEPRD_ACP_TEST_BINARY selections: two passed, zero
failed. Core is separately installed 0.1.19, SHA-256
`98b0653b40f22028eea6020b6bf3e39ff77e6012286437ca9f8efd24b7ecd758`, unchanged
afterward. Exact private terminal kill, Core child reaping and sandbox deletion
were asserted. [Detach evidence](t06-adapter-detach-process.md) explains the
process/component scope and deterministic ACP peer. This proves the signed
companion can perform those contracts; it is not real-agent or native GUI proof.

No GUI was launched, installed app replaced, real conversation used, personal
runtime selected, hosted workflow dispatched, secret uploaded, release published
or updater enabled. Preview 20261109.1 is retained separately. Native UI/input,
real-agent permissions, remote chat, installation isolation and broader platform
acceptance remain unrun. T12, G-RELEASE and G-PLATFORM remain blocked/open;
[the route audit](t06-core-duplex-route-audit.md) keeps G-CHAT open. No new
product decision or user answer was required for this bounded phase.
