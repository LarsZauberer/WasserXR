use uuid::Uuid;

pub struct Entity {
    id: Uuid,
    name: String,
}

impl Entity {
    pub fn new() -> Self {
        Self {
            id: Uuid::now_v7(),
            name: "".to_owned(),
        }
    }

    pub fn new_with_name(name: String) -> Self {
        Self {
            id: Uuid::now_v7(),
            name: name,
        }
    }

    pub fn get_uuid(&self) -> Uuid {
        self.id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
}
