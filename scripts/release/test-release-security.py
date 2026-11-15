#!/usr/bin/env python3
"""Offline CI workflow and dependency-target contract tests."""

from pathlib import Path
import re
import sys
import tomllib
import unittest

ROOT = Path(__file__).resolve().parents[2]
sys.dont_write_bytecode = True


class ReleaseTargets(unittest.TestCase):
    def test_ci_platform_checks_keep_required_gate(self):
        workflow = (ROOT / ".github/workflows/ci.yml").read_text()
        sections = re.split(r"^  ([a-z-]+):\n", workflow.split("\njobs:\n", 1)[1], flags=re.M)
        jobs = dict(zip(sections[1::2], sections[2::2]))
        for name, job in jobs.items():
            self.assertIn("github.repository == 'sheprd-labs/sheprd'", job)
            self.assertIn("github.repository == '2389-research/spaceport'", job)
            if name != "build-macos":
                condition = " ".join(re.search(r"^    if: >-\n((?:      .*\n)+)", job, re.M)[1].split())
                mapping = " ".join("""
                  ((github.repository == '2389-research/spaceport' &&
                    github.actor == 'harperreed' && github.triggering_actor == 'harperreed') ||
                   (github.repository != '2389-research/spaceport' &&
                    github.actor == 'penso' && github.triggering_actor == 'penso'))
                """.split())
                self.assertIn(mapping, condition, name)
            self.assertNotIn("secrets.", job, name)
        # Keeps every main cache inside the repository's 10 GB cache budget.
        self.assertIn("\nenv:\n  CARGO_PROFILE_DEV_DEBUG: '0'\n\njobs:\n", workflow)
        self.assertIn("    needs: [checks, windows, windows-native, windows-package, build-macos, commits]\n",
                      jobs["checks-passed"])
        self.assertIn('test "$COMMITS" = success || exit 1', jobs["checks-passed"])
        self.assertIn('test "$RESULT" = success || exit 1', jobs["checks-passed"])
        self.assertIn('test "$CODE" = false && test "$RESULT" = skipped || exit 1', jobs["checks-passed"])
        for name in ("checks", "windows", "build", "build-macos"):
            self.assertIn("needs.commits.outputs.code == 'true'", jobs[name])
            self.assertIn("needs: [zizmor, commits]", jobs[name])
        self.assertIn("WINDOWS: ${{ needs.windows.result }}", jobs["checks-passed"])
        self.assertIn("          fetch-depth: 0\n", jobs["commits"])
        self.assertIn("python3 scripts/release/check-commit-messages.py range", jobs["commits"])
        self.assertIn("runner: [macos-15, ubuntu-24.04, ubuntu-24.04-arm]\n", jobs["checks"])
        self.assertIn("    name: Format, lint, and test\n", jobs["checks-passed"])
        self.assertIn("always()", jobs["checks-passed"])
        self.assertIn('test "$RESULT" = success', jobs["checks-passed"])
        self.assertIn("github.ref == 'refs/heads/main'", jobs["build"])
        self.assertNotIn("github.event_name == 'pull_request'", jobs["build"])
        self.assertNotIn("apple-darwin", jobs["build"])
        # The macOS release builds hold no runner on main; pull requests keep
        # their release-mode CLI suite, and the gate requires it.
        macos = jobs["build-macos"]
        self.assertIn("github.event_name == 'pull_request'", macos)
        self.assertNotIn("refs/heads/main", macos)
        self.assertIn("target: [aarch64-apple-darwin, x86_64-apple-darwin]\n", macos)
        for command in ('cargo build --locked --release -p sheprd-gpui --target "$TARGET"',
                        'cargo test --locked --release -p sheprd-gpui --test cli --target "$TARGET"'):
            self.assertIn(command, macos)
            self.assertIn(command, jobs["build"])
        self.assertIn("MACOS_RELEASE: ${{ needs.build-macos.result }}", jobs["checks-passed"])
        self.assertIn('test "$MACOS_RELEASE" = success', jobs["checks-passed"])
        for name in ("checks", "build"):
            self.assertIn("bash scripts/install-linux-deps.sh", jobs[name])
        self.assertNotIn("nfpm", workflow)
        # The required gate includes retained packaging and removed-feature
        # absence checks.
        self.assertIn("python3 -m unittest discover -s scripts/release/tests -v", jobs["checks"])
        windows = jobs["windows"]
        self.assertIn("        runner: [windows-2025, windows-11-arm]\n", windows)
        self.assertIn("    runs-on: ${{ matrix.runner }}\n", windows)
        # Each platform splits into a default-features and an all-features
        # leg that run side by side; every check runs in exactly one of them.
        legs = {"cargo fmt --all -- --check": "default",
                "cargo clippy --locked --workspace --all-targets --all-features -- -D warnings": "all",
                "cargo clippy --locked --workspace --all-targets --no-default-features -- -D warnings": "default",
                "cargo test --locked --workspace": "default",
                "cargo test --locked --workspace --all-features": "all"}
        for name in ("checks", "windows"):
            self.assertIn("        features: [default, all]\n", jobs[name])
            self.assertIn("${{ matrix.features }})\n", re.search(r"^    name: .+\n", jobs[name], re.M)[0])
            self.assertIn("${{ matrix.features }}\n          save-if:", jobs[name])
            for command, leg in legs.items():
                self.assertIn(f"        if: matrix.features == '{leg}'\n        run: {command}\n", jobs[name])
        for command in ("python3 -m unittest discover -s scripts/release/tests -v",
                        "bash scripts/check-file-size.sh", "python3 scripts/check-sorted.py",
                        "python3 scripts/release/test-release-security.py"):
            self.assertEqual(jobs["checks"].count(command), 1, command)
        self.assertNotIn("--release", windows)
        self.assertNotIn("cargo build", windows)
        # The installer check runs only for its inputs, and the gate requires
        # it then, and its absence otherwise.
        package = jobs["windows-package"]
        self.assertIn("needs.commits.outputs.windows_package == 'true'", package)
        self.assertIn("needs: [zizmor, commits]", package)
        self.assertNotIn("secrets.", package)
        # Bash, so "$TARGET" and friends expand on the Windows runner.
        self.assertIn("    defaults:\n      run:\n        shell: bash\n", package)
        for command in ('cargo build --locked --release -p sheprd-gpui -p sheprd-launcher -p sheprd-acp --target "$TARGET"',
                        'python scripts/release/package-windows.py "$VERSION" "$TARGET"',
                        'python scripts/release/windows-installer.py install-iscc',
                        'python scripts/release/windows-installer.py build "$VERSION" "$TARGET"',
                        'python scripts/release/windows-installer.py smoke'):
            self.assertIn(command, package)
        self.assertIn("windows_package: ${{ steps.changes.outputs.windows_package }}", jobs["commits"])
        # Native Windows GUI tests run whenever code changes, against the
        # pinned daemon only, and the gate requires them as it does windows.
        native = jobs["windows-native"]
        self.assertIn("needs.commits.outputs.code == 'true'", native)
        self.assertIn("needs: [zizmor, commits]", native)
        self.assertIn("        runner: [windows-2025, windows-11-arm]\n", native)
        self.assertNotIn("secrets.", native)
        for command in ('python scripts/install-herdr-windows.py "$RUNNER_TEMP/herdr"',
                        "cargo test --locked -p sheprd-gpui --features integration-test --test live_gui -- --ignored",
                        "cargo test --locked --release -p sheprd-gpui --features integration-test --test performance_gui dense_terminal_sidebar_performance -- --ignored --nocapture --exact"):
            self.assertIn(command, native)
        self.assertIn("          HERDR_PERF_P95_MS: '100'\n", native)
        self.assertIn("NATIVE: ${{ needs.windows-native.result }}", jobs["checks-passed"])
        self.assertIn('test "$WINDOWS" = success && test "$NATIVE" = success', jobs["checks-passed"])
        self.assertIn('test "$WINDOWS" = skipped && test "$NATIVE" = skipped', jobs["checks-passed"])
        self.assertIn("PACKAGE: ${{ needs.windows-package.result }}", jobs["checks-passed"])
        self.assertIn('test "$PACKAGE" = success', jobs["checks-passed"])
        self.assertIn('test "$PACKAGE_INPUTS" = false && test "$PACKAGE" = skipped', jobs["checks-passed"])

    def test_dependency_checks_cover_every_shipped_target(self):
        targets = {"aarch64-apple-darwin", "x86_64-apple-darwin",
                   "x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu",
                   "x86_64-pc-windows-msvc", "aarch64-pc-windows-msvc"}
        self.assertEqual(set(tomllib.loads((ROOT / "deny.toml").read_text())["graph"]["targets"]), targets)
        self.assertEqual(set(tomllib.loads((ROOT / "scripts/release/about.toml").read_text())["targets"]), targets)

    def test_no_workflow_reaches_azure_signing(self):
        for workflow in (ROOT / ".github/workflows").glob("*.yml"):
            self.assertNotIn("azure/", workflow.read_text(), workflow.name)


if __name__ == "__main__":
    unittest.main()
