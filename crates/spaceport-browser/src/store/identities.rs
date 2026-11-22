// ABOUTME: Reads tab IDs from saved JSON before duplicate keys can collapse.
// ABOUTME: Ignores malformed entries that cannot contain a reference in their schema slot.
use serde::{
    Deserialize, Deserializer,
    de::{
        DeserializeSeed, IgnoredAny, MapAccess, SeqAccess, Visitor,
        value::{MapAccessDeserializer, SeqAccessDeserializer},
    },
};
use serde_json::Value;
use std::fmt;

// A struct's derived deserializer rejects duplicate named fields. Serde
// also accepts structs as arrays, using their declaration order.
#[derive(Default)]
struct Field<T>(T);

type Entries<T> = Field<Vec<Field<T>>>;

trait Positions {
    // None is a list of records. A record names the positions of its ID fields.
    const FIELDS: Option<&'static [usize]>;
}

impl<T> Positions for Vec<T> {
    const FIELDS: Option<&'static [usize]> = None;
}

struct Selected<'a, A> {
    seq: &'a mut A,
    positions: std::slice::Iter<'static, usize>,
    index: usize,
}

impl<'de, A: SeqAccess<'de>> SeqAccess<'de> for Selected<'_, A> {
    type Error = A::Error;

    fn next_element_seed<S: DeserializeSeed<'de>>(
        &mut self,
        seed: S,
    ) -> Result<Option<S::Value>, A::Error> {
        let Some(&position) = self.positions.next() else {
            return Ok(None);
        };
        while self.index < position {
            if self.seq.next_element::<IgnoredAny>()?.is_none() {
                return Ok(None);
            }
            self.index += 1;
        }
        self.index += 1;
        self.seq.next_element_seed(seed)
    }
}

impl<'de, T: Deserialize<'de> + Default + Positions> Deserialize<'de> for Field<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(Self::default())
    }
}

impl<'de, T: Deserialize<'de> + Default + Positions> Visitor<'de> for Field<T> {
    type Value = Self;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a saved JSON field")
    }

    fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self, A::Error> {
        let deserializer = MapAccessDeserializer::new(map);
        if T::FIELDS.is_some() {
            T::deserialize(deserializer).map(Self)
        } else {
            IgnoredAny::deserialize(deserializer).map(|_| self)
        }
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self, A::Error> {
        let Some(positions) = T::FIELDS else {
            return T::deserialize(SeqAccessDeserializer::new(seq)).map(Self);
        };
        let fields = Selected {
            seq: &mut seq,
            positions: positions.iter(),
            index: 0,
        };
        let record = T::deserialize(SeqAccessDeserializer::new(fields))?;
        while seq.next_element::<IgnoredAny>()?.is_some() {}
        Ok(Self(record))
    }

    fn visit_bool<E>(self, _: bool) -> Result<Self, E> {
        Ok(self)
    }
    fn visit_f64<E>(self, _: f64) -> Result<Self, E> {
        Ok(self)
    }
    fn visit_i64<E>(self, _: i64) -> Result<Self, E> {
        Ok(self)
    }
    fn visit_str<E>(self, _: &str) -> Result<Self, E> {
        Ok(self)
    }
    fn visit_u64<E>(self, _: u64) -> Result<Self, E> {
        Ok(self)
    }
    fn visit_unit<E>(self) -> Result<Self, E> {
        Ok(self)
    }
}

pub(crate) fn reserve(next: u64, ids: impl Iterator<Item = u64>) -> u64 {
    ids.max().map_or(next, |id| next.max(id.saturating_add(1)))
}

/// Reads the named entry list, or its first positional slot, independently of the counter.
pub(crate) fn rows<'de, T: Deserialize<'de> + Default>(bytes: &'de [u8]) -> serde_json::Result<T> {
    #[derive(Default, Deserialize)]
    #[serde(transparent)]
    struct Rows<T>(T);
    impl<T> Positions for Rows<T> {
        const FIELDS: Option<&'static [usize]> = Some(&[0]);
    }
    let Field(Rows(rows)) = serde_json::from_slice(bytes)?;
    Ok(rows)
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Tab {
    id: Value,
}
impl Positions for Tab {
    const FIELDS: Option<&'static [usize]> = Some(&[0]);
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Tabs {
    tabs: Entries<Tab>,
    next: Value,
}
impl Positions for Tabs {
    const FIELDS: Option<&'static [usize]> = Some(&[0, 1]);
}

pub(crate) fn tabs(bytes: &[u8]) -> crate::Result<u64> {
    let file = match serde_json::from_slice::<Field<Tabs>>(bytes) {
        Ok(file) => file.0,
        // A torn tab file can start over from the independent layout floor
        // once its original bytes have been preserved.
        Err(_) if serde_json::from_slice::<IgnoredAny>(bytes).is_err() => return Ok(0),
        Err(error) => return Err(error.into()),
    };
    let ids = file.tabs.0.into_iter().filter_map(|tab| tab.0.id.as_u64());
    Ok(reserve(file.next.as_u64().unwrap_or(0), ids))
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Pick {
    kind: Value,
    id: Value,
}
impl Positions for Pick {
    const FIELDS: Option<&'static [usize]> = Some(&[0, 1]);
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Group {
    pick: Field<Pick>,
    tabs: Entries<Pick>,
    hidden: Entries<Pick>,
}
impl Positions for Group {
    const FIELDS: Option<&'static [usize]> = Some(&[0, 2, 3]);
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Layout {
    groups: Entries<Group>,
}
impl Positions for Layout {
    const FIELDS: Option<&'static [usize]> = Some(&[0]);
}

#[derive(Default, Deserialize)]
#[serde(default)]
struct Entry {
    layout: Field<Layout>,
}
impl Positions for Entry {
    const FIELDS: Option<&'static [usize]> = Some(&[2]);
}

#[derive(Deserialize)]
struct Layouts {
    layouts: Vec<Field<Entry>>,
    #[serde(default)]
    next_tab_id: u64,
}

pub(crate) fn layouts(bytes: &[u8]) -> crate::Result<u64> {
    let file: Layouts = serde_json::from_slice(bytes)?;
    let groups = file
        .layouts
        .into_iter()
        .flat_map(|entry| entry.0.layout.0.groups.0);
    let ids = groups
        .map(|group| group.0)
        .flat_map(|group| {
            std::iter::once(group.pick)
                .chain(group.tabs.0)
                .chain(group.hidden.0)
        })
        .filter(|pick| pick.0.kind == "page")
        .filter_map(|pick| pick.0.id.as_u64());
    Ok(reserve(file.next_tab_id, ids))
}
