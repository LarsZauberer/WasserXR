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

    #[test]
    fn entity_naming_correct() {
        let mut entity = Entity::new();

        assert_eq!(entity.get_name(), "");

        entity.set_name("Player".to_owned());
        assert_eq!(entity.get_name(), "Player");
    }
}
