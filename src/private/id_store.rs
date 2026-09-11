use std::collections::HashMap;

use slotmap::{Key, SlotMap};

/// Records addressed by cheap IDs and, optionally, unique string names.
///
/// # Design decision
///
/// A [`SlotMap`] provides cheap, copyable, generational IDs without exposing
/// pointers, while a [`HashMap`] makes name resolution independent of the
/// number of records. Lookups return [`Option`] like the underlying standard
/// collections so callers can map absence to their own domain-specific error.
#[derive(Debug)]
pub(crate) struct IDStore<ID: Key, Record> {
    records: SlotMap<ID, Record>,
    ids: HashMap<String, ID>,
}

impl<ID: Key, Record> Default for IDStore<ID, Record> {
    fn default() -> Self {
        Self {
            records: SlotMap::with_key(),
            ids: HashMap::new(),
        }
    }
}

impl<ID: Key, Record> IDStore<ID, Record> {
    /// Inserts an unnamed record and returns its generated ID.
    pub(crate) fn insert(&mut self, record: Record) -> ID {
        self.records.insert(record)
    }

    /// Inserts `record` under its unique `name` and returns its generated ID.
    ///
    /// # Panics
    ///
    /// Panics if `name` already exists in the store.
    pub(crate) fn insert_named(&mut self, name: String, record: Record) -> ID {
        assert!(!self.ids.contains_key(&name), "name already exists");
        let id = self.insert(record);
        self.ids.insert(name, id);
        id
    }

    /// Removes and returns the record identified by `id`.
    pub(crate) fn remove(&mut self, id: ID) -> Option<Record> {
        let record = self.records.remove(id)?;
        // Removal is O(n); add a reverse map only if profiling justifies it.
        self.ids.retain(|_, stored_id| *stored_id != id);
        Some(record)
    }

    /// Returns the record identified by `id`.
    pub(crate) fn get(&self, id: ID) -> Option<&Record> {
        self.records.get(id)
    }

    /// Resolves a record name to its ID.
    pub(crate) fn resolve_id(&self, name: &str) -> Option<ID> {
        self.ids.get(name).copied()
    }

    /// Returns whether a record with `name` exists.
    pub(crate) fn contains_name(&self, name: &str) -> bool {
        self.ids.contains_key(name)
    }

    /// Iterates over the IDs of all stored records.
    pub(crate) fn keys(&self) -> impl Iterator<Item = ID> + '_ {
        self.records.keys()
    }

    /// Iterates over all stored records.
    pub(crate) fn values(&self) -> impl Iterator<Item = &Record> {
        self.records.values()
    }

    /// Iterates over every ID and record pair.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (ID, &Record)> {
        self.records.iter()
    }
}

#[cfg(test)]
mod tests {
    use slotmap::new_key_type;

    use super::IDStore;

    new_key_type! {
        struct TestId;
    }

    #[test]
    fn keeps_id_and_name_lookups_in_sync() {
        let mut store: IDStore<TestId, i32> = IDStore::default();
        let unnamed_id = store.insert(7);
        let id = store.insert_named("record".to_owned(), 42);

        assert_eq!(store.get(unnamed_id), Some(&7));
        assert_eq!(store.get(id), Some(&42));
        assert_eq!(store.resolve_id("record"), Some(id));
        assert!(store.contains_name("record"));
        assert_eq!(store.keys().collect::<Vec<_>>(), vec![unnamed_id, id]);
        assert_eq!(store.values().copied().collect::<Vec<_>>(), vec![7, 42]);
        assert_eq!(
            store.iter().collect::<Vec<_>>(),
            vec![(unnamed_id, &7), (id, &42)]
        );

        assert_eq!(store.remove(id), Some(42));
        assert_eq!(store.get(id), None);
        assert_eq!(store.resolve_id("record"), None);
        assert!(!store.contains_name("record"));
    }
}
