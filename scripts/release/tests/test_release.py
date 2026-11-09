"""Run: python3 -m unittest discover -s scripts/release/tests -v"""
import json
import os
from pathlib import Path
import subprocess
import shutil
import tarfile
import tempfile
import unittest
import zipfile

ROOT = Path(__file__).resolve().parents[3]
SCRIPTS = ROOT / "scripts/release"


def build_identity(worktree=False):
    return (b"not executable\0SPACEPORT_PREVIEW_V1\nid=ai.2389.spaceport\nversion=20260920.3\nprofile=release\n\0HERDR_BUILD_IDENTITY_V1\nworktree="
            + (b"1\nbranch=feature/test\npr=42\n\0" if worktree else b"0\nbranch=\npr=\n\0"))


class ReleaseTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="herdr-release-test-")
        self.addCleanup(self.temp.cleanup)
        self.work = Path(self.temp.name)
        self.env = {"PATH": os.environ["PATH"], "HOME": str(self.work)}
        self.notices = self.work / "third party notices.txt"
        self.notices.write_text("Third-party license text\nCopyright example\n")

    def run_script(self, name, *args, success=True):
        result = subprocess.run(
            ["bash", str(getattr(self, "scripts", SCRIPTS) / name), *map(str, args)],
            env=self.env, capture_output=True, text=True, timeout=30,
        )
        if success:
            self.assertEqual(result.returncode, 0, result.stderr)
        else:
            self.assertNotEqual(result.returncode, 0)
        return result

    def test_cask(self):
        first = self.run_script("render-cask.sh", "20260920.3", "AB" * 32).stdout
        self.assertEqual(first, self.run_script("render-cask.sh", "20260920.3", "ab" * 32).stdout)
        self.assertIn('version "20260920.3"', first)
        self.assertIn('sha256 "' + "ab" * 32 + '"', first)
        self.assertIn("sheprd-labs/sheprd/releases/download/v#{version}/Sheprd-#{version}-universal-apple-darwin.dmg", first)
        # Symbol form, not the deprecated string comparison. Cask `depends_on
        # macos:` always parses with comparator ">=", so this still means
        # "Sonoma or newer" and does not pin an exact release.
        self.assertIn("depends_on macos: :sonoma", first)
        for version in ["v20260920.3", "1.2", "020260920.3", "20260920.3-rc1", "20260920.3+build", "20260920.3\n", "$(id)"]:
            self.run_script("render-cask.sh", version, "ab" * 32, success=False)
        for sha in ["abc", "g" * 64, "a" * 65, "a" * 63 + "\n"]:
            self.run_script("render-cask.sh", "20260920.3", sha, success=False)

    def test_linux_archive(self):
        binary = self.work / "input binary"
        binary.write_bytes(build_identity())
        acp = self.work / "sheprd-acp"
        acp.write_bytes(b"chat program")
        for target in ("x86_64-unknown-linux-gnu", "aarch64-unknown-linux-gnu"):
            with self.subTest(target=target):
                result = self.run_script("package-linux.sh", "20260920.3", target, binary, self.work, self.notices)
                with tarfile.open(result.stdout.strip()) as archive:
                    base = f"Sheprd-20260920.3-{target}/"
                    files = {m.name: m for m in archive.getmembers() if m.isfile()}
                    self.assertEqual(set(files), {base + p for p in [
                        "bin/spaceport", "bin/sheprd-acp",
                        "share/applications/org.sheprd.Sheprd.desktop",
                        "share/icons/hicolor/scalable/apps/sheprd.svg",
                        "share/licenses/herdr-gpui/LICENSE-APACHE", "share/licenses/herdr-gpui/NOTICE.md",
                        "share/licenses/herdr-gpui/LICENSE", "share/licenses/herdr-gpui/NOTICE",
                        "share/licenses/herdr-gpui/LICENSE-octicons",
                        "share/licenses/herdr-gpui/THIRD-PARTY-NOTICES.txt",
                        "share/licenses/herdr-gpui/SOUND-NOTICE.md",
                    ]})
                    self.assertEqual(files[base + "bin/sheprd-acp"].mode & 0o777, 0o755)
                    self.assertEqual(archive.extractfile(base + "bin/sheprd-acp").read(), acp.read_bytes())
                    self.assertEqual(archive.extractfile(base + "share/icons/hicolor/scalable/apps/sheprd.svg").read(), (ROOT / "assets/icons/sheprd-linux.svg").read_bytes())
                    self.assertEqual(archive.extractfile(base + "share/licenses/herdr-gpui/THIRD-PARTY-NOTICES.txt").read(), self.notices.read_bytes())
                    for source in ("LICENSE", "NOTICE", "assets/icons/LICENSE-octicons", "crates/sheprd-protocol/NOTICE.md", "crates/sheprd-gpui/SOUND-NOTICE.md"):
                        self.assertEqual(archive.extractfile(base + "share/licenses/herdr-gpui/" + Path(source).name).read(), (ROOT / source).read_bytes())
                self.run_script("package-linux.sh", "20260920.3", target, binary, self.work, self.notices, success=False)
        self.run_script("package-linux.sh", "20260920.3", "bad-target", binary, self.work, self.notices, success=False)
        acp.unlink()
        result = self.run_script("package-linux.sh", "20260920.3", "x86_64-unknown-linux-gnu", binary, self.work,
                                 self.notices, success=False)
        self.assertIn("A sheprd-acp beside the binary required", result.stderr)
        acp.write_bytes(b"chat program")

    def run_windows(self, *args, success=True):
        result = subprocess.run(["python3", str(SCRIPTS / "package-windows.py"), *map(str, args)],
                                env=self.env, capture_output=True, text=True, timeout=30)
        self.assertEqual(result.returncode == 0, success, result.stderr)
        return result

    def test_windows_archive(self):
        binary = self.work / "input binary.exe"
        binary.write_bytes(build_identity())
        launcher = self.work / "input launcher.exe"
        launcher.write_bytes(b"windowed launcher")
        acp = self.work / "sheprd-acp.exe"
        acp.write_bytes(b"chat program")
        target = "x86_64-pc-windows-msvc"
        output = Path(self.run_windows("20260920.3", target, binary, launcher, self.work, self.notices).stdout.strip())
        with zipfile.ZipFile(output) as archive:
            executables = {Path(name).name for name in archive.namelist() if name.endswith(".exe")}
            self.assertEqual(executables, {"spaceport.exe", "spaceport-launcher.exe", "sheprd-acp.exe"})
        self.assertEqual(output, self.work.resolve() / f"Sheprd-20260920.3-{target}.zip")
        base = f"Sheprd-20260920.3-{target}/"
        with zipfile.ZipFile(output) as archive:
            self.assertIsNone(archive.testzip())
            self.assertEqual(set(archive.namelist()), {base + p for p in [
                "spaceport.exe", "spaceport-launcher.exe", "sheprd-acp.exe",
                "licenses/LICENSE", "licenses/NOTICE", "licenses/LICENSE-octicons",
                "licenses/LICENSE-APACHE", "licenses/NOTICE.md", "licenses/SOUND-NOTICE.md",
                "licenses/THIRD-PARTY-NOTICES.txt",
            ]})
            self.assertEqual({info.date_time for info in archive.infolist()}, {(1980, 1, 1, 0, 0, 0)})
            self.assertEqual(archive.read(base + "spaceport.exe"), binary.read_bytes())
            self.assertEqual(archive.read(base + "spaceport-launcher.exe"), launcher.read_bytes())
            self.assertEqual(archive.read(base + "sheprd-acp.exe"), acp.read_bytes())
            self.assertEqual(archive.read(base + "licenses/THIRD-PARTY-NOTICES.txt"), self.notices.read_bytes())
            for source in ("LICENSE", "NOTICE", "assets/icons/LICENSE-octicons", "crates/sheprd-protocol/LICENSE-APACHE",
                           "crates/sheprd-protocol/NOTICE.md", "crates/sheprd-gpui/SOUND-NOTICE.md"):
                self.assertEqual(archive.read(base + "licenses/" + Path(source).name), (ROOT / source).read_bytes())
        before = output.read_bytes()
        result = self.run_windows("20260920.3", target, binary, launcher, self.work, self.notices, success=False)
        self.assertIn("Output already exists", result.stderr)
        self.assertEqual(output.read_bytes(), before)
        output.unlink()
        arm = Path(self.run_windows("20260920.3", "aarch64-pc-windows-msvc", binary, launcher, self.work,
                                    self.notices).stdout.strip())
        self.assertEqual(arm, self.work.resolve() / "Sheprd-20260920.3-aarch64-pc-windows-msvc.zip")
        arm.unlink()
        result = self.run_windows("20260920.3", target, binary, self.work / "missing-launcher.exe", self.work,
                                  self.notices, success=False)
        self.assertIn("Launcher file required", result.stderr)
        acp.unlink()
        result = self.run_windows("20260920.3", target, binary, launcher, self.work, self.notices, success=False)
        self.assertIn("A sheprd-acp.exe beside the binary required", result.stderr)
        acp.write_bytes(b"chat program")
        for args in [("20260920.03", target, binary, launcher, self.work, self.notices),
                     ("v20260920.3", target, binary, launcher, self.work, self.notices),
                     ("20260920.3", "x86_64-pc-windows-gnu", binary, launcher, self.work, self.notices),
                     ("20260920.3", target, self.work / "missing.exe", launcher, self.work, self.notices),
                     ("20260920.3", target, binary, self.work, self.work, self.notices),
                     ("20260920.3", target, binary, launcher, self.work / "missing", self.notices),
                     ("20260920.3", target, binary, launcher, self.work, self.work / "missing"),
                     ("20260920.3", target, binary, launcher, self.work, self.work / "empty")]:
            (self.work / "empty").touch()
            with self.subTest(args=args):
                self.run_windows(*args, success=False)
        self.assertEqual([p.name for p in self.work.iterdir() if p.suffix == ".zip" or p.name.startswith(".herdr")], [])

    def test_packaging_requires_notices(self):
        self.mock_tools()
        binary = self.work / "binary"
        binary.touch()
        for script, inputs in [("package-linux.sh", ["x86_64-unknown-linux-gnu", binary]),
                               ("package-macos.sh", [binary, binary])]:
            self.run_script(script, "20260920.3", *inputs, self.work, success=False)
            for notices in [self.work / "missing", binary]:
                result = self.run_script(script, "20260920.3", *inputs, self.work, notices, success=False)
                self.assertIn("Nonempty third-party notices", result.stderr)
        self.assertFalse((self.work / "Spaceport.app").exists())
        self.assertFalse(list(self.work.glob("*.tar.gz")))

    def mock_tools(self):
        tools = self.work / "tools"
        tools.mkdir()
        mock = tools / "mock.py"
        mock.write_bytes((SCRIPTS / "tests/mock-tool.py").read_bytes())
        mock.chmod(0o755)
        for tool in ["uname", "security", "openssl", "base64", "plutil", "lipo", "ditto", "xcrun", "hdiutil", "codesign", "spctl"]:
            (tools / tool).symlink_to(mock)
        self.env.update(PATH=str(tools) + os.pathsep + self.env["PATH"], MOCK_LOG=str(self.work / "log"))

    def test_macos_requires_darwin(self):
        self.mock_tools()
        self.env["MOCK_OS"] = "Linux"
        for script, args in [
            ("package-macos.sh", ["20260920.3", "arm", "intel", self.work, self.notices]),
            ("sign-macos.sh", ["20260920.3", "app", self.work]),
        ]:
            result = self.run_script(script, *args, success=False)
            self.assertIn("macOS required", result.stderr)
        self.assertFalse((self.work / "log").exists())

    def macos_builds(self, arm_worktree=False, intel_worktree=False):
        """Each architecture's build folder, as the build leaves it:
        herdr-gpui and sheprd-acp beside it, in a folder
        the mock lipo reads the architecture from."""
        binaries = []
        for arch, worktree in (("arm64", arm_worktree), ("x86_64", intel_worktree)):
            folder = self.work / arch
            folder.mkdir(exist_ok=True)
            (folder / "herdr-gpui").write_bytes(build_identity(worktree))
            (folder / "sheprd-acp").write_bytes(b"chat program")
            binaries.append(folder / "herdr-gpui")
        return binaries

    def test_unsigned_assembly(self):
        self.mock_tools()
        arm, intel = self.macos_builds()
        self.run_script("package-macos.sh", "20260920.3", arm, intel, self.work, self.notices)
        app = self.work / "Spaceport.app"
        self.assertEqual({str(p.relative_to(app)) for p in app.rglob("*") if p.is_file()}, {
            "Contents/MacOS/spaceport", "Contents/MacOS/sheprd-acp",
            "Contents/Info.plist", "Contents/Resources/Sheprd.icns",
            "Contents/Resources/Assets.car",
            "Contents/Resources/LICENSE-APACHE", "Contents/Resources/NOTICE.md",
            "Contents/Resources/LICENSE", "Contents/Resources/NOTICE", "Contents/Resources/LICENSE-octicons",
            "Contents/Resources/THIRD-PARTY-NOTICES.txt",
            "Contents/Resources/SOUND-NOTICE.md",
        })
        self.assertEqual((app / "Contents/Resources/THIRD-PARTY-NOTICES.txt").read_bytes(), self.notices.read_bytes())
        self.assertFalse((app / "Contents/MacOS/Herdr").exists())
        self.assertFalse((app / "Contents/MacOS/spaceport").is_symlink())
        self.assertEqual((app / "Contents/Resources/Sheprd.icns").read_bytes(), (ROOT / "assets/icons/Sheprd.icns").read_bytes())
        self.assertEqual((app / "Contents/Resources/Assets.car").read_bytes(), (ROOT / "assets/icons/Sheprd.car").read_bytes())
        for source in ("LICENSE", "NOTICE", "assets/icons/LICENSE-octicons", "crates/sheprd-protocol/NOTICE.md", "crates/sheprd-gpui/SOUND-NOTICE.md"):
            self.assertEqual((app / "Contents/Resources" / Path(source).name).read_bytes(), (ROOT / source).read_bytes())
        self.run_script("package-macos.sh", "20260920.3", arm, intel, self.work, self.notices, success=False)

    def test_assembly_requires_each_companion_beside_each_binary(self):
        self.mock_tools()
        for companion in ("sheprd-acp",):
            with self.subTest(companion=companion):
                arm, intel = self.macos_builds()
                (intel.parent / companion).unlink()
                result = self.run_script("package-macos.sh", "20260920.3", arm, intel, self.work, self.notices,
                                         success=False)
                self.assertIn(f"A {companion} beside each binary required", result.stderr)
                self.assertFalse((self.work / "Spaceport.app").exists())

    def test_worktree_icons_and_mismatched_architectures(self):
        self.mock_tools()
        # Distinct fixture artwork proves selection without generating real icons.
        fixture = self.work / "packaging checkout"
        for path in ["scripts/release/common.sh", "scripts/release/package-macos.sh",
                     "scripts/release/package-linux.sh", "scripts/release/build-icon.py",
                     "scripts/release/preview-identity.py", "crates/sheprd-gpui/macos-preview-app-id",
                     "scripts/release/org.sheprd.Sheprd.desktop", "assets/macos/Info.plist",
                      "LICENSE", "NOTICE", "assets/icons/LICENSE-octicons",
                      "crates/sheprd-gpui/SOUND-NOTICE.md",
                     "crates/sheprd-protocol/LICENSE-APACHE", "crates/sheprd-protocol/NOTICE.md"]:
            destination = fixture / path
            destination.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(ROOT / path, destination)
        for name in ["Sheprd-worktree.icns", "Sheprd-worktree.car", "sheprd-linux-worktree.svg"]:
            (fixture / "assets/icons" / name).write_bytes(b"red fixture " + name.encode())
        self.scripts = fixture / "scripts/release"
        arm, intel = self.macos_builds(arm_worktree=True)
        result = self.run_script("package-macos.sh", "20260920.3", arm, intel, self.work, self.notices, success=False)
        self.assertIn("different build identities", result.stderr)
        intel.write_bytes(build_identity(True))
        self.run_script("package-macos.sh", "20260920.3", arm, intel, self.work, self.notices)
        self.assertEqual((self.work / "Spaceport.app/Contents/Resources/Sheprd.icns").read_bytes(), (fixture / "assets/icons/Sheprd-worktree.icns").read_bytes())
        self.assertEqual((self.work / "Spaceport.app/Contents/Resources/Assets.car").read_bytes(), (fixture / "assets/icons/Sheprd-worktree.car").read_bytes())
        result = self.run_script("package-linux.sh", "20260920.3", "aarch64-unknown-linux-gnu", arm, self.work, self.notices)
        with tarfile.open(result.stdout.strip()) as archive:
            self.assertEqual(archive.extractfile("Sheprd-20260920.3-aarch64-unknown-linux-gnu/share/icons/hicolor/scalable/apps/sheprd.svg").read(), (fixture / "assets/icons/sheprd-linux-worktree.svg").read_bytes())

    def test_missing_malformed_conflicting_identity_fails_closed(self):
        binary = self.work / "binary"
        for data in [b"old binary", build_identity() + build_identity(True),
                     build_identity().replace(b"pr=\n", b"pr=0\n"),
                     build_identity().replace(b"worktree=0", b"worktree=9")]:
            binary.write_bytes(data)
            self.run_script("package-linux.sh", "20260920.3", "x86_64-unknown-linux-gnu", binary, self.work, self.notices, success=False)
            self.assertFalse(list(self.work.glob("*.tar.gz")))

    # Signing state, cleanup and no-resubmit contracts live in test_macos_preview.py.


if __name__ == "__main__":
    unittest.main()
