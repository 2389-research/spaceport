default:
    @just --list

# Launch the bundle so macOS reads its identity and privacy declarations.
run *args:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "$(uname -s)" != Darwin ]; then
        # The app starts chats with sheprd-acp beside its executable.
        cargo build --locked --release -p sheprd-acp
        exec cargo run --locked --release -p sheprd-gpui -- {{args}}
    fi
    {{just_executable()}} bundle release
    # Open the bundle instead of executing its binary: macOS asks for Local
    # Network access (Bonjour, SSH hosts on the LAN) only for an app it
    # launched, and denies a binary started from a shell without asking.
    # Output still reaches this terminal; Ctrl-C quits the app.
    app="$PWD/target/release/Spaceport.app"
    out=/dev/null
    if [ -t 0 ]; then out=$(tty); fi
    env_args=()
    while IFS= read -r variable; do
        env_args+=(--env "$variable")
    done < <(env | grep -E '^(RUST_LOG|RUST_BACKTRACE|HERDR_[A-Z0-9_]*)=' || true)
    trap 'pkill -f "^$app/Contents/MacOS/spaceport" || true' INT TERM
    open -n -W --stdout "$out" --stderr "$out" ${env_args[@]+"${env_args[@]}"} "$app" --args {{args}}

run-debug *args:
    #!/usr/bin/env bash
    set -euo pipefail
    if [ "$(uname -s)" != Darwin ]; then
        cargo build --locked -p sheprd-acp
        exec cargo run --locked -p sheprd-gpui -- {{args}}
    fi
    {{just_executable()}} bundle debug
    exec target/debug/Spaceport.app/Contents/MacOS/spaceport {{args}}

format:
    cargo fmt --all

format-check:
    cargo fmt --all -- --check

# Check both feature sets so optional test and platform code stays honest.
lint:
    cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
    cargo clippy --locked --workspace --all-targets --no-default-features -- -D warnings

test:
    cargo test --locked --workspace
    cargo test --locked --workspace --all-features

# The same suites under cargo-nextest, a process per test (see .config/nextest.toml).
# Doctests do not run here; `just test` remains the CI gate.
nextest *args:
    cargo nextest run --locked --workspace {{args}}
    cargo nextest run --locked --workspace --all-features {{args}}

# Explicit opt-in: launches and cleans up its own isolated daemon only.
test-live binary:
    HERDR_TEST_BINARY="{{binary}}" cargo test --locked -p sheprd-client --test live -- --ignored --nocapture

# Explicit opt-in: runs its own herdr-core (Herdr 2) in a private home and
# socket, and stops only that core.
test-core binary:
    HERDR_CORE_TEST_BINARY="{{binary}}" cargo test --locked -p sheprd-core-client live -- --ignored --nocapture

# Opens a real native window; requires an active desktop. Uses an isolated daemon.
test-gui binary:
    HERDR_TEST_BINARY="{{binary}}" cargo test --locked -p sheprd-gpui --features integration-test --test live_gui -- --ignored --nocapture --test-threads=1

# Repeat the Linux suite under CPU pressure in a container, where scheduling
# races reproduce that a fast machine hides. Needs Docker; nothing else.
stress-linux iterations="30" cpus="0.5" filter="":
    bash scripts/stress-linux.sh {{iterations}} {{cpus}} {{filter}}

# Native font/glyph regression across repeated frames and sizes; no daemon needed.
test-sidebar:
    cargo test --locked -p sheprd-gpui --features integration-test --test live_gui native_sidebar -- --ignored --nocapture --test-threads=1

# Native hover/scroll CPU scene budget in milliseconds, calibrated for this machine.
test-perf budget="30":
    cargo build --locked --release -p sheprd-gpui --features integration-test
    HERDR_PERF_P95_MS="{{budget}}" target/release/spaceport --performance-test

build-release:
    cargo build --locked --release -p sheprd-gpui -p sheprd-acp

# Compile and install the release executable into Cargo's bin directory.
[unix]
install:
    cargo install --locked --force --path crates/sheprd-gpui --bin spaceport
    cargo install --locked --force --path crates/sheprd-acp

# Only installation bypasses the POSIX shell used by the other recipes.
[windows]
[extension(".cmd")]
install:
    #!cmd.exe /c
    cargo install --locked --force --path crates/sheprd-gpui --bin spaceport || exit /b 1
    cargo install --locked --force --path crates/sheprd-launcher || exit /b 1
    cargo install --locked --force --path crates/sheprd-acp || exit /b 1

# Regenerate circular Linux artwork on any platform (Python standard library).
icons-linux:
    python3 scripts/generate-linux-icons.py

# Regenerate the Windows executable icons and the installer's images (Python
# standard library and rsvg-convert).
icons-windows:
    python3 scripts/generate-windows-icons.py
    python3 scripts/generate-installer-images.py

# Regenerate all checked-in artwork on macOS (requires brew install librsvg).
icons: icons-linux icons-windows
    swift scripts/generate-icons.swift

# Validate committed platform artwork without invoking native image generators.
check-artwork:
    PYTHONDONTWRITEBYTECODE=1 python3 assets/icons/test_artwork.py

# Local, unsigned bundle with the app's companions beside it.
# Never installs or starts a daemon.
bundle profile="release" features="":
    #!/usr/bin/env bash
    set -euo pipefail
    test "$(uname -s)" = Darwin
    case "{{profile}}" in
        release) flags=--release ;;
        debug) flags= ;;
        *) echo "Unknown profile: {{profile}} (release or debug)" >&2; exit 2 ;;
    esac
    app=target/{{profile}}/Spaceport.app
    cargo build --locked $flags --target-dir target -p sheprd-gpui --features "{{features}}"
    cargo build --locked $flags --target-dir target -p sheprd-acp
    mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
    # Replace the executables rather than rewrite them in place: macOS keeps
    # a signed file's old signature with its inode, so reading the rewritten
    # pages kills the reader, and a chat still running the old sheprd-acp
    # keeps its own copy.
    rm -f "$app/Contents/MacOS/spaceport" "$app/Contents/MacOS/sheprd" "$app/Contents/MacOS/herdr-gpui" "$app/Contents/MacOS/sheprd-acp" "$app/Contents/MacOS/herdr-acp"
    cp target/{{profile}}/spaceport "$app/Contents/MacOS/spaceport"
    # Chats the app starts run the sheprd-acp beside its executable.
    cp target/{{profile}}/sheprd-acp "$app/Contents/MacOS/sheprd-acp"
    # A stale development bundle must not keep a previously packaged Core.
    rm -f "$app/Contents/MacOS/herdr-core"
    cp assets/macos/Info.plist "$app/Contents/Info.plist"
    for lproj in assets/macos/*.lproj; do
        if [[ -d "$lproj" ]]; then cp -R "$lproj" "$app/Contents/Resources/"; fi
    done
    # LaunchServices and the Dock cache icons by bundle ID, so every worktree
    # build sharing the release ID competed with the installed app's icon.
    plutil -insert CFBundleIdentifier -string "$(cat crates/sheprd-gpui/development-app-id)" "$app/Contents/Info.plist"
    plutil -replace CFBundleName -string Spaceport "$app/Contents/Info.plist"
    plutil -replace CFBundleDisplayName -string Spaceport "$app/Contents/Info.plist"
    icons=$(python3 scripts/release/build-icon.py macos "$app/Contents/MacOS/spaceport")
    cp "${icons%$'\n'*}" "$app/Contents/Resources/Sheprd.icns"
    cp "${icons##*$'\n'}" "$app/Contents/Resources/Assets.car"
    plutil -lint "$app/Contents/Info.plist"
    # The linker's ad-hoc signature names the bare executable and binds no
    # Info.plist, so services that trust the bundle ID refuse the app:
    # notification authorization fails without a prompt. Ad-hoc sign the
    # assembled bundle so its identity is spaceport.development.
    codesign --force --sign - "$app"

# Link the actual optimized application and exercise its CLI without a desktop.
test-build: build-release
    cargo test --locked --release -p sheprd-gpui --test cli

# Fail on source files over 1,000 lines.
check-file-size:
    bash scripts/check-file-size.sh

# Fail when a `keep-sorted start`/`end` region is out of order.
check-sorted:
    python3 scripts/test_check_sorted.py
    python3 scripts/check-sorted.py

# Fail when a Fluent message does not parse, repeats, or is unused, or an
# `fl!` names none (docs/i18n.md).
check-i18n:
    PYTHONDONTWRITEBYTECODE=1 python3 scripts/test_check_i18n.py
    python3 scripts/check-i18n.py

# Sort every `keep-sorted start`/`end` region in place.
sort:
    python3 scripts/check-sorted.py --fix

test-pr-closure:
    PYTHONDONTWRITEBYTECODE=1 python3 scripts/test_close_low_confidence_prs.py

ci: format-check check-file-size check-sorted check-i18n check-artwork lint test test-pr-closure

# Tools are listed in scripts/setup-ci.sh; Docker for lint-linux is only checked.
# Install the tools local CI-equivalent runs need, on macOS or Ubuntu.
setup-ci:
    bash scripts/setup-ci.sh

# Cross type-check the Windows target without a Windows machine. CI lints the
# MSVC target on a Windows runner; this uses the GNU target because a Mac or
# Linux host cannot supply the MSVC C toolchain some dependencies build against.
# Needs `rustup target add x86_64-pc-windows-gnu` and mingw-w64
# (`brew install mingw-w64`, or `apt install gcc-mingw-w64-x86-64`).
lint-windows:
    CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc \
    AR_x86_64_pc_windows_gnu=x86_64-w64-mingw32-ar \
    cargo clippy --locked --workspace --all-targets --all-features \
        --target x86_64-pc-windows-gnu -- -D warnings
    CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc \
    AR_x86_64_pc_windows_gnu=x86_64-w64-mingw32-ar \
    cargo clippy --locked --workspace --all-targets --no-default-features \
        --target x86_64-pc-windows-gnu -- -D warnings

# Clippy for Linux in the Ubuntu 24.04 that CI uses, from any host with Docker.
# A cfg gate that is wrong only on Linux is invisible from macOS, and
# lint-windows only covers Windows. The container runs the host architecture;
# pass a platform, for example linux/amd64 on Apple Silicon, for the other one.
lint-linux platform="":
    bash scripts/lint-linux.sh {{platform}}

# Audit the GitHub workflows for injection, over-broad permissions, and unpinned actions.
audit-workflows:
    zizmor .github/

# License, advisory, and source checks over the dependency graph.
audit-deps:
    cargo deny check

# Sign published release artifacts with the local GPG key (YubiKey). Not a CI step.
sign-release *args:
    ./scripts/gpg-sign-release.sh {{args}}

# Verify published checksums, Sigstore and provenance; optional local GPG approval.
verify-release *args:
    ./scripts/verify-release.sh {{args}}

# Check commit subjects the way CI does; the changelog is generated from them.
check-commits base="origin/main":
    python3 scripts/release/check-commit-messages.py range "{{base}}..HEAD"

# Install the hooks, and the merge settings that cut repeated conflict work,
# for this checkout and every worktree of it.
hooks:
    git config core.hooksPath .githooks
    git config rerere.enabled true
    git config rerere.autoupdate true
    git config merge.cargo-lock.name "Cargo.lock: keep one side, then rebuild with Cargo"
    git config merge.cargo-lock.driver "bash scripts/git/merge-cargo-lock.sh %O %A %B"
    @echo "Hooks, rerere, and the Cargo.lock merge driver are configured."

# What changed since the last release tag, straight from git history.
changelog-unreleased:
    git-cliff --config cliff.toml --unreleased

# The whole generated changelog; it is never tracked in git.
changelog:
    git-cliff --config cliff.toml

# Exactly what a release would publish: CHANGELOG.md and RELEASE_NOTES.md.
# Windows counts as signed once WINDOWS_SIGNING_SUBJECT is set, as in
# release.yml; set WINDOWS_SIGNED=true or false to preview either without gh.
changelog-release version output:
    #!/usr/bin/env bash
    set -euo pipefail
    if [[ -z ${WINDOWS_SIGNED:-} ]]; then
        if [[ -n $(gh variable get WINDOWS_SIGNING_SUBJECT --repo sheprd-labs/sheprd 2>/dev/null) ]]; then
            WINDOWS_SIGNED=true
        else
            WINDOWS_SIGNED=false
        fi
    fi
    WINDOWS_SIGNED=$WINDOWS_SIGNED bash scripts/release/generate-changelog.sh "{{version}}" "{{output}}"

# Owner-only remote release; CI derives the YYYYMMDD.COUNTER version itself.
release:
    bash scripts/release/dispatch.sh

# Owner-only beta: a GitHub prerelease kept out of Homebrew until promoted.
release-beta:
    bash scripts/release/dispatch.sh --beta

# Owner-only: make a published beta the latest stable release, then update Homebrew.
release-promote version:
    bash scripts/release/dispatch.sh --promote "{{version}}"

# Local signed/notarized preview; uses the installed identity, never publishes.
dmg $VERSION profile="debug" architecture="arm64":
    bash scripts/release/build-macos.sh "$VERSION" "{{profile}}" "{{architecture}}" installed

# Static checks and mocked release tests; does not dispatch or publish anything.
release-check:
    for script in scripts/*.sh scripts/release/*.sh; do bash -n "$script" || exit; done
    PYTHONDONTWRITEBYTECODE=1 python3 -m unittest discover -s scripts/release/tests -v
    PYTHONDONTWRITEBYTECODE=1 python3 scripts/release/test-release-security.py
    actionlint .github/workflows/*.yml
    zizmor .github/
    git diff --check
