# T01: About display name

Base integration `1bb9be4c3`. The application menu already used Spaceport, but the About box still rendered a literal Sheprd and its page title did too. The box now reads the shared `WINDOW_TITLE` constant, and the localized page title names Spaceport. Existing menu tests now expect the already-established Spaceport labels and selectors. Runtime code, upstream attribution and external link targets are unchanged.

TDD: the existing pseudo-locale render test gained an assertion that the rendered text contains `WINDOW_TITLE`; it failed on the old Sheprd literal. The About filter then passed 5/5, including copyright/NOTICE, links and narrow headless layout. The menus filter passed 28/28, including keyboard navigation, quit dispatch, menu placement and existing chat menu checks. The first offline build could not fetch the pinned `socket2 0.6.5`; a locked network-enabled build fetched the exact dependency before tests ran. A first format check requested wrapping the new assertion; formatting corrected it.

This is a display-name correction, not a full identity acceptance. Upstream website/social links and icon artwork still need the production identity audit. Native glyph/OS menu behavior and screenshots were not checked; A42 remains not_run and T01 remains in progress.

Quick formatting, file-size, sorted-region, i18n and diff checks passed after formatting. Package validation and all 21 package tool tests passed. The Cargo test link retained the existing macOS compact-unwind warning and `block 0.1.6` future-compatibility notice. A review of the diff found no new runtime or external-action path.
