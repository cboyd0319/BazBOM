use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ComponentId(pub String);

impl ComponentId {
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: ComponentId,
    pub name: String,
    pub version: String,
    pub purl: Option<String>,
    pub license: Option<String>,
    pub scope: Option<String>,
    pub hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub from: ComponentId,
    pub to: ComponentId,
    pub relationship: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub components: HashMap<ComponentId, Component>,
    pub edges: Vec<Edge>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_component(&mut self, component: Component) {
        self.components.insert(component.id.clone(), component);
    }

    pub fn add_edge(&mut self, from: ComponentId, to: ComponentId, relationship: String) {
        self.edges.push(Edge {
            from,
            to,
            relationship,
        });
    }

    pub fn get_component(&self, id: &ComponentId) -> Option<&Component> {
        self.components.get(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_empty_graph() {
        let graph = DependencyGraph::new();
        assert_eq!(graph.components.len(), 0);
        assert_eq!(graph.edges.len(), 0);
    }

    #[test]
    fn test_add_component() {
        let mut graph = DependencyGraph::new();
        let id = ComponentId::new("test:component:1.0");
        let component = Component {
            id: id.clone(),
            name: "component".to_string(),
            version: "1.0".to_string(),
            purl: Some("pkg:maven/test/component@1.0".to_string()),
            license: Some("MIT".to_string()),
            scope: Some("compile".to_string()),
            hash: None,
        };
        graph.add_component(component);
        assert_eq!(graph.components.len(), 1);
        assert!(graph.get_component(&id).is_some());
    }

    #[test]
    fn test_add_edge() {
        let mut graph = DependencyGraph::new();
        let id1 = ComponentId::new("test:a:1.0");
        let id2 = ComponentId::new("test:b:2.0");
        graph.add_edge(id1, id2, "depends_on".to_string());
        assert_eq!(graph.edges.len(), 1);
    }
}
