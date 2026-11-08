# T02 separately installed Core selection

T02 remains in progress; A03 is not passed.

The Core hub still connects to an already running Core first. When no Core answers and a start was requested, it now resolves `herdr-core` from an absolute directory on the configured PATH and excludes the GUI executable's directory by canonical path. A copy beside the GUI is not selected, even if that directory appears on PATH. Failure reports that no separately installed Core was found. No installer or updater is called by this path.

The new isolated-directory test failed to compile before the resolver existed, then passed with distinct app-adjacent and PATH binaries. Five bundled-helper tests and two hub transition tests passed on macOS. `cargo +1.99.0 fmt --all -- --check`, `just check-file-size`, and the specification package checks are the local gates for this slice. The existing linker compact-unwind warning and `block` future-compatibility notice remained.

This path selection test does not prove that an arbitrary PATH directory is trusted or that the selected binary is a compatible Core. The separately installed Core's product/version is learned only after its `hello` reply. The packaged Core copy and its release scripts still exist; T10 must remove those artifacts. Absent, stopped, incompatible and native Windows/Linux startup scenarios remain open.
