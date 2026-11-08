//! A host picker's state: the candidates as last gathered, their ranking,
//! and the host chosen. The ranking follows the samples only until they
//! settle, so the tiles do not reshuffle under the pointer.

use super::{Candidate, Destination, rank};
use std::time::{Duration, Instant};

/// Hosts shown as tiles; the rest are listed behind the "Other" field.
pub(crate) const BEST: usize = 3;
/// How long the ranking may follow the samples before it settles anyway.
/// CPU needs two samples a couple of seconds apart.
const SETTLE: Duration = Duration::from_secs(5);

#[derive(Debug, Clone)]
pub(crate) struct Picker {
    candidates: Vec<Candidate>,
    /// Endpoint IDs, best first.
    order: Vec<String>,
    settled: bool,
    opened: Instant,
    /// The dialog's choice, an endpoint ID. Starts on the current host.
    chosen: String,
    /// Whether the host list below the dialog's field is open.
    open: bool,
}

impl Picker {
    pub(crate) fn new(current: &str, now: Instant) -> Self {
        Self {
            candidates: Vec::new(),
            order: Vec::new(),
            settled: false,
            opened: now,
            chosen: current.to_owned(),
            open: false,
        }
    }

    /// Where the dialog's choice sends the new checkout.
    pub(crate) fn destination(&self) -> Destination<'_> {
        self.dispatched()
            .map_or(Destination::Here, Destination::Host)
    }

    /// Take freshly gathered candidates. Returns whether anything shown changed.
    pub(crate) fn update(&mut self, candidates: Vec<Candidate>, now: Instant) -> bool {
        if candidates == self.candidates && self.settled {
            return false;
        }
        let before = (self.order.clone(), self.chosen.clone());
        let changed = candidates != self.candidates;
        self.candidates = candidates;
        if self.settled {
            // New hosts join at the end; gone ones leave.
            let ids: Vec<&str> = self
                .candidates
                .iter()
                .map(|c| c.endpoint_id.as_str())
                .collect();
            self.order.retain(|id| ids.contains(&id.as_str()));
            for id in ids {
                if !self.order.iter().any(|known| known == id) {
                    self.order.push(id.to_owned());
                }
            }
        } else {
            self.order = rank(&self.candidates)
                .into_iter()
                .map(|index| self.candidates[index].endpoint_id.clone())
                .collect();
            self.settled = self.candidates.iter().all(Candidate::sampled)
                || now.duration_since(self.opened) >= SETTLE;
        }
        if self.chosen().is_none_or(|chosen| !chosen.online)
            && let Some(current) = self.current()
        {
            self.chosen = current.endpoint_id.clone();
        }
        changed || before != (self.order.clone(), self.chosen.clone())
    }

    fn get(&self, endpoint_id: &str) -> Option<&Candidate> {
        self.candidates
            .iter()
            .find(|candidate| candidate.endpoint_id == endpoint_id)
    }

    /// Every candidate, best first.
    pub(crate) fn ranked(&self) -> impl Iterator<Item = &Candidate> {
        self.order.iter().filter_map(|id| self.get(id))
    }

    /// The tiles: the best online hosts.
    pub(crate) fn best(&self) -> impl Iterator<Item = &Candidate> {
        self.ranked().filter(|c| c.online).take(BEST)
    }

    /// Every host that is not a tile, offline ones last.
    pub(crate) fn rest(&self) -> impl Iterator<Item = &Candidate> {
        let best: Vec<&str> = self.best().map(|c| c.endpoint_id.as_str()).collect();
        self.ranked()
            .filter(move |c| !best.contains(&c.endpoint_id.as_str()))
    }

    pub(crate) fn is_best(&self, endpoint_id: &str) -> bool {
        self.best().any(|c| c.endpoint_id == endpoint_id)
    }

    pub(crate) fn current(&self) -> Option<&Candidate> {
        self.candidates.iter().find(|c| c.current)
    }

    /// One host alone is no choice.
    pub(crate) fn offers_choice(&self) -> bool {
        self.candidates.iter().filter(|c| c.online).count() > 1
    }

    pub(crate) fn chosen(&self) -> Option<&Candidate> {
        self.get(&self.chosen)
    }

    /// The chosen host when it is not the current one and no new machine
    /// was chosen instead.
    pub(crate) fn dispatched(&self) -> Option<&Candidate> {
        self.chosen().filter(|chosen| !chosen.current)
    }

    /// The host the dialog's choice shows as picked.
    pub(crate) fn picked(&self) -> Option<&Candidate> {
        self.chosen()
    }

    /// Choose `endpoint_id`, if it is online. Closes any open list.
    pub(crate) fn choose(&mut self, endpoint_id: &str) -> bool {
        self.open = false;
        if !self.get(endpoint_id).is_some_and(|c| c.online) {
            return false;
        }
        let changed = self.chosen != endpoint_id;
        endpoint_id.clone_into(&mut self.chosen);
        changed
    }

    pub(crate) fn open(&self) -> bool {
        self.open
    }

    /// Open or close the dialog's host list.
    pub(crate) fn toggle(&mut self) {
        self.open = !self.open;
    }
}
