//! Dependency graph data structures for BazBOM
//!
//! This crate provides unified graph representations for dependencies across
//! different build systems. It normalizes Maven, Gradle, and Bazel dependency
//! structures into a common format suitable for:
//! - Vulnerability analysis
//! - Transitive dependency resolution
//! - SBOM generation
//! - Visualization

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Unique identifier for a component in the dependency graph
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

    /// Export the dependency graph to GraphML format
    ///
    /// GraphML is an XML-based format for graphs, compatible with:
    /// - Cytoscape
    /// - Gephi
    /// - yEd
    /// - NetworkX
    pub fn to_graphml(&self) -> String {
        let mut output = String::new();
        output.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        output.push('\n');
        output.push_str(r#"<graphml xmlns="http://graphml.graphdrawing.org/xmlns" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://graphml.graphdrawing.org/xmlns http://graphml.graphdrawing.org/xmlns/1.0/graphml.xsd">"#);
        output.push('\n');

        // Define attributes
        output.push_str(r#"  <key id="name" for="node" attr.name="name" attr.type="string"/>"#);
        output.push('\n');
        output.push_str(r#"  <key id="version" for="node" attr.name="version" attr.type="string"/>"#);
        output.push('\n');
        output.push_str(r#"  <key id="purl" for="node" attr.name="purl" attr.type="string"/>"#);
        output.push('\n');
        output.push_str(r#"  <key id="license" for="node" attr.name="license" attr.type="string"/>"#);
        output.push('\n');
        output.push_str(r#"  <key id="scope" for="node" attr.name="scope" attr.type="string"/>"#);
        output.push('\n');
        output.push_str(r#"  <key id="relationship" for="edge" attr.name="relationship" attr.type="string"/>"#);
        output.push('\n');

        output.push_str(r#"  <graph id="dependency_graph" edgedefault="directed">"#);
        output.push('\n');

        // Add nodes
        for component in self.components.values() {
            let node_id = xml_escape(&component.id.0);
            output.push_str(&format!(r#"    <node id="{}">"#, node_id));
            output.push('\n');

            output.push_str(&format!(r#"      <data key="name">{}</data>"#, xml_escape(&component.name)));
            output.push('\n');
            output.push_str(&format!(r#"      <data key="version">{}</data>"#, xml_escape(&component.version)));
            output.push('\n');

            if let Some(ref purl) = component.purl {
                output.push_str(&format!(r#"      <data key="purl">{}</data>"#, xml_escape(purl)));
                output.push('\n');
            }

            if let Some(ref license) = component.license {
                output.push_str(&format!(r#"      <data key="license">{}</data>"#, xml_escape(license)));
                output.push('\n');
            }

            if let Some(ref scope) = component.scope {
                output.push_str(&format!(r#"      <data key="scope">{}</data>"#, xml_escape(scope)));
                output.push('\n');
            }

            output.push_str("    </node>\n");
        }

        // Add edges
        for edge in &self.edges {
            let from_id = xml_escape(&edge.from.0);
            let to_id = xml_escape(&edge.to.0);
            output.push_str(&format!(
                r#"    <edge source="{}" target="{}">"#,
                from_id, to_id
            ));
            output.push('\n');
            output.push_str(&format!(
                r#"      <data key="relationship">{}</data>"#,
                xml_escape(&edge.relationship)
            ));
            output.push('\n');
            output.push_str("    </edge>\n");
        }

        output.push_str("  </graph>\n");
        output.push_str("</graphml>\n");

        output
    }

    /// Export the dependency graph to DOT format (Graphviz)
    ///
    /// DOT format can be rendered with Graphviz tools:
    /// - dot -Tpng graph.dot -o graph.png
    /// - dot -Tsvg graph.dot -o graph.svg
    /// - dot -Tpdf graph.dot -o graph.pdf
    pub fn to_dot(&self) -> String {
        let mut output = String::new();
        output.push_str("digraph dependency_graph {\n");
        output.push_str("  // Graph attributes\n");
        output.push_str("  rankdir=LR;\n");
        output.push_str("  node [shape=box, style=rounded];\n");
        output.push_str("  edge [fontsize=10];\n");
        output.push('\n');

        // Add nodes
        output.push_str("  // Nodes\n");
        for component in self.components.values() {
            let node_id = dot_escape(&component.id.0);
            let label = format!("{}\\n{}", component.name, component.version);
            let label_escaped = dot_escape(&label);

            output.push_str(&format!(
                r#"  "{}" [label="{}""#,
                node_id, label_escaped
            ));

            // Add color based on scope
            if let Some(ref scope) = component.scope {
                let color = match scope.as_str() {
                    "compile" => "lightblue",
                    "test" => "lightgray",
                    "runtime" => "lightgreen",
                    "provided" => "lightyellow",
                    _ => "white",
                };
                output.push_str(&format!(r#", fillcolor={}, style="filled,rounded""#, color));
            }

            output.push_str("];\n");
        }

        output.push('\n');

        // Add edges
        output.push_str("  // Edges\n");
        for edge in &self.edges {
            let from_id = dot_escape(&edge.from.0);
            let to_id = dot_escape(&edge.to.0);
            let label = dot_escape(&edge.relationship);

            output.push_str(&format!(
                r#"  "{}" -> "{}" [label="{}"];"#,
                from_id, to_id, label
            ));
            output.push('\n');
        }

        output.push_str("}\n");

        output
    }
}

/// Escape XML special characters
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Escape DOT special characters
fn dot_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
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

    #[test]
    fn test_graphml_export() {
        let mut graph = DependencyGraph::new();

        // Add a component
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

        let graphml = graph.to_graphml();

        // Verify XML structure
        assert!(graphml.contains(r#"<?xml version="1.0" encoding="UTF-8"?>"#));
        assert!(graphml.contains(r#"<graphml"#));
        assert!(graphml.contains(r#"<node id="test:component:1.0">"#));
        assert!(graphml.contains(r#"<data key="name">component</data>"#));
        assert!(graphml.contains(r#"<data key="version">1.0</data>"#));
        assert!(graphml.contains(r#"<data key="license">MIT</data>"#));
        assert!(graphml.contains("</graphml>"));
    }

    #[test]
    fn test_dot_export() {
        let mut graph = DependencyGraph::new();

        // Add components
        let id1 = ComponentId::new("test:a:1.0");
        let id2 = ComponentId::new("test:b:2.0");

        let comp1 = Component {
            id: id1.clone(),
            name: "a".to_string(),
            version: "1.0".to_string(),
            purl: None,
            license: None,
            scope: Some("compile".to_string()),
            hash: None,
        };

        let comp2 = Component {
            id: id2.clone(),
            name: "b".to_string(),
            version: "2.0".to_string(),
            purl: None,
            license: None,
            scope: Some("test".to_string()),
            hash: None,
        };

        graph.add_component(comp1);
        graph.add_component(comp2);
        graph.add_edge(id1.clone(), id2.clone(), "depends_on".to_string());

        let dot = graph.to_dot();

        // Verify DOT structure
        assert!(dot.contains("digraph dependency_graph {"));
        assert!(dot.contains(r#""test:a:1.0" [label="a\\n1.0""#));
        assert!(dot.contains(r#""test:b:2.0" [label="b\\n2.0""#));
        assert!(dot.contains(r#""test:a:1.0" -> "test:b:2.0" [label="depends_on"];"#));
        assert!(dot.contains("fillcolor=lightblue")); // compile scope
        assert!(dot.contains("fillcolor=lightgray")); // test scope
        assert!(dot.ends_with("}\n"));
    }

    #[test]
    fn test_xml_escape() {
        let input = r#"<tag attr="value" & 'single'>"#;
        let escaped = xml_escape(input);
        assert_eq!(
            escaped,
            "&lt;tag attr=&quot;value&quot; &amp; &apos;single&apos;&gt;"
        );
    }

    #[test]
    fn test_dot_escape() {
        let input = "line1\nline2\\with\"quotes";
        let escaped = dot_escape(input);
        assert_eq!(escaped, "line1\\nline2\\\\with\\\"quotes");
    }
}
