# T05 retained terminal parser check

The fork's `crates/sheprd-terminal` directory has no file differences from pinned Sheprd `806b49870ac7761b98cbcea4bd115aa2f2b00d75` (`git diff --no-index --stat` exited 0). The existing `fragmentation::STREAM` fixture covers SGR, cursor motion, Unicode wide and combining text, OSC title and links, input modes, scrolling, and alternate screen. Its tests compare whole input with every two-part split and byte-at-a-time input. Separate damage tests check resize and both alternate-screen transitions; input tests cover semantic keys, mouse and paste. This source and test comparison is the A10 parser continuity check, not a native UI result.

- Fork: `CARGO_TARGET_DIR=/tmp/spaceport-t00-target cargo +1.99.0 test --locked -p sheprd-terminal --lib`: 71 passed, 0 failed.
- Isolated pinned reference: `CARGO_TARGET_DIR=/tmp/spaceport-sheprd-reference-target cargo +1.99.0 test --locked -p sheprd-terminal --lib`: 71 passed, 0 failed.
- `git diff --no-index --stat /tmp/spaceport-sheprd-reference/crates/sheprd-terminal crates/sheprd-terminal`: no differences, exit 0.

The check preserves the existing parser contract without changing the terminal engine. It does not prove live Core attachment, native glyph painting, OS input, process survival, or Windows/Linux behavior; the rest of T05 and A11–A13 remain open.
