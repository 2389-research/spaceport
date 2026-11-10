# Signed nebula preview 20261110.2

Doctor Biz explicitly requested a signed build to distribute. Signing access
and access to the approved notarization metadata both became available. The
existing [20261110.2 build](t12-macos-preview-20261110.2.md) was signed without
rebuilding it. Source remains `8f94bef21e6e47d682b4b8e317b1c852d013e9f5`.

| Observation | Verified result |
| --- | --- |
| App identity / Team | `ai.2389.spaceport` / `HD9NM9NSMK` |
| Version / profile / architecture | `20261110.2` / debug / arm64 |
| App notarization | `aa426f58-59ed-4972-8593-7dd88cae7cc2`, Accepted |
| DMG notarization | `cdb2fd2e-5665-42ee-ae37-3706b5785080`, Accepted |
| DMG | `Spaceport-20261110.2-arm64-apple-darwin.dmg` |
| Size | 59,507,149 bytes |
| SHA-256 | `d64535c27f2bf1751bef2436bcc4bf99b5f8b119124a8045f519d123ef38a536` |

The existing `sign-macos.sh` installed-identity mode signed the helper and app
with the approved 2389 Developer ID. Each artifact had one successful upload;
both Apple submissions returned Accepted. Persistent submission IDs, artifact
digests and status JSON remain under
`target/notarization/ai.2389.spaceport-20261110.2-arm64/{app,dmg}/`.
No private key was uploaded to GitHub or placed in a package. The signing
script did not import a P12, unlock a keychain or change its search list.

## Delivered bytes

Stable delivery in the integration checkout is
`target/test-builds/signed/20261110.2/`. It contains the signed DMG, metadata,
`qualification.json`, verification log and signed app copied from the image.
The adjacent assembler app under `target/distribution` remains unsigned;
distribute the verified signed DMG.

Actual checks passed:

- Strict signatures, stapled tickets and Gatekeeper assessment for the DMG,
  mounted app, and copied signed app.
- Compiled preview identity and canonical CLI smoke on the exact signed
  executable mounted read-only from the final DMG.
- Only `spaceport` and `sheprd-acp` are in Contents/MacOS; both are arm64.
- The packaged ICNS and Assets.car match the generated nebula development
  variant byte-for-byte. The approved development PNG is embedded in the app.
- Package scan found no `.p8`, `.p12` or `.key` files.
- DMG digest matches its signing metadata. The private read-only mount was
  detached by its exact device after verification.

This signed preview is suitable for the requested manual distribution; no
GitHub release was published or application installed. It remains a debug
Apple Silicon preview. Native Dock/About/full application QA is unproved,
and broader platform, runtime, update and production acceptance gates remain
open. Previous signed and unsigned artifacts were preserved.
