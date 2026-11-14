"""Tests for scripts/check-i18n.py."""

import importlib.util
import sys
import unittest
from pathlib import Path

# Loading the hyphenated script as a module would leave a __pycache__ beside it.
sys.dont_write_bytecode = True
SPEC = importlib.util.spec_from_file_location(
    "check_i18n", Path(__file__).with_name("check-i18n.py")
)
assert SPEC and SPEC.loader
check_i18n = importlib.util.module_from_spec(SPEC)
# Dataclasses look their module up by name.
sys.modules["check_i18n"] = check_i18n
SPEC.loader.exec_module(check_i18n)

EN = Path("crates/app/locales/en-US/menu.ftl")


def messages(text: str, path: Path = EN):
    return check_i18n.messages(path, text)


class MessagesTests(unittest.TestCase):
    def test_ids_start_a_line_and_values_name_other_messages(self) -> None:
        ftl = messages(
            "# menu-comment = Not a message, nor { menu-example } a use.\n"
            "\n"
            "menu-quit = Quit\n"
            "menu-tabs = { $count ->\n"
            "    [one] One tab\n"
            "   *[other] { $count } tabs = many\n"
            "}\n"
            "menu-long =\n"
            "    First line\n"
            "\n"
            "    second = line\n"
            "    .title = { menu-quit } and { menu-tabs.title }\n"
            'menu-brace = { "{" } "{ menu-long }"\n'
            "-brand = Herdr\n"
        )
        self.assertEqual(
            ftl.messages, {"menu-quit": 3, "menu-tabs": 4, "menu-long": 8, "menu-brace": 13}
        )
        self.assertEqual(ftl.references, {"menu-quit", "menu-tabs", "menu-long"})
        self.assertEqual((ftl.language, ftl.feature), ("en-US", "menu"))


class CheckTests(unittest.TestCase):
    def files(self, *texts: tuple[Path, str]):
        return [messages(text, path) for path, text in texts]

    def test_examples_in_comments_are_not_calls(self) -> None:
        files = self.files((EN, "menu-quit = Quit\n"))
        sources = {Path("a.rs"): '/// `fl!("id")`\nfl!("menu-quit")'}
        self.assertEqual(check_i18n.check(files, sources), [])

    def test_used_messages_pass(self) -> None:
        files = self.files((EN, "menu-quit = Quit\nmenu-all = All { menu-quit }\n"))
        sources = {Path("a.rs"): 'fl!("menu-all")'}
        self.assertEqual(check_i18n.check(files, sources), [])

    def test_a_table_literal_counts_as_a_use(self) -> None:
        files = self.files((EN, "menu-quit = Quit\n"))
        sources = {Path("a.rs"): 'message: "menu-quit",'}
        self.assertEqual(check_i18n.check(files, sources), [])

    def test_a_message_only_tests_name_is_unused(self) -> None:
        files = self.files((EN, "menu-quit = Quit\nmenu-open = Open\n"))
        sources = {
            Path("crates/app/src/menu.rs"): 'fl!("menu-quit")',
            Path("crates/app/src/menu/tests.rs"): 'fl!("menu-open")',
            Path("crates/app/src/menu/tests/keys.rs"): '"menu-open"',
            Path("crates/app/src/layout_tests/rows.rs"): '"menu-open"',
            Path("crates/app/src/interaction_tests.rs"): '"menu-open"',
            Path("crates/app/tests/cli.rs"): '"menu-open"',
        }
        self.assertEqual(
            check_i18n.check(files, sources), [f"{EN}:2: menu-open is not used"]
        )

    def test_an_unused_message_is_reported_and_translations_are_not(self) -> None:
        french = Path("crates/app/locales/fr/menu.ftl")
        files = self.files((EN, "menu-quit = Quit\n"), (french, "menu-quit = Quitter\n"))
        sources = {Path("a.rs"): 'fl!("menu-open")'}
        self.assertEqual(
            check_i18n.check(files, sources), [f"{EN}:1: menu-quit is not used"]
        )


class MigratedTests(unittest.TestCase):
    ROOT = Path("crates/app/src/menu.rs")

    def errors(self, sources: dict[Path, str]) -> list[str]:
        errors = []
        for path in sorted(check_i18n.migrated(sources)):
            errors.extend(check_i18n.shown_english(path, sources[path]))
        return errors

    def test_unmarked_modules_are_not_checked(self) -> None:
        self.assertEqual(self.errors({self.ROOT: 'label("Quit Herdr")'}), [])

    def test_a_marked_module_and_its_directory_reject_english(self) -> None:
        sources = {
            self.ROOT: '// i18n: migrated\nlabel("Quit Herdr");\nfl!("menu-quit")',
            Path("crates/app/src/menu/row.rs"): 'let a = "open the file";\nlet b = "x-y";',
            Path("crates/app/src/menu/tests/row.rs"): 'label("Quit")',
            Path("crates/app/src/other.rs"): 'label("Quit")',
        }
        self.assertEqual(
            self.errors(sources),
            [
                'crates/app/src/menu/row.rs:1: "open the file" is not a message',
                'crates/app/src/menu.rs:2: "Quit Herdr" is not a message',
            ],
        )

    def test_logs_attributes_and_data_are_allowed(self) -> None:
        source = (
            "// i18n: migrated\n"
            '#[error("Could not read {path}")]\n'
            'tracing::warn!(\n    "Unable to open {}",\n    path,\n);\n'
            'write!(f, "Could not save")?;\n'
            'value.expect("Present in tests");\n'
            '.key_context("SettingsWindow")\n'
            'let font = "SF Mono"; // i18n: data\n'
            "// i18n: data\n"
            'let theme = "Catppuccin Mocha";\n'
            "// Comments say \"Quit Herdr\".\n"
            "let c = '\"'; let lifetime: &'static str = \"ok\";\n"
            'let shown = format!("Open {name}");\n'
        )
        self.assertEqual(
            self.errors({self.ROOT: source}),
            ['crates/app/src/menu.rs:15: "Open {name}" is not a message'],
        )

    def test_placeholders_and_raw_strings(self) -> None:
        source = '// i18n: migrated\nformat!("{Name} {count}");\nlet r = r#"Raw text"#;\n'
        self.assertEqual(
            self.errors({self.ROOT: source}),
            ['crates/app/src/menu.rs:3: "Raw text" is not a message'],
        )

    def test_unicode_names(self) -> None:
        source = '// i18n: migrated\nlet café = 1;\nnaïve::größe!("Open file");\n'
        self.assertEqual(
            self.errors({self.ROOT: source}),
            ['crates/app/src/menu.rs:3: "Open file" is not a message'],
        )


class PlatformTests(unittest.TestCase):
    FILES = [
        messages("language-name = English\n", Path("crates/app/locales/en-US/language.ftl")),
        messages("language-name = עברית\n", Path("crates/app/locales/he/language.ftl")),
    ]
    PLIST = {"CFBundleDevelopmentRegion": "en-US", "CFBundleLocalizations": ["he", "en-US"]}

    def test_files_naming_the_apps_languages_pass(self) -> None:
        translations = [
            Path("assets/macos/he.lproj"),
            Path("scripts/release/installer/en-US.isl"),
            Path("scripts/release/installer/he.isl"),
        ]
        desktop = "[Desktop Entry]\nComment=Client\nComment[he]=Klient\n"
        self.assertEqual(
            check_i18n.platform(self.FILES, self.PLIST, translations, desktop), []
        )

    def test_a_language_missing_or_extra_is_reported(self) -> None:
        plist = {"CFBundleDevelopmentRegion": "en", "CFBundleLocalizations": ["en-US"]}
        translations = [Path("scripts/release/installer/en-US.isl"), Path("scripts/release/installer/de.isl")]
        desktop = "Comment=Client\nComment[pt_BR]=Cliente\n"
        unnamed = messages("menu-quit = Quit\n", Path("crates/app/locales/fr/menu.ftl"))
        errors = check_i18n.platform([*self.FILES, unnamed], plist, translations, desktop)
        plist_path, desktop_path = check_i18n.INFO_PLIST, check_i18n.DESKTOP_ENTRY
        self.assertEqual(
            errors,
            [
                "fr does not name itself: add language-name to its language.ftl",
                f"{plist_path}: CFBundleDevelopmentRegion must be en-US",
                f"{plist_path}: CFBundleLocalizations lists ['en-US'], the app has ['en-US', 'fr', 'he']",
                "scripts/release/installer/de.isl: de is not one of the app's languages",
                "scripts/release/installer/fr.isl is missing: the installer has no fr text",
                "scripts/release/installer/he.isl is missing: the installer has no he text",
                f"{desktop_path}: [pt_BR] is not one of the app's languages",
                f"{desktop_path}: no Comment[fr] for fr",
                f"{desktop_path}: no Comment[he] for he",
            ],
        )

    def test_english_needs_the_plain_comment(self) -> None:
        english = self.FILES[:1]
        plist = {"CFBundleDevelopmentRegion": "en-US", "CFBundleLocalizations": ["en-US"]}
        installer = [Path("scripts/release/installer/en-US.isl")]
        self.assertEqual(check_i18n.platform(english, plist, installer, "Comment=Client\n"), [])
        errors = check_i18n.platform(english, plist, installer, "Name=Herdr\n")
        self.assertEqual(len(errors), 1)
        self.assertIn("no Comment[en_US] for en-US", errors[0])

if __name__ == "__main__":
    unittest.main()
