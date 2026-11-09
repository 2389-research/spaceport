"""macOS development launch paths use Spaceport's independent bundle identity."""
import json
from pathlib import Path
import plistlib
import re
import subprocess
import sys
import unittest


ROOT = Path(__file__).resolve().parents[3]


class AppNameTests(unittest.TestCase):
    def test_development_process_name_uses_window_identity(self):
        constants = (ROOT / "crates/sheprd-gpui/src/constants.rs").read_text()
        self.assertIn('pub(crate) const WINDOW_TITLE: &str = "Spaceport"', constants)
        icon = (ROOT / "crates/sheprd-gpui/src/app_icon.rs").read_text()
        self.assertRegex(
            icon,
            r"setProcessName\s*\(\s*&NSString::from_str\s*\(\s*crate::constants::WINDOW_TITLE\s*\)\s*\)",
        )

    def test_development_bundle_uses_running_app_identity(self):
        constants = (ROOT / "crates/sheprd-gpui/src/constants.rs").read_text()
        app_id = (ROOT / "crates/sheprd-gpui/development-app-id").read_text().strip()
        self.assertEqual(app_id, "spaceport.development")
        self.assertIn('APP_ID: &str = env!("SPACEPORT_APP_ID")', constants)
        justfile = (ROOT / "justfile").read_text()
        self.assertIn('plutil -insert CFBundleIdentifier -string "$(cat crates/sheprd-gpui/development-app-id)"', justfile)
        selector = (ROOT / "crates/sheprd-gpui/preview_identity.rs").read_text()
        self.assertIn('include_str!("development-app-id")', selector)
        self.assertIn('include_str!("macos-preview-app-id")', selector)
        title = re.search(r'pub\(crate\) const WINDOW_TITLE: &str = "([^"]+)"', constants)
        self.assertIsNotNone(title)
        for key in ("CFBundleName", "CFBundleDisplayName"):
            self.assertIn(f'plutil -replace {key} -string {title[1]}', justfile)

    def test_bundle_plist_names_the_app_spaceport(self):
        info = plistlib.loads((ROOT / "assets/macos/Info.plist").read_bytes())
        for key in ("CFBundleName", "CFBundleDisplayName", "CFBundleExecutable"):
            self.assertEqual(info[key], "spaceport" if key == "CFBundleExecutable" else "Spaceport", key)

    @unittest.skipUnless(sys.platform == "darwin", "assetutil requires macOS")
    def test_asset_catalogs_contain_the_plist_icon_name(self):
        # A catalog compiled under another name silently falls back to the
        # flattened .icns, which macOS 26 and later render soft in the Dock.
        name = plistlib.loads((ROOT / "assets/macos/Info.plist").read_bytes())["CFBundleIconName"]
        for catalog in ("Sheprd.car", "Sheprd-worktree.car"):
            info = subprocess.run(
                ["xcrun", "assetutil", "--info", ROOT / "assets/icons" / catalog],
                check=True, capture_output=True,
            ).stdout
            # Older assetutil releases list only the flattened renditions that
            # macOS 14 and 15 read, not the layered Icon Composer stack.
            icons = {
                item.get("Name") for item in json.loads(info)
                if item.get("AssetType") in ("IconImageStack", "MultiSized Image", "Icon Image")
            }
            self.assertEqual(icons, {name}, catalog)

    def test_development_recipes_launch_the_bundled_executable(self):
        justfile = (ROOT / "justfile").read_text()
        for recipe, profile in (("run", "release"), ("run-debug", "debug")):
            body = re.search(rf"(?m)^{re.escape(recipe)} \*args:\n((?:[ \t]+.*\n|\n)+)", justfile)
            self.assertIsNotNone(body, recipe)
            self.assertNotIn(f"target/{profile}/herdr-gpui", body.group(1))
            if recipe == "run":
                # Opened through LaunchServices, so macOS asks for Local
                # Network access instead of denying it to a shell's child.
                self.assertIn('app="$PWD/target/release/Spaceport.app"', body.group(1))
                self.assertIn('"$app" --args {{args}}', body.group(1))
                self.assertRegex(body.group(1), r"\n\s*open -n -W ")
                self.assertIn("{{just_executable()}} bundle release\n", body.group(1))
                self.assertIn("--release -p sheprd-gpui -- {{args}}", body.group(1))
            else:
                launch = f"exec target/{profile}/Spaceport.app/Contents/MacOS/spaceport {{{{args}}}}"
                self.assertIn(launch, body.group(1))

    def test_bundle_recipe_uses_spaceport_with_retained_icon_resources(self):
        justfile = (ROOT / "justfile").read_text()
        body = re.search(r'(?m)^bundle profile="release" features="":\n((?:[ \t]+.*\n|\n)+)', justfile)
        self.assertIsNotNone(body)
        for expected in (
            'app=target/{{profile}}/Spaceport.app',
            'cargo build --locked $flags --target-dir target -p sheprd-gpui --features "{{features}}"',
            'cp target/{{profile}}/spaceport "$app/Contents/MacOS/spaceport"',
            'cp assets/macos/Info.plist "$app/Contents/Info.plist"',
            '"$app/Contents/Resources/Sheprd.icns"',
            '"$app/Contents/Resources/Assets.car"',
        ):
            self.assertIn(expected, body.group(1))
