# T03 selected remote document reader

At base `4cde5443`, the app observed only the hub's `spaceport` document. A
private two-Core client test had proved `doc.get(on=Machine.id)` survives a
machine rename, but the desktop had no remote document reader. This cut adds one
read-only document under the selected Core machine in the existing Devices
picker. The Workspaces sidebar still reads only the hub document, whose
`{core_id, terminal_id}` members may span hosts. The two documents are distinct
stores; no remote groups are merged into the hub's group list or edited here.

The app-global Core observer keeps its one `CoreChanges::next_batch`
subscription. The selected ID and generation are shared across windows. A
cloned-backend read thread calls `doc.get("spaceport", on=selected Machine.id)`;
the UI click only changes selection and renders the latest projection. Reads
start on selection and initial connected catalog, and restart on a remote
`spaceport` notice, `missed` batch, or disconnected-to-connected catalog
transition. Hub loss clears the old document; a new observer epoch reads it
after catalog refresh. Epoch, ID, selection generation, and request serial
discard late results. Disconnected, invalid/newer-schema, and failed reads
never show old group rows. The picker shows loading, stale, and bounded error
text, at most 20 group rows, at most 50 machine rows, and 160 Unicode scalar
values plus an ellipsis per displayed label or diagnostic.

Pinned Core mirrors a remote document notice with the machine's mutable
**name**, while `on` accepts its permanent ID. The observer therefore treats
any remote `spaceport` notice as invalidating the one selected ID. A batch
causes at most one remote read request, and the pending request and result
slots each hold only the latest value. An unrelated machine's notice can cause
an extra read; filtering it by catalog name could miss a selected machine's
old-name notice during rename or name reuse. A Core notice carrying immutable
ID would permit exact filtering. No new RPC, client SSH, remote write, process
kill, or second subscription was added.

Red: the first observer tests could not compile without remote selection; the
picker test could not compile without the document reader; the hub-loss test
failed because its stale message was absent. Green on macOS with
`CARGO_INCREMENTAL=0 CARGO_TARGET_DIR=/tmp/spaceport-t04-install-offer-target`:
focused `remote_document` app tests passed 6/6 (one opt-in case ignored);
the existing `core_catalog` filter passed 5/5; and production `cargo +1.99.0
check --locked -p sheprd-gpui --bin sheprd` passed with default and all
features, with no new code warnings.
The opt-in private Core app observer test passed 1/1 with
`HERDR_CORE_TEST_BINARY=/tmp/spaceport-t00-target/debug/herdr-core`: the selected
ID read version 0, a remote notice advanced it to 1, machine loss made it
stale, reconnect restored version 1, and a later notice advanced it to 2. The
hub document stayed at version 0. The fixture starts two isolated exact Core
children with a test-only `HC_SSH` relay, stops them through their private
sockets, and reaps them. No personal Core or environment was used.

`just format-check`, `just check-file-size`, `just check-sorted`,
`just check-i18n`, `git diff --check`, and package-root `spec.py check` passed;
the package check validates specification integrity, not app behavior. The app test link reported
the existing macOS compact-unwind warning; Cargo reported the existing
`block v0.1.6` future-compatibility notice. Native keyboard, accessibility,
scrolling, and visual QA were not run. A53 remains `not_run`: this slice did not
exercise a hub restart with a changed remote document while disconnected in
the desktop UI, nor prove old sequence-number isolation end to end. T03 stays
`in_progress`; remote terminal attach/action parity and other T03 gates remain.
The previous T03 route evidence has a PACT observation; this reader has
separate integration evidence below.

## Integrated branch verification

Merged as `53ec1bfa` after the Boxd and checkpoint-worker cuts. Default and
all-feature `sheprd-gpui --bin sheprd` production checks passed with
`--locked`; only Cargo's existing `block v0.1.6` notice appeared. The
merged GUI test binary passed six focused remote document tests (one opt-in
case ignored), and the retained Core catalog filter passed 5/5. The opt-in
selected remote document test passed 1/1 against the pinned Core binary in
two private sandboxes, including notice/reconnect behavior. `just
format-check`, file-size, sorted and i18n checks passed; i18n validated
5,006 messages in 263 files. The test linker repeated its existing macOS
compact-unwind warning. Package integrity and its 21 tooling tests passed.
The integrated checks still do not include native picker input, full hub
restart while the remote document changes, or Windows/Linux behavior; A53
remains not_run.
