# Sparkle preview 20261111.2: upload stall

Status: no distributable or signed feed published. This version is reserved;
never rerun its workflow or blindly resubmit its upload.

## Verified source and tests

- Main source: `df539fd92eb652b2739747284e3c5c3f2be71478`, merged by PR #3.
- [Full main CI 43080384560](https://github.com/harperreed/spaceport/actions/runs/43080384560)
  passed every required job, including Windows native/performance and Linux
  release CLI suites. Full PR CI also passed before merge.
- Fresh signed native ZIP/DMG each installed v2 and relaunched exactly once.
  Eight negative cases and the trusted publisher identity-rotation observation
  completed; see the [native record](t12-sparkle-native-20261111.md).
- Annotated tag `v20261111.2` points to the main source above. Tag object:
  `abcea3df980c7756bef33d882ac9ae1123975838`.

## Hosted preview outcome

[Workflow 43082879618](https://github.com/harperreed/spaceport/actions/runs/43082879618)
passed request validation and its exact-main CI gate. The unsigned arm64 release
build passed actual CLI smoke. Artifact `13886610950` retains that unsigned input.
Build logs contain the already recorded toolchain override, yanked git-cliff
lockfile entries and `block 0.1.6` future-compatibility notices.

The protected signing job began at 19:14 UTC. Its retained receipts and terminal
logs establish this sequence:

| Stage | Evidence |
| --- | --- |
| Signed app submitted | UUID `6a8da49f-f2a3-48cb-88e7-f78c37addbd5`, 19:14:54 UTC |
| App notarization | Accepted; independent Apple log reports Ready for distribution, 26 ticket entries, zero issues |
| App ticket and policy checks | Stapling, validation, strict codesign and Gatekeeper passed by 19:15:23 UTC |
| DMG creation | Completed at 19:15:29 UTC |
| DMG submission | Upload command never returned a UUID or diagnostic |
| Job termination | GitHub cancelled the command at 20:14:34 UTC after the 60-minute job limit |

The accepted app ZIP SHA-256 in both Apple's log and the CI receipt is
`61ef0ec3153467c1f86751489c88deb487647248d0c8bb36d6b6f3d90c7e2b4e`.
The unpublished DMG input hash is
`b3ca0bd3e3e8edf60e8532f0e949eef2c3f59eb972e3ad25ad8e3553cf3dd527`.
Its submission JSON and stderr are empty, and no submission ID/status file exists.
Read-only Apple history showed no matching new DMG submission. This does not
prove that an uncertain upload can safely be repeated; keep the version reserved.

Artifact `13889952359` retains 12 files of release notes and submission receipts.
The temporary signed payload was not promoted to a distributable. Final image
verification, Sparkle archive/feed signing and publication did not run.

## Root cause boundary and next action

The script bounds processing waits at 20 minutes but leaves `notarytool submit`
unbounded. It redirects the upload's output into receipt files, so the combined
GitHub step gave no useful live phase detail through the API. Direct Apple
history would have narrowed the problem sooner; repeating that job label did
not diagnose it. The confirmed stall is the DMG upload command. Its underlying
network failure is not established by the empty diagnostics.

The follow-up bounds upload execution, preserves partial receipts on timeout,
reports safe stage progress and tests Apple's documented standard upload route.
[Apple documents](https://developer.apple.com/documentation/security/customizing-the-notarization-workflow)
that `--no-s3-acceleration` selects the standard S3 endpoint; it preserves Apple's
authentication and validation. No upload retry, forced submission or relaxed
gate is authorized by this diagnosis. A fresh version and successful actual
workflow must qualify the fix before publication. The baseline's real public
update and broader T12/A43 gates remain open.
