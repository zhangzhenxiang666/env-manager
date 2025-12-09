use daggy::{Dag, NodeIndex, Walker};
use std::collections::{HashMap, HashSet};

use crate::config::models::Profile;

#[derive(Debug)]
pub enum DependencyError {
    CircularDependency(Vec<String>),
    /// Profile references a non-existent dependency: (parent_profile, missing_dependency)
    DependencyNotFound(String, String),
    /// Profile itself does not exist
    ProfileNotFound(String),
    /// Context wrapper for dependency errors
    DependencyChain {
        profile: String,
        cause: Box<DependencyError>,
    },
    /// Multiple errors occurred
    MultipleErrors(Vec<DependencyError>),
}

impl std::fmt::Display for DependencyError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // Helper to unwind the chain and build stack trace
        fn unwind_error<'a>(
            err: &'a DependencyError,
            stack: &mut Vec<&'a str>,
        ) -> &'a DependencyError {
            match err {
                DependencyError::DependencyChain { profile, cause } => {
                    stack.push(profile);
                    unwind_error(cause, stack)
                }
                _ => err,
            }
        }

        // Special handling for MultipleErrors - direct printing
        if let DependencyError::MultipleErrors(errors) = self {
            for (i, err) in errors.iter().enumerate() {
                if i > 0 {
                    writeln!(f)?;
                }
                write!(f, "{err}")?;
            }
            return Ok(());
        }

        let mut stack = Vec::new();
        let root_cause = unwind_error(self, &mut stack);

        if !stack.is_empty() {
            write!(f, "Trace: {}", stack.join(" -> "))?;
            write!(f, " -> ")?;
        }

        match root_cause {
            DependencyError::CircularDependency(path) => {
                write!(f, "Circular dependency detected: {}", path.join(" -> "))
            }
            DependencyError::DependencyNotFound(parent, dep) => {
                write!(
                    f,
                    "Profile '{parent}' references non-existent profile '{dep}'."
                )
            }
            DependencyError::ProfileNotFound(profile) => {
                write!(f, "Profile '{profile}' not found.")
            }
            DependencyError::DependencyChain { .. } => unreachable!(),
            DependencyError::MultipleErrors(errors) => {
                // This can happen if MultipleErrors is nested inside DependencyChain
                // In this case, we're at the end of a trace pointing to a multiple error block
                for (i, err) in errors.iter().enumerate() {
                    if i > 0 {
                        writeln!(f)?;
                    }
                    write!(f, "{err}")?; // Recurse
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for DependencyError {}

pub struct ProfileGraph {
    graph: Dag<String, ()>,
    profile_nodes: HashMap<String, NodeIndex>,
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
                let &dep_index = profile_graph.profile_nodes.get(dep_name).ok_or_else(|| {
                    DependencyError::DependencyNotFound(name.clone(), dep_name.clone())
                })?;

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
            // This shouldn't happen if the graph was built correctly, but just in case
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

    /// Get all parent profiles that depend on the given profile
    pub fn get_parents(&self, profile_name: &str) -> Option<Vec<String>> {
        if let Some(&node_index) = self.profile_nodes.get(profile_name) {
            Some(
                self.graph
                    .parents(node_index)
                    .iter(&self.graph)
                    .map(|(_, parent_index)| self.graph[parent_index].clone())
                    .collect(),
            )
        } else {
            None
        }
    }

    /// Add a dependency edge from parent to child
    pub fn add_dependency(&mut self, parent: &str, child: &str) -> Result<(), DependencyError> {
        let &parent_index = self
            .profile_nodes
            .get(parent)
            .ok_or_else(|| DependencyError::ProfileNotFound(parent.to_string()))?;

        let &child_index = self.profile_nodes.get(child).ok_or_else(|| {
            DependencyError::DependencyNotFound(parent.to_string(), child.to_string())
        })?;

        // Try to add the edge
        if self.graph.add_edge(parent_index, child_index, ()).is_err() {
            // Would create a cycle
            let mut path = self
                .find_path(child, parent)
                .unwrap_or_else(|| vec![child.to_string(), parent.to_string()]);
            path.insert(0, parent.to_string());
            return Err(DependencyError::CircularDependency(path));
        }

        Ok(())
    }

    /// Remove a dependency edge from parent to child
    pub fn remove_dependency(&mut self, parent: &str, child: &str) -> Result<(), DependencyError> {
        let &parent_index = self
            .profile_nodes
            .get(parent)
            .ok_or_else(|| DependencyError::ProfileNotFound(parent.to_string()))?;

        let &child_index = self.profile_nodes.get(child).ok_or_else(|| {
            DependencyError::DependencyNotFound(parent.to_string(), child.to_string())
        })?;

        // Find and remove the edge
        if let Some(edge_index) = self.graph.find_edge(parent_index, child_index) {
            self.graph.remove_edge(edge_index);
            Ok(())
        } else {
            // Edge doesn't exist, but that's okay
            Ok(())
        }
    }

    /// Add a new profile node to the graph
    pub fn add_node(&mut self, name: String) {
        if !self.profile_nodes.contains_key(&name) {
            let index = self.graph.add_node(name.clone());
            self.profile_nodes.insert(name, index);
        }
    }

    /// Remove a profile node from the graph
    /// Note: This will also remove all edges connected to this node
    pub fn remove_node(&mut self, name: &str) -> Result<(), DependencyError> {
        if let Some(&node_index) = self.profile_nodes.get(name) {
            self.graph.remove_node(node_index);
            self.profile_nodes.remove(name);
            Ok(())
        } else {
            Err(DependencyError::ProfileNotFound(name.to_string()))
        }
    }

    /// Rename a profile node in the graph
    pub fn rename_node(&mut self, old_name: &str, new_name: String) -> Result<(), DependencyError> {
        let &node_index = self
            .profile_nodes
            .get(old_name)
            .ok_or_else(|| DependencyError::ProfileNotFound(old_name.to_string()))?;

        // Update the node's data
        self.graph[node_index] = new_name.clone();

        // Update the profile_nodes map
        self.profile_nodes.remove(old_name);
        self.profile_nodes.insert(new_name, node_index);

        Ok(())
    }
}
