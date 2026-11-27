"""ABOUTME: Check the release channel table names each channel's release and downloads.
ABOUTME: One calendar version, one tag shape and one https host serve both signed channels.
"""

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

SCRIPTS = Path(__file__).resolve().parents[1]
VERSION = "20261126.1"
INSTALLER = "Spaceport-{version}-arm64-apple-darwin.dmg"
SPEC = importlib.util.spec_from_file_location("channels", SCRIPTS / "channels.py")
channels = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(channels)
SPARKLE = json.loads((SCRIPTS / "sparkle.json").read_text())


class ChannelTableTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory(prefix="spaceport-channels-")
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.reset()

    def reset(self):
        self.table = json.loads(channels.CHANNELS.read_text())

    def load(self):
        path = self.root / "channels.json"
        path.write_text(json.dumps(self.table))
        return channels.load(path)

    def refusal(self, message):
        return self.assertRaisesRegex(ValueError, message)

    def test_table_names_the_app_channels_and_their_feeds(self):
        table = channels.load()
        self.assertEqual(sorted(table), sorted(SPARKLE["feeds"]))
        self.assertEqual(sorted(table), sorted(self.table["channels"]))
        for name, channel in table.items():
            self.assertEqual(channel.feed_release, SPARKLE["feeds"][name]["release"])
            self.assertEqual(channel.feed_url, SPARKLE["feeds"][name]["url"])
            self.assertTrue(channel.feed_url.startswith(f"{channel.base_url}/releases/download/"))

    def test_stable_channel_publishes_a_release_and_previews_stay_prereleases(self):
        preview, stable = channels.load()["macos-preview"], channels.load()["production"]
        self.assertTrue(preview.prerelease)
        self.assertFalse(stable.prerelease)
        self.assertEqual(stable.release_title(VERSION), f"Spaceport {VERSION}")
        self.assertEqual(preview.release_title(VERSION), f"Spaceport {VERSION} (macOS preview)")
        self.assertEqual(stable.release_url(VERSION), f"{stable.base_url}/releases/tag/v{VERSION}")
        # GitHub names no prerelease latest, so only the stable channel has a latest page.
        self.assertEqual(stable.download_page_url, f"{stable.base_url}/releases/latest")
        self.assertEqual(preview.download_page_url, f"{preview.base_url}/releases")

    def test_both_channels_publish_the_signer_artifact_at_its_version_tag(self):
        asset = INSTALLER.format(version=VERSION)
        for name, channel in channels.load().items():
            with self.subTest(channel=name):
                download = channel.appcast_download
                self.assertEqual(download.asset(VERSION), asset)
                self.assertEqual(channel.download_url(VERSION),
                                 "https://github.com/2389-research/spaceport/releases/"
                                 f"download/v{VERSION}/{asset}")
                self.assertEqual((download.platform, download.architecture), ("macos", "arm64"))
                self.assertEqual(download.minimum_system_version, "14.2")

    def test_artifact_name_matches_the_signing_script_and_kind_follows_the_channel(self):
        signer = (SCRIPTS / "sign-macos.sh").read_text()
        self.assertIn("/Spaceport-$1-$architecture-apple-darwin.dmg", signer)
        self.assertIn('kind: "signed-macos-preview"', signer)
        table = channels.load()
        self.assertEqual(table["macos-preview"].appcast_download.kind, "signed-macos-preview")
        # No route writes the stable artifact yet, so its kind is the contract one must.
        self.assertEqual(table["production"].appcast_download.kind, "signed-macos-production")

    def test_only_the_preview_route_publishes_a_feed_marker_as_a_prerelease(self):
        publication = (SCRIPTS / "publish-sparkle-feed.py").read_text()
        self.assertIn('CHANNEL = "macos-preview"', publication)
        self.assertIn("--prerelease", publication)
        self.assertIn("feed.CONFIG.read_text()", publication)

    def test_a_version_needs_its_calendar_shape_before_it_can_be_tagged(self):
        self.assertEqual(channels.release_tag(VERSION), f"v{VERSION}")
        for value in ("v20261126.1", "20261126.0", "20261126.01", "2026112.1", ""):
            with self.subTest(version=value), self.refusal("YYYYMMDD"):
                channels.release_tag(value)
        self.assertTrue(channels.is_release_tag(f"v{VERSION}"))
        for tag in ("macos-stable-feed", "v20261126.0", "20261126.1", "v"):
            with self.subTest(tag=tag):
                self.assertFalse(channels.is_release_tag(tag))

    def test_unknown_channels_are_refused_by_name(self):
        for name in ("macos-stable", "production-feed", ""):
            with self.subTest(name=name), self.refusal("macos-preview, production"):
                channels.channel(name)

    def test_table_and_base_url_must_be_one_https_repository(self):
        for url in ("http://github.com/2389-research/spaceport",
                    "https://elsewhere.test/2389-research/spaceport",
                    "https://github.com/2389-research/spaceport/",
                    "https://user@github.com/2389-research/spaceport",
                    "https://github.com/2389-research/spaceport?x=1"):
            with self.subTest(url=url):
                self.table["url"] = url
                with self.refusal("url"):
                    self.load()
        self.reset()
        self.table["repository"] = "2389-research"
        with self.refusal("repository"):
            self.load()
        self.reset()
        del self.table["channels"]
        with self.refusal("repository, url and channels"):
            self.load()

    def test_downloads_must_be_served_beside_the_feed(self):
        self.table["url"] = "https://downloads.test/2389-research/spaceport"
        with self.refusal("url must serve the channel feeds"):
            self.load()
        self.reset()
        sparkle = json.loads((SCRIPTS / "sparkle.json").read_text())
        for feed in sparkle["feeds"].values():
            feed["url"] = feed["url"].replace("/2389-research/spaceport/", "/attacker/spaceport/")
        elsewhere = self.root / "sparkle.json"
        elsewhere.write_text(json.dumps(sparkle))
        with self.refusal("release asset of the channel repository"):
            channels.load(channels.CHANNELS, elsewhere)

    def test_channels_must_match_the_app_feed_table(self):
        for change in ("rename", "drop"):
            with self.subTest(change=change):
                self.reset()
                channel = self.table["channels"].pop("production")
                if change == "rename":
                    self.table["channels"]["stable"] = channel
                with self.refusal("app's channels"):
                    self.load()

    def test_release_and_appcast_text_must_name_the_version_once(self):
        self.table["channels"]["production"]["release"]["title"] = "Spaceport"
        with self.refusal("version"):
            self.load()
        self.reset()
        self.table["channels"]["macos-preview"]["appcast"]["title"] = "Spaceport {tag}"
        with self.refusal("version"):
            self.load()
        self.reset()
        self.table["channels"]["production"]["release"]["prerelease"] = "no"
        with self.refusal("bool"):
            self.load()

    def test_downloads_must_name_one_artifact_per_platform_target(self):
        production = json.loads(channels.CHANNELS.read_text())["channels"]["production"]["downloads"]
        for downloads in (production * 2,
                          [dict(production[0], platform="MacOS")],
                          [dict(production[0], name="Spaceport.dmg")],
                          [dict(production[0], name="../Spaceport-{version}.dmg")],
                          [dict(production[0], format="zip")],
                          [dict(production[0], label="")],
                          []):
            with self.subTest(downloads=downloads):
                self.reset()
                self.table["channels"]["production"]["downloads"] = downloads
                with self.refusal("download|Download"):
                    self.load()

    def test_unreadable_oversized_and_malformed_tables_are_refused(self):
        big = self.root / "big.json"
        big.write_bytes(b"{" + b" " * channels.MAX_FILE)
        truncated = self.root / "truncated.json"
        truncated.write_text('{"repository": ')
        for path, message in ((self.root / "missing.json", "unreadable"),
                              (big, "size bound"),
                              (truncated, "malformed")):
            with self.subTest(path=path.name), self.refusal(message):
                channels.load(path)


if __name__ == "__main__":
    unittest.main()
