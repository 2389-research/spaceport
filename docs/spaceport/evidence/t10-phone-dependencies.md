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
