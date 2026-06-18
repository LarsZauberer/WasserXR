use uuid::Uuid;

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
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_id(&self) -> Uuid {
        self.id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
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
}
