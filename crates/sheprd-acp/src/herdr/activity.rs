// ABOUTME: Owns every transition of the reporter's bounded activity mailbox.
// ABOUTME: Retains failed reports until acknowledgement or newer activity.

use crate::session::Activity;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) enum ActivityReport {
    // keep-sorted start
    Acknowledged,
    Queued(Activity),
    QueuedInitial(Activity),
    Reporting {
        activity: Activity,
        latest: Option<Activity>,
    },
    Retry(Activity),
    #[default]
    Unreported,
    // keep-sorted end
}

pub(super) enum Event {
    // keep-sorted start
    Begin,
    Complete { acknowledged: bool },
    Queue(Activity),
    // keep-sorted end
}

impl ActivityReport {
    /// Moves activity between the mailbox and the worker's single request.
    pub(super) fn transition(&mut self, event: Event) -> Option<Activity> {
        let mut report = None;
        let next = match event {
            Event::Queue(latest) => match *self {
                Self::Unreported | Self::QueuedInitial(_) => Self::QueuedInitial(latest),
                Self::Reporting { activity, .. } => Self::Reporting {
                    activity,
                    latest: Some(latest),
                },
                Self::Acknowledged | Self::Queued(_) | Self::Retry(_) => Self::Queued(latest),
            },
            Event::Begin => match *self {
                Self::QueuedInitial(activity) | Self::Queued(activity) | Self::Retry(activity) => {
                    report = Some(activity);
                    Self::Reporting {
                        activity,
                        latest: None,
                    }
                }
                other => other,
            },
            Event::Complete { acknowledged } => match *self {
                Self::Reporting {
                    latest: Some(latest),
                    ..
                } => Self::Queued(latest),
                Self::Reporting { activity, .. } => {
                    if acknowledged {
                        Self::Acknowledged
                    } else {
                        Self::Retry(activity)
                    }
                }
                other => other,
            },
        };
        *self = next;
        report
    }

    pub(super) fn is_queued(&self) -> bool {
        matches!(self, Self::QueuedInitial(_) | Self::Queued(_))
    }

    /// An attempted report may have reached Herdr even when its reply failed.
    pub(super) fn was_attempted(&self) -> bool {
        !matches!(self, Self::Unreported | Self::QueuedInitial(_))
    }

    pub(super) fn needs_retry(&self) -> bool {
        matches!(self, Self::Retry(_))
    }
}
