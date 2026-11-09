# T04: combined headless regression corrections

At integration `7af7d2249`, the full all-feature headless application suite reported 3,994 passed, ten failed and nine ignored. This slice owns six failures; separate pseudo-locale fixture work owns the other four. No native or live personal runtime was used.

Three failures compared localized messages with their typed errors' Display output. The chat-start error now uses the same capitalization; the clipboard reader error uses the shared Spaceport name instead of Sheprd; the generic credential-policy error and its message now say only that no secure store is configured. The latter removes obsolete GitHub/Coder plaintext-setting advice without changing credential policy or access. All 58 error-localization tests passed.

The deleted browser's blank-tab port-chip test now checks the retained Devices tab's group menu. A simulated click opens its exact loopback URL through the system handler and leaves the native tab visible, with no browser placeholder. All seven group-menu new-tab tests passed. This is a headless handler check, not a real forwarded-service or OS browser test.

Settings search still indexed the removed Git change-count row and workspace-close confirmation. Their catalog entries and the unused confirmation message are removed; current retained controls are unchanged. The first rerun exposed the second stale entry after the exhaustive check stopped at the first. Source inspection of the full catalog and render owners found no further missing static row IDs; font rows use the existing generated selectors. All nine settings-search tests then passed. A format check requested the shorter credential-error attribute; formatting corrected it.

Quick format/file-size/sorted/i18n/diff checks passed. This does not close T04, native parity, credential-provider removal or release gates. It preserves the existing exhaustive catalog and error checks instead of loosening them.
