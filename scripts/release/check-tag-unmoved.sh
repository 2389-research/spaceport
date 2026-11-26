#!/bin/bash
# ABOUTME: Refuses to continue when a release tag no longer names the commit a workflow built.
# ABOUTME: Reads the tag ref itself from GitHub, never a same-named branch, peeling annotated tags.
set -euo pipefail
if [[ $# != 3 ]]; then
    echo "usage: $0 OWNER/REPO TAG EXPECTED_SHA" >&2
    exit 2
fi
repository=$1 tag=$2 expected=$3
refs=$(git ls-remote "https://github.com/$repository" "refs/tags/$tag" "refs/tags/$tag^{}")
# An annotated tag lists its tag object first and the peeled commit (^{}) after it; keep the last.
current=$(awk -v tag="refs/tags/$tag" '$2 == tag || $2 == tag "^{}" { sha = $1 } END { print sha }' <<< "$refs")
if [[ "$current" != "$expected" ]]; then
    echo "::error::$tag now names ${current:-nothing}, not the built commit $expected; refusing to publish" >&2
    exit 1
fi
