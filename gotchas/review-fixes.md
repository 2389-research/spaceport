# Review fixes

- ScrollHandle indexes direct children. Keep live rows direct. Capture snapped extents for departure. Reset form handles on cancel. See [popup evidence](../docs/evidence/review-host-popup-departure.md).
- Headless GPUI exposes element metadata. Qualify accessibility trees natively. Release dependencies disable debug selectors. Run selector GUI suites in debug. See [accessibility evidence](../docs/evidence/review-accessibility-gaps.md).
- zbus encodes fixed arrays as D-Bus structures. Pass slices for arrays. Verify the real wire signature with an isolated peer. See [OS accessibility evidence](../docs/evidence/review-os-accessibility.md).
- APFS rejects invalid UTF-8 filenames with EILSEQ. Preserve OS bytes in Unix APIs. Run real invalid-filename fixtures on Linux. See [CLI transport evidence](../docs/evidence/review-cli-contracts.md).
- A captured Unix process-group number does not preserve identity after reaping. Keep the leader waitable with NOWAIT until group signals finish. Retire signaling before the final reap. See [agent ownership evidence](../docs/evidence/review-jargon-main-integration.md).
- Keep instant headless motion inside GPUI's `cfg(test)` policy. Cargo dev features also reach normal native-test binaries. See [reorder integration evidence](../docs/evidence/review-reorder-integration.md).
- A test glob import from a parent containing gpui::* can import GPUI's test attribute. Import specific dependencies to prevent recursive test expansion.
- GPUI simulate_keystrokes sends only KeyDown and may commit text. Send matching KeyUp after a focused frame. Use raw key events for IME ownership. Headless draw does not deliver on_next_frame callbacks. Use simulate_next_frame or sidebar run_frames. See [platform shortcut evidence](../docs/evidence/review-platform-shortcuts.md).
