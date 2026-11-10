#!/usr/bin/env bash
# ABOUTME: Generate release notes from the shared conventional-commit history rules.
# ABOUTME: Bound macOS preview notes by the last published commit, ignoring failed tags.
set -euo pipefail
source "$(dirname -- "$0")/common.sh"
usage='Usage: bash generate-changelog.sh VERSION OUTPUT_DIR [BETA_TAG...] or --macos-preview VERSION OUTPUT_DIR BASE_SHA'
preview=false
if [[ ${1:-} == --macos-preview ]]; then
    [[ $# == 4 ]] || fail "$usage"
    preview=true
    shift
fi
[[ $# -ge 2 ]] || fail "$usage"
version_check "$1"
[[ -d $2 ]] || fail 'Existing output directory required'
command -v git-cliff >/dev/null || fail 'git-cliff is required: cargo install --locked git-cliff'
out=$(cd -- "$2" && pwd)
version=$1
shift 2

if [[ $preview == true ]]; then
    base_sha=$1
    [[ $base_sha =~ ^[0-9a-f]{40}$ ]] || fail 'Preview base must be a full lowercase commit SHA'
    cd -- "$release_root"
    [[ $(git cat-file -t "$base_sha" 2>/dev/null) == commit ]] || fail 'Preview base must name an existing commit'
    git merge-base --is-ancestor "$base_sha" HEAD || fail 'Preview base must be an ancestor of HEAD'
    source_sha=$(git rev-parse HEAD)
    new_output "$out/CHANGELOG.md"
    new_output "$out/RELEASE_NOTES.md"
    # A pushed tag already exists, and failed preview tags must not divide this
    # published range. Render once so both artifacts contain identical history.
    git-cliff --config cliff.toml --tag "v$version" --ignore-tags '.*' \
        --strip header "$base_sha..HEAD" --output "$out/CHANGELOG.md"
    printf 'macOS arm64 preview: signed/notarized DMG, requires macOS 14.2+. Source: %s.\n\n' "$source_sha" > "$out/RELEASE_NOTES.md"
    cat "$out/CHANGELOG.md" >> "$out/RELEASE_NOTES.md"
    # A same-commit rebuild has no new history; retain only the factual preamble.
    [[ -s $out/CHANGELOG.md || $base_sha == "$source_sha" ]] || fail 'Generated changelog is empty'
    [[ -s $out/RELEASE_NOTES.md ]] || fail 'Generated release notes are empty'
    exit 0
fi

# Windows is signed, and so updates in the app, only once release.yml's
# validate job finds the signing publisher set. No default, so a preview can
# never promise a signature the release will not have.
case ${WINDOWS_SIGNED:-} in
    true) windows='Authenticode-signed Windows x86_64 and ARM64 installers and zips; installed copies update in the app' ;;
    false) windows='experimental, unsigned Windows x86_64 and ARM64 installers and zips; installed copies update in the app, except on Windows' ;;
    *) fail 'WINDOWS_SIGNED must be true or false' ;;
esac

# Unpromoted beta tags fold into the release after them, so a stable release
# (or a beta promoted to one) lists everything since the last stable release
# rather than only what changed since the newest beta.
ignore=()
range=()
if (( $# )); then
    pattern=''
    for beta in "$@"; do
        [[ $beta =~ ^v[1-9][0-9]{7}\.[1-9][0-9]*$ ]] || fail "Beta tag must be vYYYYMMDD.COUNTER: $beta"
        pattern+="${pattern:+|}${beta//./\\.}"
    done
    ignore=(--ignore-tags "^($pattern)\$")
fi
new_output "$out/CHANGELOG.md"
new_output "$out/RELEASE_NOTES.md"

# The tag is created only when the release publishes, so the commits being cut
# are still "unreleased" here and `--tag` names the version they belong to.
tag=v$version
# git-cliff resolves the repository from the working directory; `--workdir`
# misses the commits of a linked worktree, which release rehearsals run from.
cd -- "$release_root"
source_sha=$(git rev-parse HEAD)
if (( ${#ignore[@]} )); then
    # git-cliff's --unreleased stops at the newest tag even when it is ignored,
    # so the notes name their range: from the newest non-beta tag, if any.
    for candidate in $(git tag --list 'v[0-9]*' --merged HEAD --sort=-v:refname); do
        [[ $candidate =~ ^v[1-9][0-9]{7}\.[1-9][0-9]*$ ]] || continue
        for beta in "$@"; do [[ $candidate == "$beta" ]] && continue 2; done
        range=("$candidate..HEAD")
        break
    done
fi

git-cliff --config cliff.toml --tag "$tag" ${ignore[@]+"${ignore[@]}"} --output "$out/CHANGELOG.md"

# The release body keeps the platform and daemon facts every release states,
# then lists what actually changed since the previous tag.
cat > "$out/RELEASE_NOTES.md" <<NOTES
GUI-only release. macOS 14.2+ universal signed/notarized DMG; x86_64 and ARM64 Linux .deb, .rpm, Arch packages and tarballs (glibc 2.39+); $windows. Requires an existing Herdr daemon. Source: $source_sha.

NOTES
if (( ${#ignore[@]} )); then
    git-cliff --config cliff.toml --tag "$tag" ${ignore[@]+"${ignore[@]}"} --strip header ${range[@]+"${range[@]}"} >> "$out/RELEASE_NOTES.md"
else
    git-cliff --config cliff.toml --tag "$tag" --unreleased --strip header >> "$out/RELEASE_NOTES.md"
fi

[[ -s $out/CHANGELOG.md && -s $out/RELEASE_NOTES.md ]] || fail 'Generated changelog is empty'
