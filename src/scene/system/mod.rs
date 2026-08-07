//! Plugin-provided system definitions and active system state.

use std::{rc::Rc, time::Instant};

use crate::scene::serialization::SystemData;

/// C-compatible system declarations used by plugin manifests.
pub mod descriptor;
mod error;

pub use descriptor::{
    Attacher, Detacher, Runner, WXRSystemDescriptor, WXRSystemEntityGroupDescriptor,
};
pub use error::SystemError;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct SelectionGroup {
    components: Vec<String>,
}

impl SelectionGroup {
    pub(crate) fn new(mut components: Vec<String>) -> Self {
        components.sort();
        Self { components }
    }

    pub(crate) fn components(&self) -> &[String] {
        &self.components
    }

    pub(crate) fn matches(&self, mut has_component: impl FnMut(&str) -> bool) -> bool {
        self.components
            .iter()
            .all(|component| has_component(component))
    }
}

pub(crate) struct SystemDefinition {
    id: String,
    plugin_id: String,
    runner: Runner,
    groups: Vec<SelectionGroup>,
    attacher: Option<Attacher>,
    detacher: Option<Detacher>,
}

impl SystemDefinition {
    pub(crate) fn new(
        id: String,
        plugin_id: String,
        runner: Runner,
        groups: Vec<SelectionGroup>,
        attacher: Option<Attacher>,
        detacher: Option<Detacher>,
    ) -> Self {
        Self {
            id,
            plugin_id,
            runner,
            groups,
            attacher,
            detacher,
        }
    }

    pub(crate) fn get_id(&self) -> &str {
        &self.id
    }

    pub(crate) fn get_plugin_id(&self) -> &str {
        &self.plugin_id
    }

    pub(crate) fn groups(&self) -> &[SelectionGroup] {
        &self.groups
    }

    pub(crate) fn runner(&self) -> Runner {
        self.runner
    }
}

pub(crate) struct System {
    definition: Rc<SystemDefinition>,
    priority: usize,
    last_called: Option<Instant>,
}

impl System {
    pub(crate) fn new(definition: Rc<SystemDefinition>, priority: usize) -> Self {
        Self {
            definition,
            priority,
            last_called: None,
        }
    }

    pub(crate) fn get_plugin_id(&self) -> &str {
        &self.definition.plugin_id
    }

    pub(crate) fn serialize(&self) -> SystemData {
        SystemData {
            id: self.definition.id.clone(),
            priority: self.priority,
        }
    }
    pub(crate) fn get_id(&self) -> &str {
        &self.definition.id
    }
    pub(crate) fn get_priority(&self) -> usize {
        self.priority
    }
    pub(crate) fn get_attacher(&self) -> Option<Attacher> {
        self.definition.attacher
    }
    pub(crate) fn get_detacher(&self) -> Option<Detacher> {
        self.definition.detacher
    }
    pub(crate) fn definition(&self) -> Rc<SystemDefinition> {
        Rc::clone(&self.definition)
    }

    pub(crate) fn tick_delta(&mut self) -> f32 {
        let now = Instant::now();
        let delta = self.last_called.map_or(0.0, |last_called| {
            now.duration_since(last_called).as_secs_f32()
        });
        self.last_called = Some(now);
        delta
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_group_normalizes_component_order() {
        let group = SelectionGroup::new(vec!["Mesh".to_owned(), "Transform".to_owned()]);
        assert_eq!(group.components(), ["Mesh", "Transform"]);
    }
}
