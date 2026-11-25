# Owner rulings, 25 November 2026

Four decisions, asked and answered this evening.

1. **Typing boxes (P2): start small.** Build the shared text-field crate with
   `SearchInput` only, see how it feels, then decide on the full kit
   (`DialogInput`, the rest). Menu page bodies stay parked until the first
   piece lands and earns the rest.
2. **Translation lookup (P1): move into `spaceport-i18n`.** The crates hold
   the labels' keys; the i18n crate holds the words. This unblocks menu
   stage 6 (3,692 lines) and eventually retires the app-side `fl!` sites
   (~305 remain).
3. **Signing environment: leave open.** The `macos-preview` environment keeps
   no required reviewers. The risk is accepted as recorded: anyone who can
   push a tag can publish a signed, notarized build. Branch protection on
   main remains the only gate.
4. **About box (#245): shipped, no sound.** The demoscene showcase goes into
   every build, visuals and real-time rendering included; sound stays in the
   notification path and never plays in the About box.
