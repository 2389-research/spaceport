"""ABOUTME: Guards removal of the desktop embedded browser engine.
ABOUTME: Leaves native terminal, chat, file, and system URL owners in place.
"""

from pathlib import Path
import unittest
import tomllib


ROOT = Path(__file__).resolve().parents[3]


class WebviewSourceAbsence(unittest.TestCase):
    def test_engine_and_native_page_fixtures_are_absent(self):
        browser = ROOT / "crates/sheprd-gpui/src/browser"
        for module in (
            "native", "page", "building", "page_keys", "parking", "stacking",
            "preview", "kit", "markdown", "favicon", "underlay",
        ):
            with self.subTest(module=module):
                self.assertFalse((browser / f"{module}.rs").exists())
        for asset in ("kit", "markdown.css", "favicon.js"):
            self.assertFalse((browser / asset).exists(), asset)
        for fixture in ("smoke_pages.rs", "smoke_pages_windows.rs", "smoke_page_fixture.rs"):
            self.assertFalse((browser.parent / fixture).exists(), fixture)

    def test_no_engine_dependency_or_live_native_hook_remains(self):
        for manifest in ("Cargo.toml", "crates/sheprd-gpui/Cargo.toml"):
            content = (ROOT / manifest).read_text()
            for package in ("lb-wry", "wry.workspace", "objc2-web-kit"):
                self.assertNotIn(package, content, manifest)
        for source in (ROOT / "crates/sheprd-gpui/src").rglob("*.rs"):
            content = source.read_text()
            for hook in ("wry::", "browser.pages", "drawn_pages", "around_pages", "bind_page_keys"):
                self.assertNotIn(hook, content, str(source.relative_to(ROOT)))

    def test_lockfile_has_no_embedded_browser_engine(self):
        packages = {p["name"] for p in tomllib.loads((ROOT / "Cargo.lock").read_text())["package"]}
        self.assertTrue({"lb-wry", "objc2-web-kit", "webview2-com", "webkit2gtk"}.isdisjoint(packages))

    def test_control_schema_has_no_browser_capability(self):
        schema = (ROOT / "crates/sheprd-control-protocol/src/protocol/tabs.rs").read_text()
        for variant in ("    Blank,", "    Web,", "    File,"):
            self.assertNotIn(variant, schema)
        status = (ROOT / "crates/sheprd-control-protocol/src/protocol/app.rs").read_text()
        self.assertNotIn("pub browser_tabs:", status)

    def test_retained_content_owners_exist(self):
        for owner in (
            "browser/store.rs", "browser/groups.rs", "core_terminals.rs",
            "group_terminals.rs", "chat/rich.rs", "chat/visual.rs",
            "window/file_links.rs", "listening_ports.rs", "links.rs",
        ):
            self.assertTrue((ROOT / "crates/sheprd-gpui/src" / owner).exists(), owner)


if __name__ == "__main__":
    unittest.main()
