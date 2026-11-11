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
  # Canonical DNS outages must not block CI. Never edit host sources or keys.
  apt_temp=$(mktemp -d /tmp/spaceport-apt.XXXXXX)
  trap 'rm -rf "$apt_temp"' EXIT
  mkdir "$apt_temp/parts"
  python3 "$(dirname "$0")/ubuntu-ci-sources.py" \
    --architecture "$(dpkg --print-architecture)" \
    /etc/apt/sources.list.d/ubuntu.sources > "$apt_temp/ubuntu.sources"
  # APT's _apt sandbox must be able to read this public configuration.
  chmod 755 "$apt_temp" "$apt_temp/parts"
  chmod 644 "$apt_temp/ubuntu.sources"
  apt_sources=(-o "Dir::Etc::SourceList=$apt_temp/ubuntu.sources" \
    -o "Dir::Etc::SourceParts=$apt_temp/parts")
  echo 'Using temporary Ubuntu CI mirrors with the runner Ubuntu keyring.'
fi
"${sudo[@]+"${sudo[@]}"}" apt-get "${apt_sources[@]+"${apt_sources[@]}"}" update
"${sudo[@]+"${sudo[@]}"}" apt-get "${apt_sources[@]+"${apt_sources[@]}"}" install --no-install-recommends -y \
  build-essential pkg-config libxkbcommon-dev libxkbcommon-x11-dev libxcb1-dev \
  libfreetype6-dev libfontconfig1-dev fonts-dejavu-core libasound2-dev \
  python3 jq zstd
