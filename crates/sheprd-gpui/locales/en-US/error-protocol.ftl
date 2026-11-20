# Failures reading or writing what the daemon sends: a pane screen, the
# update that changes it, and a clipboard image. Each says what arrived, in
# a person's words, and leaves the wire's name for it to the log.
# $error is the decoder's own diagnostic.

error-protocol-cell-count = A pane screen arrived with the wrong number of cells for its size.
error-protocol-clipboard-image-extension = A clipboard image is in a format this app cannot read.
error-protocol-clipboard-image-size = A clipboard image must hold 1 byte through 16 MiB.
error-protocol-clipboard-image-target = A clipboard image's target must hold 1 through 1024 bytes.
error-protocol-cursor-bounds = A pane screen put the cursor outside the screen.
error-protocol-decode = A message from the daemon could not be read: { $error }
error-protocol-delta-grid-limit = A pane screen update is larger than the size limits this app accepts.
error-protocol-delta-metadata-cells = A pane screen update carried cells where its metadata belongs.
error-protocol-delta-popup = A pane popup update does not match the popup it belongs to.
error-protocol-delta-span = A pane screen update named cells outside the screen, out of order, or too many.
error-protocol-empty-frame = A message cannot be empty.
error-protocol-encode = A message could not be written for the daemon: { $error }
error-protocol-encoding-json = An encoded pane screen's JSON could not be read: { $error }
error-protocol-encoding-limit = An encoded pane screen is larger than a message may be.
error-protocol-encoding-payload = An encoded pane screen could not be decoded: { $error }
error-protocol-frame-limit = The message is outside the allowed size limits.
error-protocol-hyperlink-index = A pane screen named a link it does not carry.
error-protocol-io = Could not talk to the daemon: { $error }
error-protocol-patch-cursor-bounds = A pane screen update put the cursor outside the screen.
error-protocol-patch-geometry = A pane screen update tried to change the pane's size.
error-protocol-patch-identity = A pane screen update named a pane or revision it does not match.
error-protocol-patch-row-bounds = A pane screen update named a row the screen does not have.
error-protocol-scroll-bounds = A pane screen scroll named rows the screen does not have.
error-protocol-scroll-count = A pane screen scroll asked for an impossible number of lines.
error-protocol-scroll-payload = A pane screen scroll came without the screen lines it moves.
error-protocol-scroll-truncated = A pane screen scroll ended early.
error-protocol-surface-baseline = A pane screen update does not match the screen it belongs to.
error-protocol-trailing-bytes = The daemon sent extra bytes after a message.
