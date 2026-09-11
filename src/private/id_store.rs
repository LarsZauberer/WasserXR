use std::collections::HashMap;

use slotmap::{Key, SlotMap};

/// Records addressed by cheap IDs and resolved by unique string names.
///
/// # Design decision
///
/// A [`SlotMap`] provides cheap, copyable, generational IDs without exposing
/// pointers, while a [`HashMap`] makes name resolution independent of the
/// number of records. Storing the not-found error lets each domain choose its
/// own error without requiring a separate error trait or factory.
#[derive(Debug)]
pub(crate) struct IDStore<ID: Key, Record, Error> {
    records: SlotMap<ID, Record>,
    ids: HashMap<String, ID>,
    error: Error,
}

impl<ID: Key, Record, Error: Clone> IDStore<ID, Record, Error> {
    /// Creates an empty store that returns `error` for failed lookups.
    pub(crate) fn new(error: Error) -> Self {
        Self {
            records: SlotMap::with_key(),
            ids: HashMap::new(),
            error,
        }
    }

    /// Inserts `record` under its unique `name` and returns its generated ID.
    ///
    /// # Panics
    ///
    /// Panics if `name` already exists in the store.
    pub(crate) fn insert(&mut self, name: String, record: Record) -> ID {
        assert!(!self.ids.contains_key(&name), "name already exists");
        let id = self.records.insert(record);
        self.ids.insert(name, id);
        id
    }

    /// Removes and returns the record identified by `id`.
    ///
    /// Returns the stored error when `id` is not present.
    pub(crate) fn remove(&mut self, id: ID) -> Result<Record, Error> {
        let record = self.records.remove(id).ok_or_else(|| self.error.clone())?;
        // Removal is O(n); add a reverse map only if profiling justifies it.
        self.ids.retain(|_, stored_id| *stored_id != id);
        Ok(record)
    }

    /// Returns the record identified by `id`.
    ///
    /// Returns the stored error when `id` is not present.
    pub(crate) fn get(&self, id: ID) -> Result<&Record, Error> {
        self.records.get(id).ok_or_else(|| self.error.clone())
    }

    /// Resolves a record name to its ID.
    ///
    /// Returns the stored error when `name` is not present.
    pub(crate) fn resolve_id(&self, name: &str) -> Result<ID, Error> {
        self.ids
            .get(name)
            .copied()
            .ok_or_else(|| self.error.clone())
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
        let mut store: IDStore<TestId, i32, &str> = IDStore::new("not found");
        let id = store.insert("record".to_owned(), 42);

        assert_eq!(store.get(id), Ok(&42));
        assert_eq!(store.resolve_id("record"), Ok(id));
        assert!(store.contains_name("record"));
        assert_eq!(store.keys().collect::<Vec<_>>(), vec![id]);
        assert_eq!(store.values().copied().collect::<Vec<_>>(), vec![42]);
        assert_eq!(store.iter().collect::<Vec<_>>(), vec![(id, &42)]);

        assert_eq!(store.remove(id), Ok(42));
        assert_eq!(store.get(id), Err("not found"));
        assert_eq!(store.resolve_id("record"), Err("not found"));
        assert!(!store.contains_name("record"));
    }
}
