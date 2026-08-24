use std::collections::HashMap;

use slotmap::{Key, SlotMap};
use uuid::Uuid;

pub trait StorageBackend {
    type Key: Copy;
    type Value;

    fn insert(&mut self, elem: Self::Value) -> Self::Key;
    fn remove(&mut self, key: Self::Key) -> Option<Self::Value>;
    fn get(&self, key: Self::Key) -> Option<&Self::Value>;
    fn get_mut(&mut self, key: Self::Key) -> Option<&mut Self::Value>;
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
}
