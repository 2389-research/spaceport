# Unused provider credential-store removal

Base `749a117a5`; code commit `c83835788`, independently reviewed before commit.
The consumer audit found no retained reader or writer. Only a release-policy
re-export, dead error/localization/log entries and fixtures kept the store linked
after the provider, GitHub and cloud removals.

Deleted `sheprd-credentials`, its OS/private-file implementations, dead app error
and locale routes and dependency declarations. No actual credential, keychain,
private file or saved account was read, erased or migrated. Generic credential
and private-file security rules still apply to retained code.

The original `option_env!("HERDR_RELEASE_VERSION").is_some()` expression now lives
in app constants, with its production-policy meaning unchanged. The unconditional
legacy release compile refusal remains. Signed macOS previews keep this policy
false and do not start automatic skill installation/refresh. Issue reporting
classification is unchanged. The strengthened regression compiles the actual
constants without a fake credential module: development and preview run with
false policy, and both legacy release combinations refuse compilation.

## Tests and dependency evidence

Two absence tests failed first on eight real stale graph/source assertions, then
passed. The constants test failed first after removing its fake module, then
passed after the actual owner moved. Thirteen identity integration tests, 77
release absence tests and 87 targeted app tests passed. The latter cover constants,
skill policy, issue classification, typed source chains/redaction and localization.
Only tests of the deleted store and its retired error variants were removed.

Strict app Clippy still fails on exactly the 253 inherited findings; diagnostic
code/message/primary file/multiplicity match the prior baseline, with none added
or removed by this bounded cut. Format, file-size, sorted and i18n checks pass.
The inherited `block v0.1.6` future-compatibility notice remains.

The lock delta removes only `sheprd-credentials 0.1.0`, `secrecy 0.10.3`,
`security-framework 3.7.0` and `security-framework-sys 2.17.0`; no upgrades.
`oo7` remains for Linux notifications/GPUI; `zeroize`, `rustix` and platform
libraries retain other resolved owners. The app's signing scripts and
entitlements did not change. The
[combined dependency comparison](t04-tail-dependency-comparison.json) records both cuts, and [integrated verification](t04-teleport-engine-removal.md#integrated-verification)
records the final source, full app/CLI/release counts and remaining lint/native
limits. Final review approved the combined source. This is not native or
cross-platform qualification.
