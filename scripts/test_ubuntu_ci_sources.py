# ABOUTME: Tests CI Ubuntu source rewriting without changing APT trust settings.
# ABOUTME: Exercises the real CLI against isolated Deb822 source fixtures.

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).with_name("ubuntu-ci-sources.py")
SOURCE = """# Ubuntu runner sources
Types: deb
URIs: mirror+file:/etc/apt/apt-mirrors.txt
Suites: noble noble-updates noble-backports
Components: main restricted universe multiverse
Signed-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg

Types: deb
URIs: http://security.ubuntu.com/ubuntu/
Suites: noble-security
Components: main restricted universe multiverse
Signed-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg
"""


class UbuntuSourcesTests(unittest.TestCase):
    def rewrite(self, source=SOURCE, architecture="amd64"):
        with tempfile.TemporaryDirectory() as directory:
            original = Path(directory) / "ubuntu.sources"
            original.write_text(source)
            result = subprocess.run(
                [sys.executable, str(SCRIPT), "--architecture", architecture, str(original)],
                capture_output=True, text=True, check=False,
            )
            self.assertEqual(original.read_text(), source)
            return result

    def test_amd64_changes_only_uri_fields(self):
        result = self.rewrite()
        self.assertEqual(result.returncode, 0, result.stderr)
        expected = SOURCE.replace(
            "mirror+file:/etc/apt/apt-mirrors.txt", "https://mirrors.edge.kernel.org/ubuntu/"
        ).replace("http://security.ubuntu.com/ubuntu/", "https://mirrors.edge.kernel.org/ubuntu/")
        self.assertEqual(result.stdout, expected)

    def test_arm64_preserves_suites_components_keyring_and_comments(self):
        source = SOURCE.replace("mirror+file:/etc/apt/apt-mirrors.txt", "http://ports.ubuntu.com/ubuntu-ports")
        source = source.replace("http://security.ubuntu.com/ubuntu/", "http://ports.ubuntu.com/ubuntu-ports")
        result = self.rewrite(source, "arm64")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, source.replace(
            "http://ports.ubuntu.com/ubuntu-ports", "https://mirrors.mit.edu/ubuntu-ports/"
        ))

    def test_continuations_and_architecture_fields_stay_unchanged(self):
        source = SOURCE.replace("Components: main restricted universe multiverse", "Components: main restricted\n universe multiverse")
        source = source.replace("Types: deb", "Architectures: amd64 i386\nTypes: deb deb-src")
        source = source.replace("URIs: mirror+file:/etc/apt/apt-mirrors.txt", "URIs:\thttp://azure.archive.ubuntu.com/ubuntu/")
        result = self.rewrite(source)
        self.assertEqual(result.returncode, 0, result.stderr)
        for before, after in zip(source.splitlines(), result.stdout.splitlines(), strict=True):
            if not before.startswith("URIs:"):
                self.assertEqual(before, after)

    def test_ci_mirror_override_refuses_a_developer_shell(self):
        result = subprocess.run(
            ["/bin/bash", str(SCRIPT.with_name("install-linux-deps.sh"))],
            env={"SPACEPORT_CI_APT_MIRROR": "1", "GITHUB_ACTIONS": "false", "PATH": "/nonexistent"},
            capture_output=True, text=True, check=False,
        )
        self.assertEqual(result.returncode, 1)
        self.assertEqual(result.stderr, "Temporary Ubuntu mirrors require GitHub Actions.\n")

    def test_unknown_or_unsafe_sources_are_refused_without_partial_output(self):
        for source, architecture in [
            (SOURCE, "riscv64"),
            (SOURCE.replace("noble-updates", "jammy-updates"), "amd64"),
            (SOURCE.replace("mirror+file:/etc/apt/apt-mirrors.txt", "https://example.com/ubuntu/"), "amd64"),
            (SOURCE.replace("Signed-By: /usr/share/keyrings/ubuntu-archive-keyring.gpg\n", ""), "amd64"),
            (SOURCE.replace("ubuntu-archive-keyring.gpg", "other.gpg"), "amd64"),
            (SOURCE + "Trusted: yes\n", "amd64"),
            (SOURCE + "URIs: http://archive.ubuntu.com/ubuntu/\n", "amd64"),
            (SOURCE.replace("Types: deb", "malformed"), "amd64"),
            (SOURCE.replace("URIs:", "URIs:\n "), "amd64"),
            (SOURCE.replace("Types: deb", "Architectures: arm64\nTypes: deb"), "amd64"),
            ("", "amd64"),
        ]:
            with self.subTest(source=source, architecture=architecture):
                result = self.rewrite(source, architecture)
                self.assertNotEqual(result.returncode, 0)
                self.assertEqual(result.stdout, "")
                self.assertIn("Ubuntu CI sources:", result.stderr)


if __name__ == "__main__":
    unittest.main()
