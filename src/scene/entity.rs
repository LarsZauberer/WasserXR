use std::fmt::Debug;

pub struct Entity {
    id: usize,
    name: String,
    components: Vec<String>,
}

impl Entity {
    pub fn new(id: usize) -> Self {
        Self {
            id,
            name: "".to_owned(),
            components: Vec::new(),
        }
    }

    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = name.to_owned();
    }

    pub fn component_exists(&self, id: &str) -> bool {
        self.components.iter().find(|x| *x == id).is_some()
    }

    pub fn add_component(&mut self, id: &str) -> bool {
        if self.component_exists(id) {
            false
        } else {
            self.components.push(id.to_string());
            true
        }
    }

    pub fn remove_component(&mut self, id: &str) -> bool {
        if let Some(index) = self.components.iter().position(|x| x == id) {
            self.components.remove(index);
            true
        } else {
            false
        }
    }
}

impl PartialEq for Entity {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Debug for Entity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Entity")
            .field("id", &self.id)
            .field("name", &self.name)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_lifecycle() {
        let mut entity = Entity::new(0);

        assert!(!entity.component_exists("hello"));
        assert!(entity.add_component("hello"));
        assert!(!entity.add_component("hello"));
        assert!(entity.component_exists("hello"));
        assert!(entity.remove_component("hello"));
        assert!(!entity.remove_component("world"));
    }
}
