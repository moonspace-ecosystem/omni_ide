use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};

use super::canvas::{NodeId, NodeCanvasEditor};

/// A single edge in the directed acyclic graph.
#[derive(Clone, Debug)]
pub struct DagEdge {
    pub from: NodeId,
    pub to: NodeId,
}

/// A node in the output DAG plan.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DagNode {
    pub id: String,
    pub skill: String,
    pub prompt: String,
    pub model: String,
    pub depends_on: Vec<String>,
}

/// The serializable DAG plan that gets written to `plan.json`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DagPlan {
    pub version: String,
    pub nodes: Vec<DagNode>,
}

pub struct DagCompiler;

impl DagCompiler {
    /// Detects if a set of edges contains a cycle using iterative DFS.
    pub fn has_cycle(edges: &[DagEdge]) -> bool {
        let mut adjacency: HashMap<NodeId, Vec<NodeId>> = HashMap::new();
        let mut all_nodes: HashSet<NodeId> = HashSet::new();

        for edge in edges {
            adjacency.entry(edge.from).or_default().push(edge.to);
            all_nodes.insert(edge.from);
            all_nodes.insert(edge.to);
        }

        let mut visited: HashSet<NodeId> = HashSet::new();
        let mut in_stack: HashSet<NodeId> = HashSet::new();

        for &start_node in &all_nodes {
            if visited.contains(&start_node) {
                continue;
            }

            let mut stack: Vec<(NodeId, bool)> = vec![(start_node, false)];

            while let Some((node, is_backtrack)) = stack.pop() {
                if is_backtrack {
                    in_stack.remove(&node);
                    continue;
                }

                if in_stack.contains(&node) {
                    return true;
                }

                if visited.contains(&node) {
                    continue;
                }

                visited.insert(node);
                in_stack.insert(node);
                stack.push((node, true));

                if let Some(neighbors) = adjacency.get(&node) {
                    for &neighbor in neighbors {
                        if in_stack.contains(&neighbor) {
                            return true;
                        }
                        if !visited.contains(&neighbor) {
                            stack.push((neighbor, false));
                        }
                    }
                }
            }
        }

        false
    }

    /// Compiles the current canvas state into a serializable DAG plan.
    pub fn compile(editor: &NodeCanvasEditor) -> DagPlan {
        let nodes = editor.nodes();
        let connections = editor.connections();

        let mut depends_map: HashMap<NodeId, Vec<String>> = HashMap::new();
        for connection in connections {
            depends_map
                .entry(connection.to_node)
                .or_default()
                .push(format!("node_{}", connection.from_node));
        }

        let dag_nodes: Vec<DagNode> = nodes
            .values()
            .map(|node| DagNode {
                id: format!("node_{}", node.id),
                skill: node.skill_name.clone(),
                prompt: node.prompt_override.clone(),
                model: node.model_override.clone(),
                depends_on: depends_map
                    .get(&node.id)
                    .cloned()
                    .unwrap_or_default(),
            })
            .collect();

        DagPlan {
            version: "1.0".to_string(),
            nodes: dag_nodes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_cycle() {
        let edges = vec![
            DagEdge { from: 1, to: 2 },
            DagEdge { from: 2, to: 3 },
            DagEdge { from: 1, to: 3 },
        ];
        assert!(!DagCompiler::has_cycle(&edges));
    }

    #[test]
    fn test_simple_cycle() {
        let edges = vec![
            DagEdge { from: 1, to: 2 },
            DagEdge { from: 2, to: 1 },
        ];
        assert!(DagCompiler::has_cycle(&edges));
    }

    #[test]
    fn test_indirect_cycle() {
        let edges = vec![
            DagEdge { from: 1, to: 2 },
            DagEdge { from: 2, to: 3 },
            DagEdge { from: 3, to: 1 },
        ];
        assert!(DagCompiler::has_cycle(&edges));
    }

    #[test]
    fn test_self_loop() {
        let edges = vec![DagEdge { from: 1, to: 1 }];
        assert!(DagCompiler::has_cycle(&edges));
    }

    #[test]
    fn test_empty_graph() {
        let edges: Vec<DagEdge> = vec![];
        assert!(!DagCompiler::has_cycle(&edges));
    }

    #[test]
    fn test_diamond_no_cycle() {
        let edges = vec![
            DagEdge { from: 1, to: 2 },
            DagEdge { from: 1, to: 3 },
            DagEdge { from: 2, to: 4 },
            DagEdge { from: 3, to: 4 },
        ];
        assert!(!DagCompiler::has_cycle(&edges));
    }
}
