//! An endpoint's link state for every combination of the fields it is read
//! from, as commands and the device picker word it. The words are the CLI's
//! output and the `endpoint-status` selector, so they are pinned here.
use super::*;
use crate::endpoint::link::Failure;
use spaceport_i18n::Locale;

const STATUSES: [ConnectionStatus; 6] = [
    ConnectionStatus::Connecting,
    ConnectionStatus::StartingDaemon,
    ConnectionStatus::AwaitingSnapshot,
    ConnectionStatus::Connected,
    ConnectionStatus::Disconnected,
    ConnectionStatus::Detached,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Refusal {
    None,
    Daemon,
    Client,
}

/// The fields an endpoint's link state is read from.
#[derive(Clone, Copy, Debug)]
struct Flags {
    enabled: bool,
    detached: bool,
    refusal: Refusal,
    missing: bool,
    outage: bool,
    status: ConnectionStatus,
    error: bool,
}

impl Flags {
    fn all() -> Vec<Self> {
        let mut all = Vec::new();
        for enabled in [false, true] {
            for detached in [false, true] {
                for refusal in [Refusal::None, Refusal::Daemon, Refusal::Client] {
                    for missing in [false, true] {
                        for outage in [false, true] {
                            for status in STATUSES {
                                for error in [false, true] {
                                    all.push(Self {
                                        enabled,
                                        detached,
                                        refusal,
                                        missing,
                                        outage,
                                        status,
                                        error,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        all
    }

    fn endpoint(self) -> Endpoint {
        let mut endpoint = Endpoint::new(
            "ssh:box".into(),
            "box".into(),
            ConnectTarget::Local,
            self.enabled,
        );
        endpoint.never_connect();
        endpoint.live.status = self.status;
        if self.error {
            endpoint
                .live
                .notices
                .show(crate::notice::Cause::Connection, "refused".into());
        }
        let reason = self.outage.then(|| "connection lost".into());
        let why = match self.refusal {
            Refusal::Daemon => Some(Failure::Version {
                mismatch: VersionMismatch::DaemonOutdated {
                    server_version: None,
                },
                reason,
            }),
            Refusal::Client => Some(Failure::Version {
                mismatch: VersionMismatch::ClientOutdated {
                    server_version: None,
                },
                reason,
            }),
            Refusal::None if self.missing => Some(Failure::Missing { reason }),
            Refusal::None if self.outage || (!self.status.is_connected() && self.error) => {
                Some(Failure::Transport {
                    reason: reason
                        .or_else(|| endpoint.live.notices.connection_reason().map(str::to_owned)),
                    needs_user: false,
                })
            }
            Refusal::None => None,
        };
        let retry_at = endpoint.retry_at();
        let link = if !self.enabled {
            Link::Disabled
        } else if self.detached {
            Link::Detached { retry_at, why }
        } else if self.status.is_connected() && why.is_none() {
            Link::Up {
                retry_at,
                since: None,
            }
        } else {
            Link::Waiting { retry_at, why }
        };
        endpoint.enter(Transition::Fixture(link));
        endpoint
    }
}

/// One row of the table: the fields it fixes (`None` is any value) and the
/// words for an endpoint they match.
struct Row {
    enabled: bool,
    detached: Option<bool>,
    refusal: Option<Refusal>,
    missing: Option<bool>,
    outage: Option<bool>,
    connected: Option<bool>,
    error: Option<bool>,
    /// The `endpoint-status` selector.
    state: &'static str,
    /// What commands print.
    command: &'static str,
    /// What the device picker shows, in English.
    label: &'static str,
}

impl Row {
    fn matches(&self, flags: Flags) -> bool {
        let fits = |want: Option<bool>, have: bool| want.is_none_or(|want| want == have);
        self.enabled == flags.enabled
            && fits(self.detached, flags.detached)
            && self.refusal.is_none_or(|want| want == flags.refusal)
            && fits(self.missing, flags.missing)
            && fits(self.outage, flags.outage)
            && fits(self.connected, flags.status.is_connected())
            && fits(self.error, flags.error)
    }
}

/// The rows are disjoint and cover every combination, so the table states
/// the precedence without leaning on row order.
#[rustfmt::skip]
const TABLE: &[Row] = &[
    Row { enabled: false, detached: None, refusal: None, missing: None, outage: None, connected: None, error: None,
          state: "disabled", command: "disabled", label: "disabled" },
    Row { enabled: true, detached: Some(true), refusal: None, missing: None, outage: None, connected: None, error: None,
          state: "detached", command: "detached", label: "detached" },
    Row { enabled: true, detached: Some(false), refusal: Some(Refusal::Daemon), missing: None, outage: None, connected: None, error: None,
          state: "daemon-outdated", command: "Herdr update needed", label: "Herdr update needed" },
    Row { enabled: true, detached: Some(false), refusal: Some(Refusal::Client), missing: None, outage: None, connected: None, error: None,
          state: "client-outdated", command: "app update needed", label: "app update needed" },
    // A missing Herdr says what fixes it, through retries and their outages,
    // until a connection succeeds.
    Row { enabled: true, detached: Some(false), refusal: Some(Refusal::None), missing: Some(true), outage: None, connected: None, error: None,
          state: "herdr-missing", command: "Herdr not installed", label: "Herdr not installed" },
    // An outage wins over a link that still looks connected: the handle can
    // stop before its disconnect state arrives.
    Row { enabled: true, detached: Some(false), refusal: Some(Refusal::None), missing: Some(false), outage: Some(true), connected: None, error: None,
          state: "reconnecting", command: "reconnecting", label: "reconnecting" },
    Row { enabled: true, detached: Some(false), refusal: Some(Refusal::None), missing: Some(false), outage: Some(false), connected: Some(true), error: None,
          state: "online", command: "online", label: "online" },
    Row { enabled: true, detached: Some(false), refusal: Some(Refusal::None), missing: Some(false), outage: Some(false), connected: Some(false), error: Some(true),
          state: "reconnecting", command: "reconnecting", label: "reconnecting" },
    Row { enabled: true, detached: Some(false), refusal: Some(Refusal::None), missing: Some(false), outage: Some(false), connected: Some(false), error: Some(false),
          state: "connecting", command: "connecting", label: "connecting" },
];

fn row(flags: Flags) -> &'static Row {
    let mut rows = TABLE.iter().filter(|row| row.matches(flags));
    let row = rows.next().unwrap_or_else(|| panic!("no row: {flags:?}"));
    assert!(rows.next().is_none(), "two rows: {flags:?}");
    row
}

#[test]
fn every_combination_of_link_fields_keeps_its_words() {
    crate::i18n::show(&Locale::FALLBACK);
    for flags in Flags::all() {
        let endpoint = flags.endpoint();
        let row = row(flags);
        assert_eq!(endpoint.status(), row.command, "{flags:?}");
        assert_eq!(endpoint.status_label(), row.label, "{flags:?}");
    }
}

/// The phase each `endpoint-status` selector names.
fn named(state: &str) -> Phase {
    match state {
        "disabled" => Phase::Disabled,
        "detached" => Phase::Detached,
        "daemon-outdated" => Phase::DaemonOutdated,
        "client-outdated" => Phase::ClientOutdated,
        "herdr-missing" => Phase::HerdrMissing,
        "reconnecting" => Phase::Reconnecting,
        "connecting" => Phase::Connecting,
        "online" => Phase::Online,
        other => panic!("{other}"),
    }
}

#[test]
fn every_combination_of_link_fields_has_one_phase() {
    for flags in Flags::all() {
        let mut endpoint = flags.endpoint();
        let row = row(flags);
        assert_eq!(endpoint.phase(), named(row.state), "{flags:?}");
        // The stored link remains authoritative over an older window projection.
        let mut other = flags;
        other.status = ConnectionStatus::Connecting;
        other.error = false;
        let stale = other.endpoint();
        endpoint.live = stale.live.clone();
        assert_eq!(endpoint.phase(), named(row.state), "{flags:?}");
    }
}

#[test]
fn a_phase_is_spelled_as_its_selector_and_command_word() {
    for row in TABLE {
        let phase = named(row.state);
        assert_eq!(phase.as_str(), row.state);
        assert_eq!(phase.status(), row.command);
    }
}

/// The worker stops a handle before its disconnect state reaches the inbox,
/// so for a moment an endpoint has an outage while `live` still says
/// connected. The Devices overview reads that as the picker does.
#[test]
fn the_devices_overview_does_not_show_a_dropped_host_online() {
    use crate::devices_overview::{Device, Link};
    let mut endpoint = Endpoint::new("ssh:box".into(), "box".into(), ConnectTarget::Local, true);
    endpoint.never_connect();
    endpoint.live.status = ConnectionStatus::Connected;
    endpoint.live.snapshot = Some(Arc::new(crate::sidebar::layout_tests::snapshot(1)));
    endpoint.sync_live();
    let device = Device::new(1, &endpoint, &endpoint.live);
    assert_eq!(device.link, Link::Online);
    assert!(device.tally.total() > 0);

    endpoint.lose_connection_for_test("connection lost");
    assert_eq!(endpoint.status(), "reconnecting");
    let device = Device::new(1, &endpoint, &endpoint.live);
    assert_eq!(device.link, Link::Offline);
    assert_eq!(device.tally.total(), 0, "no stale agents");
}
