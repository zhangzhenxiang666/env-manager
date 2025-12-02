use daggy::{Dag, NodeIndex, Walker};
use std::collections::{HashMap, HashSet};

use crate::config::models::Profile;

#[derive(Debug)]
pub enum DependencyError {
    CircularDependency(Vec<String>),
    ProfileNotFound(String),
}

impl std::fmt::Display for DependencyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DependencyError::CircularDependency(path) => {
                write!(f, "Circular dependency detected: {}", path.join(" -> "))
            }
            DependencyError::ProfileNotFound(name) => {
                write!(f, "Profile '{name}' not found in configuration.")
            }
        }
    }
}

impl std::error::Error for DependencyError {}

pub struct ProfileGraph {
    pub graph: Dag<String, ()>,
    pub profile_nodes: HashMap<String, NodeIndex>,
}

impl Default for ProfileGraph {
    fn default() -> Self {
        Self {
            graph: Dag::new(),
            profile_nodes: HashMap::new(),
        }
    }
}

impl ProfileGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn build(profiles: &HashMap<String, Profile>) -> Result<Self, DependencyError> {
        let mut profile_graph = Self::new();

        for name in profiles.keys() {
            let index = profile_graph.graph.add_node(name.clone());
            profile_graph.profile_nodes.insert(name.clone(), index);
        }

        for (name, profile) in profiles {
            let parent_index = profile_graph.profile_nodes[name];
            for dep_name in &profile.profiles {
                let &dep_index = profile_graph
                    .profile_nodes
                    .get(dep_name)
                    .ok_or_else(|| DependencyError::ProfileNotFound(dep_name.clone()))?;

                if profile_graph
                    .graph
                    .add_edge(parent_index, dep_index, ())
                    .is_err()
                {
                    // A cycle would be created by adding an edge from `name` to `dep_name`.
                    // This means there is already a path from `dep_name` to `name`.
                    let mut path = profile_graph
                        .find_path(dep_name, name)
                        .unwrap_or_else(|| vec![dep_name.clone(), name.clone()]);

                    // Prepend `name` to show the full cycle: name -> dep_name -> ... -> name
                    path.insert(0, name.clone());

                    return Err(DependencyError::CircularDependency(path));
                }
            }
        }

        Ok(profile_graph)
    }

    pub fn resolve_dependencies(&self, profile_name: &str) -> Result<Vec<String>, DependencyError> {
        let mut resolved = HashSet::new();
        let mut visiting = Vec::new();
        let mut result = Vec::new();

        self.dfs_resolve(profile_name, &mut visiting, &mut resolved, &mut result)?;

        Ok(result)
    }

    fn dfs_resolve<'a>(
        &'a self,
        profile_name: &'a str,
        visiting: &mut Vec<&'a str>,
        resolved: &mut HashSet<&'a str>,
        result: &mut Vec<String>,
    ) -> Result<(), DependencyError> {
        visiting.push(profile_name);

        if let Some(&node_index) = self.profile_nodes.get(profile_name) {
            for (_, child_index) in self.graph.children(node_index).iter(&self.graph) {
                let dep_name: &String = &self.graph[child_index];

                if resolved.contains(dep_name.as_str()) {
                    continue;
                }

                if let Some(pos) = visiting.iter().position(|p| p == &dep_name.as_str()) {
                    let mut cycle_path = visiting[pos..]
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>();
                    cycle_path.push(dep_name.clone());
                    return Err(DependencyError::CircularDependency(cycle_path));
                }
                self.dfs_resolve(dep_name, visiting, resolved, result)?;
            }
        } else {
            return Err(DependencyError::ProfileNotFound(profile_name.to_string()));
        }

        visiting.pop();
        if resolved.insert(profile_name) {
            result.push(profile_name.to_string());
        }

        Ok(())
    }

    pub fn find_path(&self, start_node: &str, end_node: &str) -> Option<Vec<String>> {
        if let (Some(&start_index), Some(&end_index)) = (
            self.profile_nodes.get(start_node),
            self.profile_nodes.get(end_node),
        ) {
            let mut found_path = None;
            let mut visiting = HashSet::new();
            let mut path_stack = vec![start_index];

            self.find_path_dag(end_index, &mut path_stack, &mut visiting, &mut found_path);
            return found_path;
        }
        None
    }

    fn find_path_dag(
        &self,
        end_index: NodeIndex,
        path_stack: &mut Vec<NodeIndex>,
        visiting: &mut HashSet<NodeIndex>,
        found_path: &mut Option<Vec<String>>,
    ) {
        let current_index = *path_stack.last().unwrap();
        visiting.insert(current_index);

        for (_, child_index) in self.graph.children(current_index).iter(&self.graph) {
            if found_path.is_some() {
                return;
            }

            if visiting.contains(&child_index) {
                continue;
            }

            path_stack.push(child_index);

            if child_index == end_index {
                *found_path = Some(path_stack.iter().map(|&i| self.graph[i].clone()).collect());
                path_stack.pop();
                return;
            }

            self.find_path_dag(end_index, path_stack, visiting, found_path);

            path_stack.pop();
        }
        visiting.remove(&current_index);
    }
}
