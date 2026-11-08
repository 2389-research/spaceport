# T10 phone-only workspace dependencies

The deleted phone service and nearby helper left seven direct entries in the
workspace dependency table: `rcgen`, `x509-parser`, `roxmltree`, `aes-gcm`,
`hkdf`, `p256`, and `time`. No surviving member manifest inherited these
entries. This cut removes the declarations and their phone-only comments; it
does not change any retained crate's dependency or claim that transitive
packages with the same names have left `Cargo.lock`.

The new release regression first failed on `rcgen` in the workspace manifest,
then passed after the cut. The phone absence suite passed 4/4.
The full Python release suite passed 189 tests with eight platform skips.
`cargo +1.99.0 metadata --locked --format-version 1` passed without rewriting
the lockfile. `git diff --check` passed. No native process, installer, or
network inventory ran, so A31 remains `not_run` and T10 remains in progress.

The desktop manifest also enabled `rustix/system` only to advertise the host
name over the deleted Bluetooth helper. A new assertion first failed on that
feature declaration, then passed after replacing it with the existing Unix
`rustix` dependency without extra features. The phone absence suite passed
4/4; production `cargo +1.99.0 check --locked -p sheprd-gpui --bin sheprd`
passed. Cargo printed the pre-existing `block v0.1.6` future-compatibility
notice. This does not change the A31 native inventory gate.
