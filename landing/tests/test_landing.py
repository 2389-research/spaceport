"""ABOUTME: Checks the published landing page folder before GitHub Pages deploys it.
ABOUTME: Every local file the page names must exist, and paths must work under /spaceport/.
"""

import re
import unittest
from html.parser import HTMLParser
from pathlib import Path
from urllib.parse import urlparse

PUBLIC = Path(__file__).resolve().parent.parent / "public"
SITE_URL = "https://2389-research.github.io/spaceport/"
ALLOWED_HOSTS = {
    "2389-research.github.io",
    "api.github.com",
    "github.com",
    "herdr.dev",
}


class References(HTMLParser):
    """Collects every URL the page's tags point at, and its fragment targets."""

    def __init__(self):
        super().__init__()
        self.urls = []
        self.ids = set()

    def handle_starttag(self, tag, attrs):
        for name, value in attrs:
            if not value:
                continue
            if name == "id":
                self.ids.add(value)
            elif name in ("href", "src"):
                self.urls.append(value)
            elif name == "srcset":
                self.urls.extend(part.split()[0] for part in value.split(","))
            elif name == "content" and value.startswith("http"):
                self.urls.append(value)


def page_references():
    parser = References()
    parser.feed((PUBLIC / "index.html").read_text(encoding="utf-8"))
    return parser


class LandingPageTest(unittest.TestCase):
    def setUp(self):
        self.page = page_references()

    def test_local_references_exist(self):
        local = [u for u in self.page.urls if not urlparse(u).scheme and not u.startswith("#")]
        self.assertTrue(local)
        for url in local:
            self.assertTrue((PUBLIC / url).is_file(), f"index.html names a missing file: {url}")

    def test_paths_are_relative(self):
        # The page is served under /spaceport/, so a root path would miss it.
        for url in self.page.urls:
            self.assertFalse(url.startswith("/"), f"root-absolute path breaks under /spaceport/: {url}")

    def test_fragment_links_have_targets(self):
        for url in self.page.urls:
            if url.startswith("#"):
                self.assertIn(url[1:], self.page.ids, f"no element has id {url[1:]}")

    def test_own_site_urls_resolve_to_files(self):
        own = [u for u in self.page.urls if u.startswith(SITE_URL)]
        self.assertTrue(own)
        for url in own:
            path = url[len(SITE_URL):] or "index.html"
            self.assertTrue((PUBLIC / path).is_file(), f"{url} has no file behind it")

    def test_external_hosts_are_expected(self):
        sources = [(PUBLIC / "assets" / name).read_text(encoding="utf-8") for name in ("site.js", "site.css")]
        urls = self.page.urls + re.findall(r"https?://[^\s\"')]+", "\n".join(sources))
        for url in urls:
            host = urlparse(url).hostname
            if host:
                self.assertIn(host, ALLOWED_HOSTS, f"unexpected external host in {url}")

    def test_stylesheet_files_exist(self):
        css = (PUBLIC / "assets" / "site.css").read_text(encoding="utf-8")
        fonts = re.findall(r"url\(\"([^\"]+)\"\)", css)
        self.assertTrue(fonts)
        for font in fonts:
            self.assertTrue((PUBLIC / "assets" / font).is_file(), f"site.css names a missing file: {font}")


if __name__ == "__main__":
    unittest.main()
