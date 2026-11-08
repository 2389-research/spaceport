# T12: Core stays outside Spaceport packages

Source baseline `37d41c8f` still fetched a pinned Core during release builds,
required it beside app binaries, signed it into macOS/Windows artifacts, and
installed it in Linux and Windows packages. The development bundle recipe also
copied it. This contradicted the separately installed Core contract. The
existing Core connection worker already used `bundled::locate_installed`, which
excludes the app's executable directory; that discovery path was left intact.

The new package-input test failed first because `package-linux.sh` required
`herdr-core` and `package-windows.py` required `herdr-core.exe`. After the cut,
both package scripts accept only the retained chat companion beside the app,
and their exact archive manifests contain no Core. The existing mocked macOS
assembly/signing test now uses no Core input or output. Release and CI workflows
no longer fetch or smoke-test Core as a package input. Linux distro, Windows
installer/signing, development bundle, and release docs no longer place Core in
Spaceport artifacts. The retired fetch script, pin and bundled license snapshot
are removed. Spaceport still refers to the external `herdr-core` executable at
runtime and keeps legal attribution for the client dependencies.

`python3 -m unittest scripts.release.tests.test_external_core` passed 2/2.
The release Python suite passed 187 tests with 9 skips; release security tests
passed 25 with 1 skip. These use isolated files and mocked macOS signing tools.
No native install, signed update, or platform package was run. The source
change prevents new packages from owning Core, but an old installed package
may still have an old bundled Core file. No automatic deletion, stop, or
upgrade of that file was added. Owner-safe cleanup is a separate migration gate.

The upstream production identities, signing keys and update source remain in
the release code. The compile-time release gate stays closed; this slice does
not satisfy A42–A44 or authorize publishing.
