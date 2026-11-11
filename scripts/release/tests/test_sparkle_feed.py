"""ABOUTME: Exercise signed Sparkle appcasts and archive trust with real Ed25519 keys.
ABOUTME: Reject tampered bytes, wrong keys, bad release URLs and version rollback.
"""

import base64
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
import xml.etree.ElementTree as ET

SCRIPT = Path(__file__).resolve().parents[1] / "sparkle-feed.py"
spec = importlib.util.spec_from_file_location("sparkle_feed", SCRIPT)
feed = importlib.util.module_from_spec(spec)
spec.loader.exec_module(feed)


class SparkleFeedTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="spaceport-feed-")
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.private = self.root / "fixture.key"
        self.run_crypto("genpkey", "-algorithm", "ED25519", "-out", self.private)
        public = self.run_crypto("pkey", "-in", self.private, "-pubout", "-outform", "DER")
        self.public = base64.b64encode(public[-32:]).decode()
        self.config = {"public_key": self.public}
        self.archive = self.root / "Spaceport-20261111.1-arm64-apple-darwin.dmg"
        self.archive.write_bytes(b"private fixture archive bytes")

    def run_crypto(self, *args):
        return subprocess.check_output([feed.OPENSSL, *map(str, args)],
                                       stderr=subprocess.DEVNULL, timeout=10)

    def sign(self, data):
        content = self.root / "content"
        content.write_bytes(data)
        return base64.b64encode(self.run_crypto(
            "pkeyutl", "-sign", "-inkey", self.private, "-rawin", "-in", content)).decode()

    def signed_feed(self, version="20261111.1", notes="Fix <terminal> & save state"):
        xml = feed.render(version, self.archive.stat().st_size,
                          self.sign(self.archive.read_bytes()), notes)
        signature = self.sign(xml)
        return xml + (f"<!-- sparkle-signatures:\nedSignature: {signature}\n"
                      f"length: {len(xml)}\n-->\n").encode()

    def test_real_signatures_and_escaped_notes_round_trip(self):
        item = feed.parse(self.signed_feed(), self.config)
        self.assertEqual(item["version"], "20261111.1")
        self.assertEqual(item["notes"], "Fix <terminal> & save state")
        self.assertEqual(item["url"],
                         "https://github.com/harperreed/spaceport/releases/download/"
                         "v20261111.1/Spaceport-20261111.1-arm64-apple-darwin.dmg")
        feed.verify_archive(item, self.archive, self.public)

    def test_release_notes_declare_markdown_format(self):
        xml = feed.render("20261111.1", 5, self.sign(b"hello"), "## Added\n- Rocket")
        description = ET.fromstring(xml).find("./channel/item/description")
        self.assertEqual(description.get(f"{{{feed.NS}}}format"), "markdown")

    def test_create_rejects_foreign_bundle_metadata_before_signing(self):
        metadata = {"app_id": "org.unrelated.app", "team_id": "HD9NM9NSMK",
                    "version": "20261111.1", "architecture": "arm64", "profile": "release",
                    "kind": "signed-macos-preview",
                    "sha256": hashlib.sha256(self.archive.read_bytes()).hexdigest()}
        self.archive.with_suffix(".dmg.json").write_text(json.dumps(metadata))
        config = self.root / "sparkle.json"
        config.write_text(json.dumps(self.config))
        notes = self.root / "notes.md"
        notes.write_text("## Added\nFixture")
        output = self.root / "appcast.xml"
        result = subprocess.run(["python3", str(SCRIPT), "--config", str(config), "create",
                                 "20261111.1", str(self.archive), str(notes), str(output),
                                 "--tool", str(self.root / "nonexistent-tool")],
                                capture_output=True, text=True, timeout=10)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("metadata mismatch", result.stderr)
        self.assertFalse(output.exists())

    def test_feed_and_archive_tampering_rejected(self):
        signed = self.signed_feed()
        with self.assertRaisesRegex(ValueError, "signature"):
            feed.parse(signed.replace(b"Fix", b"Mix"), self.config)
        item = feed.parse(signed, self.config)
        self.archive.write_bytes(b"private fixture archive bytez")
        with self.assertRaisesRegex(ValueError, "signature"):
            feed.verify_archive(item, self.archive, self.public)

    def test_wrong_public_key_rejected(self):
        with self.assertRaisesRegex(ValueError, "signature"):
            feed.parse(self.signed_feed(), {"public_key": base64.b64encode(bytes(32)).decode()})

    def test_unsigned_feed_and_unsigned_trailing_data_rejected(self):
        for data in (b"<rss/>", self.signed_feed() + b"<rss/>"):
            with self.subTest(data=data[-15:]), self.assertRaises(ValueError):
                feed.parse(data, self.config)

    def test_signature_length_must_cover_all_content(self):
        signed = self.signed_feed()
        signed = signed.replace(b"length: ", b"length: 1")
        with self.assertRaisesRegex(ValueError, "length"):
            feed.parse(signed, self.config)

    def test_signed_foreign_download_url_rejected(self):
        xml = feed.render("20261111.1", 5, self.sign(b"hello"), "note")
        xml = xml.replace(b"github.com/harperreed/spaceport", b"github.com/attacker/spaceport")
        signed = xml + (f"<!-- sparkle-signatures:\nedSignature: {self.sign(xml)}\n"
                        f"length: {len(xml)}\n-->\n").encode()
        with self.assertRaisesRegex(ValueError, "URL"):
            feed.parse(signed, self.config)

    def test_same_or_older_version_cannot_replace_feed(self):
        for old, new in (("20261111.2", "20261111.2"),
                         ("20261111.10", "20261111.9"),
                         ("20261111.1", "20261110.99")):
            with self.subTest(old=old, new=new), self.assertRaisesRegex(ValueError, "newer"):
                feed.require_newer(new, old)
        feed.require_newer("20261111.10", "20261111.9")

    def test_invalid_versions_and_signatures_rejected_before_render(self):
        for version in ("v20261111.1", "20261111.01", "20261111.0", "$(id)"):
            with self.subTest(version=version), self.assertRaises(ValueError):
                feed.render(version, 5, self.sign(b"hello"), "note")
        for size, signature in ((0, self.sign(b"hello")), (5, "bad")):
            with self.subTest(size=size), self.assertRaises(ValueError):
                feed.render("20261111.1", size, signature, "note")

    def test_wrong_archive_size_rejected(self):
        item = feed.parse(self.signed_feed(), self.config)
        self.archive.write_bytes(b"short")
        with self.assertRaisesRegex(ValueError, "length"):
            feed.verify_archive(item, self.archive, self.public)

    def test_publication_requires_newest_published_version_and_signed_previous_feed(self):
        new = self.signed_feed("20261111.2")
        old = self.signed_feed("20261111.1")
        self.assertEqual(feed.publication_version(new, old, self.config,
                         ["macos-preview-feed", "v20261111.1", "v20261111.2"]), "20261111.2")
        for previous, tags in ((old, ["v20261111.3", "v20261111.2"]),
                               (new, ["v20261111.2"]),
                               (b"unsigned", ["v20261111.2"]),
                               (None, ["v20261111.1"])):
            with self.subTest(tags=tags), self.assertRaises(ValueError):
                feed.publication_version(new, previous, self.config, tags)

    def test_official_sdk_signatures_verify_with_public_key(self):
        sdk = os.environ.get("SPARKLE_TEST_SDK")
        tool = ((Path(sdk) if sdk else Path(__file__).resolve().parents[3] / "target/sparkle-sdk")
                / "bin/sign_update")
        if not tool.is_file():
            self.assertIsNone(sdk, f"Explicit SDK has no sign_update tool: {tool}")
            self.skipTest("Official macOS SDK tool unavailable; pure crypto tests still run")
        seed = self.run_crypto("pkey", "-in", self.private, "-outform", "DER")[-32:]
        key = base64.b64encode(seed)
        signature = feed.sign(tool, self.archive, key)
        xml = self.root / "appcast.xml"
        xml.write_bytes(feed.render("20261111.1", self.archive.stat().st_size, signature, "fixture"))
        feed.sign(tool, xml, key)
        item = feed.parse(xml.read_bytes(), self.config)
        feed.verify_archive(item, self.archive, self.public)
        metadata = {"app_id": "ai.2389.spaceport", "team_id": "HD9NM9NSMK",
                    "version": "20261111.1", "architecture": "arm64", "profile": "release",
                    "kind": "signed-macos-preview",
                    "sha256": hashlib.sha256(self.archive.read_bytes()).hexdigest()}
        self.archive.with_suffix(".dmg.json").write_text(json.dumps(metadata))
        config = self.root / "sparkle.json"
        config.write_text(json.dumps(self.config))
        notes = self.root / "notes.md"
        notes.write_text("## Added\nFixture")
        output = self.root / "created.xml"
        result = subprocess.run(["python3", str(SCRIPT), "--config", str(config), "create",
                                 "20261111.1", str(self.archive), str(notes), str(output),
                                 "--tool", str(tool)],
                                env=dict(os.environ, SPARKLE_PRIVATE_KEY=key.decode()),
                                capture_output=True, text=True, timeout=15)
        self.assertEqual(result.returncode, 0, result.stderr)
        item = feed.parse(output.read_bytes(), self.config)
        feed.verify_archive(item, self.archive, self.public)


if __name__ == "__main__":
    unittest.main()
