#!/usr/bin/env bash
# Ubuntu 24.04, native x86_64 or ARM64. GPUI 0.2.2 defaults enable X11 and Wayland.
set -euo pipefail
# A developer and a CI runner need sudo; root in a container has none installed.
if [[ $EUID -eq 0 ]]; then sudo=(); else sudo=(sudo); fi
apt_sources=()
if [[ ${SPACEPORT_CI_APT_MIRROR:-0} == 1 ]]; then
  if [[ ${GITHUB_ACTIONS:-} != true ]]; then
    echo 'Temporary Ubuntu mirrors require GitHub Actions.' >&2
    exit 1
  fi
  # Try two archives for metadata failures. Never edit host sources or keys.
  apt_temp=$(mktemp -d /tmp/spaceport-apt.XXXXXX)
  trap 'rm -rf "$apt_temp"' EXIT
  mkdir "$apt_temp/parts"
  # APT's _apt sandbox must be able to read this public configuration.
  chmod 755 "$apt_temp" "$apt_temp/parts"
  apt_sources=(-o "Dir::Etc::SourceList=$apt_temp/ubuntu.sources" \
    -o "Dir::Etc::SourceParts=$apt_temp/parts")
  architecture=$(dpkg --print-architecture)
  for mirror in primary fallback; do
    python3 "$(dirname "$0")/ubuntu-ci-sources.py" \
      --architecture "$architecture" --mirror "$mirror" \
      /etc/apt/sources.list.d/ubuntu.sources > "$apt_temp/ubuntu.sources"
    chmod 644 "$apt_temp/ubuntu.sources"
    # Logs outlive the temporary source list, including on failure.
    apt_log=$(mktemp "${RUNNER_TEMP:-/tmp}/spaceport-apt-${mirror}.XXXXXX")
    echo "Refreshing Ubuntu $mirror metadata with the runner Ubuntu keyring; log: $apt_log"
    if "${sudo[@]+"${sudo[@]}"}" apt-get "${apt_sources[@]}" \
      -o Acquire::Retries=0 -o Acquire::http::Timeout=20 -o Acquire::https::Timeout=20 \
      update --error-on=any > "$apt_log" 2>&1; then
      cat "$apt_log"
      break
    else
      apt_status=$?
      echo "Ubuntu $mirror metadata refresh failed (exit $apt_status); full log: $apt_log" >&2
      tail -n 20 "$apt_log" >&2
      if [[ $mirror == fallback ]]; then exit "$apt_status"; fi
      echo 'Trying the independent Ubuntu fallback archive before installing packages.' >&2
    fi
  done
else
  "${sudo[@]+"${sudo[@]}"}" apt-get update
fi
# Installation is never retried: only a successful metadata refresh reaches it.
"${sudo[@]+"${sudo[@]}"}" apt-get "${apt_sources[@]+"${apt_sources[@]}"}" install --no-install-recommends -y \
  build-essential pkg-config libxkbcommon-dev libxkbcommon-x11-dev libxcb1-dev \
  libfreetype6-dev libfontconfig1-dev fonts-dejavu-core libasound2-dev \
  python3 jq zstd
