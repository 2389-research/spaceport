# ABOUTME: Guard retained desktop Fluent labels against the old product name.
# ABOUTME: Keep release, provider, and legacy-route text outside this copy cut.

from pathlib import Path
import re
import unittest


ROOT = Path(__file__).resolve().parents[3]
ENGLISH = ROOT / "crates/sheprd-gpui/locales/en-US"
LABELS = {
    "commands.ftl": ("commands-about",),
    "config-diagnostic.ftl": (
        "config-diagnostic-icon-font-homebrew",
        "config-diagnostic-icon-font-download",
    ),
    "error-clipboard.ftl": ("error-clipboard-macos-missing",),
    "error-config.ftl": ("error-config-unknown-keys",),
    "error-control.ftl": (
        "error-control-no-response",
        "error-control-pipe-foreign",
        "error-control-socket-in-use",
        "error-control-unavailable",
    ),
    "menus.ftl": (
        "menus-herdr",
        "menus-about-herdr",
        "menus-hide-herdr",
        "menus-quit-herdr",
    ),
    "settings.ftl": (
        "settings-source-herdr-ui",
        "settings-general-language-description",
        "settings-general-launch-at-login-description",
        "settings-notif-delivery-description",
        "settings-config-local",
        "settings-config-local-description",
    ),
    "settings-notifications.ftl": (
        "settings-notifications-delivery-herdr",
        "settings-notifications-sound-description-no-window",
    ),
    "window-notices.ftl": ("window-notices-diagnostic-app",),
}
ENTRY = re.compile(r"^([a-z][a-z0-9-]+)\s*=\s*(.*)$", re.MULTILINE)


class DesktopProductCopy(unittest.TestCase):
    def test_retained_desktop_labels_name_spaceport(self):
        for filename, labels in LABELS.items():
            messages = dict(ENTRY.findall((ENGLISH / filename).read_text()))
            for label in labels:
                with self.subTest(label=label):
                    self.assertIn(label, messages)
                    self.assertIn("Spaceport", messages[label])
                    self.assertNotIn("Sheprd", messages[label])
