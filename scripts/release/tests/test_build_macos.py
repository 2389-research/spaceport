"""Local build orchestration only: never compile or sign real artifacts."""
import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

from sparkle_fixture import sdk_fixture


SCRIPTS = Path(__file__).resolve().parents[1]
SECRETS = (
    "SPARKLE_PRIVATE_KEY",
    "MACOS_CERTIFICATE_P12_BASE64", "MACOS_CERTIFICATE_PASSWORD",
    "MACOS_SIGNING_IDENTITY", "APPLE_API_PRIVATE_KEY",
    "APPLE_API_KEY_ID", "APPLE_API_ISSUER_ID", "APPLE_API_KEY_PATH",
)


class BuildMacosTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="herdr local build ")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        scripts = self.root / "scripts/release"
        scripts.mkdir(parents=True)
        for name in ("build-macos.sh", "common.sh"):
            shutil.copyfile(SCRIPTS / name, scripts / name)
        sdk_fixture(scripts, self.root / "sdk")
        (scripts / "package-macos.sh").write_text(
            'set -eu\n[[ ! -e sourced && $# == 6 && -s $5 ]]\n'
            'printf "package\\n" >> calls\nmkdir "$4/Spaceport.app"\n'
            '[[ ${FAIL_PACKAGE:-0} == 0 ]]\n'
        )
        (scripts / "generate-notices.py").write_text(
            'import os, pathlib, sys\n'
            f'assert not any(name in os.environ for name in {SECRETS!r})\n'
            'assert not pathlib.Path("sourced").exists()\n'
            'if os.environ.get("FAIL_NOTICES"): sys.exit(1)\n'
            'pathlib.Path(sys.argv[1]).write_text("test notices")\n'
        )
        tools = self.root / "tools"
        tools.mkdir()
        for name, body in {
            "uname": 'if [[ $1 == -m ]]; then printf "arm64\\n"; else printf "Darwin\\n"; fi',
            "lipo": 'printf "arm64 x86_64\\n"',
            "cargo": '\n'.join([
                'set -eu',
                *(f'[[ ${{{name}+present}} != present ]]' for name in SECRETS),
                '[[ $MACOSX_DEPLOYMENT_TARGET == 14.2 ]]',
                '[[ $CARGO_PROFILE_RELEASE_BUILD_OVERRIDE_STRIP == none ]]',
                '[[ $SPACEPORT_MACOS_PREVIEW_VERSION == 20260920.3 ]]',
                '[[ ! -e sourced ]]',
                'printf "%s\\n" "$*" >> calls',
                '[[ ${FAIL_BUILD:-0} == 0 ]]',
            ]),
        }.items():
            tool = tools / name
            tool.write_text("#!/bin/bash\n" + body + "\n")
            tool.chmod(0o755)
        self.env = {"PATH": str(tools) + os.pathsep + os.environ["PATH"],
                    "HOME": str(self.root),
                    "SPARKLE_FRAMEWORK_PATH": str(self.root / "sdk"),
                    **dict.fromkeys(SECRETS, "dummy")}

    def config(self):
        scripts = self.root / "scripts/release"
        (scripts / "smoke-cli.py").write_text('import sys\n')
        (scripts / "sign-macos.sh").write_text(
            'set -eu\n[[ $1 == 20260920.3 && $2 == "$3/Spaceport.app" && $4 == universal && $5 == installed ]]\n'
            'printf "sign\\n" >> calls\nprintf "sign diagnostic\\n"\n'
            '[[ ${MISSING_DMG:-0} == 0 ]] || exit 0\n'
            'printf "mock dmg" > "$3/Spaceport-$1-universal-apple-darwin.dmg"\n'
            '[[ ${FAIL_SIGN:-0} == 0 ]] || exit 1\n'
            'if [[ ${LATE_OUTPUT:-0} == 1 ]]; then\n'
            'mkdir target/distribution/20260920.3-release-universal\n'
            'printf preserve > target/distribution/20260920.3-release-universal/existing\nfi\n'
        )

    def run_build(self, version="20260920.3", success=False):
        result = subprocess.run(
            ["/bin/bash", str(self.root / "scripts/release/build-macos.sh"), version, "release", "universal", "installed"],
            env=self.env, capture_output=True, text=True, timeout=10,
        )
        self.assertEqual(result.returncode == 0, success, result.stderr)
        if success:
            output = self.root / "target/distribution/20260920.3-release-universal"
            dmg = output / "Spaceport-20260920.3-universal-apple-darwin.dmg"
            self.assertEqual(result.stdout, str(dmg) + "\n")
            self.assertEqual(dmg.read_text(), "mock dmg")
            self.assertTrue((output / "Spaceport.app").is_dir())
            self.assertTrue((output / "THIRD-PARTY-NOTICES.txt").is_file())
            self.assertIn("sign diagnostic", result.stderr)
        self.assertFalse(list((self.root / "target/distribution").glob(".spaceport-build.*")))
        return result.stderr

    def test_version_validation_before_build(self):
        for version in ("v20260920.3", "020260920.3", "20260920", "2026092.3",
                        "20260920.03", "20260920.3-rc1", "$(id)"):
            self.assertIn("Version must", self.run_build(version))
        self.config()
        self.assertFalse((self.root / "calls").exists())
        self.assertFalse((self.root / "sourced").exists())

    def test_general_release_mode_stays_blocked(self):
        self.config()
        self.env['HERDR_RELEASE_VERSION'] = '20260920.3'
        self.assertIn('remains blocked', self.run_build())
        self.assertFalse((self.root / 'calls').exists())

    def test_build_retry_order_and_existing_output(self):
        self.config()
        self.env["FAIL_BUILD"] = "1"
        self.run_build()
        self.assertFalse((self.root / "target/distribution/20260920.3-release-universal").exists())
        self.assertFalse((self.root / "sourced").exists())
        self.env["FAIL_BUILD"] = "0"
        self.run_build(success=True)
        calls = (self.root / "calls").read_text().splitlines()
        # Each target's Rust build precedes packaging.
        self.assertEqual(calls[-4:], [
            "build --locked --features sheprd-gpui/macos-updater --release --target-dir target -p sheprd-gpui -p sheprd-acp",
            "build --locked --features sheprd-gpui/macos-updater --release --target-dir target -p sheprd-gpui -p sheprd-acp --target x86_64-apple-darwin",
            "package", "sign",
        ])
        self.assertIn("Output already exists", self.run_build())
        self.assertEqual((self.root / "calls").read_text().splitlines(), calls)

    def test_notice_failure_prevents_packaging_and_signing(self):
        self.config()
        self.env["FAIL_NOTICES"] = "1"
        self.run_build()
        calls = (self.root / "calls").read_text()
        self.assertNotIn("package", calls)
        self.assertNotIn("sign", calls)
        self.assertFalse((self.root / "sourced").exists())
        self.assertFalse((self.root / "target/distribution/20260920.3-release-universal").exists())
        del self.env["FAIL_NOTICES"]
        self.run_build(success=True)

    def test_assembly_failures_can_retry_without_partial_output(self):
        self.config()
        for failure in ("FAIL_PACKAGE", "FAIL_SIGN", "MISSING_DMG"):
            with self.subTest(failure=failure):
                self.env[failure] = "1"
                self.run_build()
                self.assertFalse((self.root / "target/distribution/20260920.3-release-universal").exists())
                del self.env[failure]
                (self.root / "sourced").unlink(missing_ok=True)
                self.run_build(success=True)
                shutil.rmtree(self.root / "target/distribution/20260920.3-release-universal")
                (self.root / "sourced").unlink(missing_ok=True)

    def test_output_created_during_signing_is_preserved(self):
        self.config()
        self.env["LATE_OUTPUT"] = "1"
        self.assertIn("Output already exists", self.run_build())
        output = self.root / "target/distribution/20260920.3-release-universal"
        self.assertEqual([p.name for p in output.iterdir()], ["existing"])
        self.assertEqual((output / "existing").read_text(), "preserve")

    def test_existing_symlink_is_preserved(self):
        self.config()
        output = self.root / "target/distribution/20260920.3-release-universal"
        output.parent.mkdir(parents=True)
        output.symlink_to(self.root / "missing")
        self.assertIn("Output already exists", self.run_build())
        self.assertTrue(output.is_symlink())
        self.assertFalse((self.root / "calls").exists())


if __name__ == "__main__":
    unittest.main()
