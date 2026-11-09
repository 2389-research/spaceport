# ABOUTME: Checks that excluded host-resource dashboards have no shipped implementation.
# ABOUTME: Retained process inspection, ports and activity history remain separate concerns.
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[3]
GPUI = ROOT / 'crates/sheprd-gpui'


class ResourceDashboardAbsence(unittest.TestCase):
    def test_sampler_is_removed_after_extracting_worker_stop(self):
        self.assertFalse((GPUI / 'src/system_load.rs').exists())
        self.assertFalse((GPUI / 'src/system_load').exists())
        for file in ['listening_ports.rs', 'network_quality.rs']:
            self.assertTrue('worker_stop::Stop' in (GPUI / 'src' / file).read_text(), file)

    def test_shipped_rust_has_no_dashboard_symbols(self):
        for source in (GPUI / 'src').rglob('*.rs'):
            # Rejection fixtures must still name the excluded settings and wire forms.
            if 'tests' in source.parts or source.name == 'tests.rs':
                continue
            text = source.read_text()
            for symbol in ['system_load', 'show_gpu_load', 'show_sidebar_gpu_load',
                           'HostOp::System', 'ControlCommand::System', 'LoadInfo']:
                self.assertFalse(symbol in text, f"{symbol}: {source.relative_to(ROOT)}")

    def test_removed_command_has_no_family_help(self):
        help_text = (GPUI / "src/control/client/help.rs").read_text()
        self.assertFalse('"system" =>' in help_text)

    def test_devices_timeline_and_navigation_are_retained(self):
        for file in ['chart.rs', 'history.rs', 'table.rs']:
            self.assertTrue((GPUI / 'src/devices_overview' / file).exists())
        table = (GPUI / 'src/devices_overview/table.rs').read_text()
        self.assertIn('navigate_endpoint', table)
        self.assertIn('NavigationTarget::Pane', table)
        self.assertIn('page.expanded', table)
        self.assertNotIn('fn meter(', table)


if __name__ == '__main__':
    unittest.main()
