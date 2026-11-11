#!/usr/bin/env bash
# ABOUTME: Build a macOS preview, smoke its CLI, and optionally sign and notarize it.
# ABOUTME: Keep build processes free of signing credentials and retain notary receipts.
set +x
set -euo pipefail
source "$(dirname -- "$0")/common.sh"
[[ $# -ge 1 && $# -le 4 ]] || fail 'Usage: bash build-macos.sh VERSION [debug|release] [arm64|universal] [unsigned|installed|import] (defaults: debug arm64 unsigned)'
version_check "$1"
[[ -z ${HERDR_RELEASE_VERSION+x} ]] || fail 'HERDR_RELEASE_VERSION remains blocked for Spaceport'
version=$1
profile=${2:-debug}
architecture=${3:-arm64}
mode=${4:-unsigned}
[[ $(uname -s) == Darwin ]] || fail 'macOS required'
flags=(build --locked --features sheprd-gpui/macos-updater)
case $profile in debug) ;; release) flags+=(--release) ;; *) fail 'Profile must be debug or release' ;; esac
[[ $architecture == arm64 || $architecture == universal ]] || fail 'Architecture must be arm64 or universal'
[[ $mode == unsigned || $mode == installed || $mode == import ]] || fail 'Mode must be unsigned, installed or import'
cd -- "$release_root"
[[ -n ${SPARKLE_FRAMEWORK_PATH:-} ]] || fail 'SPARKLE_FRAMEWORK_PATH must contain the pinned SDK; use sparkle_framework.py acquire'
python3 scripts/release/sparkle_framework.py validate-sdk "$SPARKLE_FRAMEWORK_PATH"
# Raw CLI checks run before the framework is embedded into the app bundle.
export DYLD_FRAMEWORK_PATH="$SPARKLE_FRAMEWORK_PATH"
out="$release_root/target/distribution/$version-$profile-$architecture"
[[ ! -e $out && ! -L $out ]] || fail "Output already exists: $out"
# Cargo and dependency build scripts must not inherit signing credentials.
build_env=(env
    -u SPARKLE_PRIVATE_KEY
    -u MACOS_CERTIFICATE_P12_BASE64 -u MACOS_CERTIFICATE_PASSWORD
    -u MACOS_SIGNING_IDENTITY -u APPLE_API_PRIVATE_KEY -u APPLE_API_KEY_PATH
    -u APPLE_API_KEY_ID -u APPLE_API_ISSUER_ID
    SPACEPORT_MACOS_PREVIEW_VERSION="$version"
    MACOSX_DEPLOYMENT_TARGET=14.2 CARGO_INCREMENTAL=0
    # Apple strip can misalign host proc-macro dylibs during cross-compilation.
    CARGO_PROFILE_RELEASE_BUILD_OVERRIDE_STRIP=none)
# Build the host arm64 target without --target so local debug caches are reusable.
[[ $(uname -m) == arm64 ]] || fail 'Build on Apple Silicon; Intel-only hosting is not supported by this preview route'
"${build_env[@]}" cargo "${flags[@]}" --target-dir target -p sheprd-gpui -p sheprd-acp
intel=-
if [[ $architecture == universal ]]; then
    "${build_env[@]}" cargo "${flags[@]}" --target-dir target -p sheprd-gpui -p sheprd-acp --target x86_64-apple-darwin
    intel="target/x86_64-apple-darwin/$profile/spaceport"
fi
"${build_env[@]}" python3 scripts/release/smoke-cli.py "target/$profile/spaceport" >&2
mkdir -p target/distribution
stage=$(mktemp -d "$release_root/target/distribution/.spaceport-build.XXXXXX")
trap 'rm -rf -- "$stage"' EXIT
trap 'exit 130' INT
trap 'exit 143' TERM
"${build_env[@]}" python3 scripts/release/generate-notices.py "$stage/THIRD-PARTY-NOTICES.txt"
"${build_env[@]}" bash scripts/release/package-macos.sh "$version" "target/$profile/spaceport" "$intel" "$stage" "$stage/THIRD-PARTY-NOTICES.txt" "$profile" >&2
if [[ $mode != unsigned ]]; then
    bash scripts/release/sign-macos.sh "$version" "$stage/Spaceport.app" "$stage" "$architecture" "$mode" >&2
    dmg="Spaceport-$version-$architecture-apple-darwin.dmg"
    [[ -s $stage/$dmg ]] || fail 'Signing did not produce a nonempty DMG'
    result=$dmg
else
    echo 'Unsigned preview bundle only; no Developer ID signing or notarization performed.' >&2
    result=Spaceport.app
fi
new_output "$out"
mv -n "$stage" "$out"
[[ ! -d $stage ]] || fail "Could not promote output: $out"
printf '%s\n' "$out/$result"
