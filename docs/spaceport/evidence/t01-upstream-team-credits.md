# T01: About credits for both upstream teams

Doctor Biz requested explicit references to the Herdr and Sheprd teams. D59 records that choice. The existing About layout now includes a credit sentence and two labeled upstream-team links, using the existing Herdr website and Sheprd source URLs. Project names use Fluent placeables so pseudo-localization preserves them verbatim. The old unqualified Website link to Sheprd's homepage is removed; no Spaceport website or account ownership was invented.

The existing NOTICE-derived copyright, author attribution and Apache-2.0 license remain. The independence disclaimer now names both upstream teams. This changes the About view only; public issue-reporting routes, updater/release URLs and production identity still need their release audit.

TDD: the pseudo-locale render test first failed because no rendered credit named both teams. After the change, all five About tests passed, including exact link targets, unchanged copyright attribution and the new credit row's bounds at 800- and 360-pixel widths. No actual browser, personal runtime or native window was opened; these are headless tests, not screenshot or OS-menu proof.

The static i18n checker first flagged the proper project-name strings in multiline Fluent arguments. Private name constants now identify those untranslated data values while the surrounding credit/team labels remain translated. The i18n check passed after that correction. Other quick checks, specification validation and all 21 package tool tests passed. Production source does not add I/O or side effects during rendering; upstream links still require explicit clicks.
