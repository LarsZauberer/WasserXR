//! This module describes the trait [`StorageBackend`], which is a store that maps a copyable handle
//! to some object.

use std::collections::HashMap;

use slotmap::{Key, SlotMap};
use uuid::Uuid;

/// A storage backend is some data structure that acts similar to a map. It maps some copyable
/// handle to some object. The handle will always be memory safe and never be unsafe like a pointer
/// while it can be easily used without much expensive actions as a key to get data from the data
/// structure.
pub trait StorageBackend {
    type Key: Copy;
    type Value;

    /// Inserts an element into the storage backend. This will produce a unique key that will be
    /// returned, which can later be used to retrieve the object efficiently.
    fn insert(&mut self, elem: Self::Value) -> Self::Key;

    /// Removes an element from the storage backend with a given key. It the object couldn't be
    /// found, the function will return [`None`]. If the funciton found the object associated to the
    /// handle, it will return the ownership of the object.
    fn remove(&mut self, key: Self::Key) -> Option<Self::Value>;

    /// Retrieves a reference to an object with some handle. If the handle doesn't correspond to any
    /// object in the storage backend, then the function will return [`None`]
    fn get(&self, key: Self::Key) -> Option<&Self::Value>;

    /// Same as [`Self::get`] but returns a mutable reference to the object
    fn get_mut(&mut self, key: Self::Key) -> Option<&mut Self::Value>;

    /// Builds an iterator with the references to the objects.
    fn iter(&self) -> impl Iterator<Item = &Self::Value>;

    /// Same as [`Self::iter`] but iterates over mutable references.
    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Self::Value>;

    /// Iterates over all the handles/keys in the storage backend.
    ///
    /// Note: This iterator will return owned values of the handle/key
    fn iter_key(&self) -> impl Iterator<Item = Self::Key>;

    /// Iterates over the handle/key and the object simultaneously
    fn iter_parts(&self) -> impl Iterator<Item = (Self::Key, &Self::Value)>;

    /// Same as [`Self::iter_parts`] but the objects are iterated via mutable references as
    /// [`Self::iter_mut`]
    fn iter_mut_parts(&mut self) -> impl Iterator<Item = (Self::Key, &mut Self::Value)>;
}

impl<V> StorageBackend for HashMap<Uuid, V> {
    type Key = Uuid;
    type Value = V;
    fn insert(&mut self, elem: Self::Value) -> Uuid {
        let uuid = Uuid::new_v4();
        self.insert(uuid, elem);
        uuid
    }

    fn remove(&mut self, key: Uuid) -> Option<Self::Value> {
        self.remove(&key)
    }

    fn get(&self, key: Uuid) -> Option<&Self::Value> {
        self.get(&key)
    }

    fn get_mut(&mut self, key: Uuid) -> Option<&mut Self::Value> {
        self.get_mut(&key)
    }

    fn iter(&self) -> impl Iterator<Item = &Self::Value> {
        self.values()
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Self::Value> {
        self.values_mut()
    }

    fn iter_key(&self) -> impl Iterator<Item = Self::Key> {
        self.keys().copied()
    }

    fn iter_parts(&self) -> impl Iterator<Item = (Self::Key, &Self::Value)> {
        self.iter().map(|(&key, value)| (key, value))
    }

    fn iter_mut_parts(&mut self) -> impl Iterator<Item = (Self::Key, &mut Self::Value)> {
        self.iter_mut().map(|(&key, value)| (key, value))
    }
}

impl<K: Key, V> StorageBackend for SlotMap<K, V> {
    type Key = K;
    type Value = V;
    fn insert(&mut self, elem: Self::Value) -> Self::Key {
        self.insert(elem)
    }

    fn remove(&mut self, key: Self::Key) -> Option<Self::Value> {
        self.remove(key)
    }

    fn get(&self, key: Self::Key) -> Option<&Self::Value> {
        self.get(key)
    }

    fn get_mut(&mut self, key: Self::Key) -> Option<&mut Self::Value> {
        self.get_mut(key)
    }

    fn iter(&self) -> impl Iterator<Item = &Self::Value> {
        self.values()
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut Self::Value> {
        self.values_mut()
    }

    fn iter_key(&self) -> impl Iterator<Item = Self::Key> {
        self.keys()
    }

    fn iter_parts(&self) -> impl Iterator<Item = (Self::Key, &Self::Value)> {
        self.iter()
    }

    fn iter_mut_parts(&mut self) -> impl Iterator<Item = (Self::Key, &mut Self::Value)> {
        self.iter_mut()
    }
}
