#!/usr/bin/env python3
# ABOUTME: Exercises CI mirror fallback with real signed Ubuntu APT metadata.
# ABOUTME: Runs only inside an explicitly selected disposable Ubuntu container.

import argparse
import hashlib
import os
from pathlib import Path
import re
import subprocess
import sys


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Run real APT success, network fallback, and failure scenarios in a disposable Ubuntu container."
    )
    parser.add_argument("artifacts", type=Path, help="directory for full scenario and APT logs")
    args = parser.parse_args()
    if not Path("/.dockerenv").exists() or os.environ.get("SPACEPORT_APT_CONTAINER_TEST") != "1":
        parser.error("requires an explicitly selected disposable Docker container (SPACEPORT_APT_CONTAINER_TEST=1)")
    artifacts = args.artifacts.resolve()
    artifacts.mkdir(parents=True, exist_ok=True)
    architecture = subprocess.check_output(["dpkg", "--print-architecture"], text=True).strip()
    print(f"Real APT container architecture: {architecture}", flush=True)
    sources = Path("/etc/apt/sources.list.d/ubuntu.sources")
    keyring = Path("/usr/share/keyrings/ubuntu-archive-keyring.gpg")
    original = {file: hashlib.sha256(file.read_bytes()).digest() for file in (sources, keyring)}
    # An unreachable proxy injects a real connection failure; successful requests
    # still reach the public Ubuntu repositories and APT verifies their signatures.
    proxy = Path("/etc/apt/apt.conf.d/99spaceport-test-proxy")
    if proxy.exists():
        parser.error(f"refusing to replace existing fixture configuration: {proxy}")
    scenarios = (
        ("primary", "", 1, 1, True),
        ("fallback", '\n'.join(
            f'Acquire::https::Proxy::{host} "http://127.0.0.1:9";'
            for host in ("archive.ubuntu.com", "ports.ubuntu.com")
        ) + '\n', 2, 1, True),
        ("unreachable", 'Acquire::https::Proxy "http://127.0.0.1:9";\n', 2, 0, False),
    )
    try:
        for name, config, updates, installs, success in scenarios:
            proxy.write_text(config)
            environment = os.environ | {
                "GITHUB_ACTIONS": "true", "SPACEPORT_CI_APT_MIRROR": "1", "RUNNER_TEMP": str(artifacts),
            }
            log = artifacts / f"{name}.log"
            with log.open("w") as output:
                result = subprocess.run(
                    ["/bin/bash", "-x", str(Path(__file__).with_name("install-linux-deps.sh"))],
                    env=environment, stdout=output, stderr=subprocess.STDOUT, check=False, timeout=600,
                )
            trace = log.read_text()
            actual_updates = len(re.findall(r"^\+ apt-get .* update --error-on=any$", trace, re.M))
            actual_installs = len(re.findall(r"^\+ apt-get .* install --no-install-recommends", trace, re.M))
            if (result.returncode == 0, actual_updates, actual_installs) != (success, updates, installs):
                raise AssertionError(f"{name}: exit={result.returncode}, updates={actual_updates}, installs={actual_installs}; see {log}")
            for file, digest in original.items():
                if hashlib.sha256(file.read_bytes()).digest() != digest:
                    raise AssertionError(f"{name}: source or keyring changed: {file}")
            if name != "primary" and "metadata refresh failed" not in trace:
                raise AssertionError(f"{name}: missing failed update diagnostic; see {log}")
            print(f"{name}: passed ({updates} metadata attempts, {installs} installs); log: {log}")
    finally:
        proxy.unlink(missing_ok=True)
    return 0


if __name__ == "__main__":
    sys.exit(main())
