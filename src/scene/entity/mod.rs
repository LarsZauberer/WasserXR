use uuid::Uuid;

use crate::scene::serialization::EntityData;

/// Entity identity and display name stored by a scene.
pub struct Entity {
    id: Uuid,
    name: String,
}

impl Default for Entity {
    fn default() -> Self {
        Self {
            id: Uuid::now_v7(),
            name: Default::default(),
        }
    }
}

impl Entity {
    /// Creates an entity with a fresh UUID and an empty name.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the stable entity id.
    pub fn get_id(&self) -> Uuid {
        self.id
    }

    /// Returns the entity's display name.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Replaces the entity's display name.
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub(crate) fn serialize(&self) -> EntityData {
        EntityData {
            id: self.id,
            name: self.name.clone(),
        }
    }

    pub(crate) fn deserialize(data: EntityData) -> Self {
        Self {
            id: data.id,
            name: data.name,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn entity_new() {
        let entity = Entity::new();

        assert_ne!(entity.get_id(), Uuid::nil());
    }

    #[test]
    fn entity_get_id_after_name_change() {
        let mut entity = Entity::new();
        let id = entity.get_id();

        entity.set_name("Player".to_owned());

        assert_eq!(entity.get_id(), id);
    }

    #[test]
    fn entity_set_name() {
        let mut entity = Entity::new();

        assert_eq!(entity.get_name(), "");

        entity.set_name("Player".to_owned());
        assert_eq!(entity.get_name(), "Player");
    }

    #[test]
    fn entity_serialize_round_trip() {
        let mut entity = Entity::new();
        entity.set_name("Player".to_owned());
        let id = entity.get_id();

        let deserialized = Entity::deserialize(entity.serialize());

        assert_eq!(deserialized.get_id(), id);
        assert_eq!(deserialized.get_name(), "Player");
    }
}
