use globset::{Glob, GlobSetBuilder};
use petgraph::algo::kosaraju_scc;
use petgraph::prelude::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

// Re-export types from plugin_protocol for convenience
pub use marty_plugin_protocol::{InferredProject, Project, WorkspaceProvider};

/// Extended workspace structure with dependency graph information
#[derive(Debug)]
pub struct Workspace {
    pub root: PathBuf,
    pub projects: Vec<Project>,
    pub inferred_projects: Vec<InferredProject>,
    pub dep_graph: Option<petgraph::Graph<String, ()>>,
    pub dependency_cycles: Vec<Vec<String>>,
}

impl From<&Workspace> for marty_plugin_protocol::Workspace {
    fn from(workspace: &Workspace) -> Self {
        Self {
            root: workspace.root.clone(),
            projects: workspace.projects.clone(),
            inferred_projects: workspace.inferred_projects.clone(),
        }
    }
}

const DEFAULT_INCLUDE_GLOBS: &[&str] = &["**"];
const DEFAULT_EXCLUDE_GLOBS: &[&str] = &["**/.git/**", "**/target/**", "**/node_modules/**"];

pub fn traverse_workspace(caller: &dyn WorkspaceProvider, workspace: &mut Workspace) {
    let include_globs = caller.include_path_globs();
    let exclude_globs = caller.exclude_path_globs();

    // Use provided includes or default to include everything
    let includes = if include_globs.is_empty() {
        DEFAULT_INCLUDE_GLOBS
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<_>>()
    } else {
        include_globs
    };

    // Combine provided excludes with defaults
    let mut excludes = DEFAULT_EXCLUDE_GLOBS
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
    excludes.extend(exclude_globs);

    // Build the include glob set
    let mut include_builder = GlobSetBuilder::new();
    for pattern in &includes {
        if let Ok(glob) = Glob::new(pattern) {
            include_builder.add(glob);
        }
    }
    let include_set = include_builder.build().unwrap_or_default();

    // Build the exclude glob set
    let mut exclude_builder = GlobSetBuilder::new();
    for pattern in &excludes {
        if let Ok(glob) = Glob::new(pattern) {
            exclude_builder.add(glob);
        }
    }
    let exclude_set = exclude_builder.build().unwrap_or_default();

    let mut queue = VecDeque::new();
    queue.push_back(workspace.root.clone());

    while let Some(current_dir) = queue.pop_front() {
        if let Ok(entries) = std::fs::read_dir(&current_dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                // Check if path should be included and not excluded
                let relative_path = path.strip_prefix(&workspace.root).unwrap_or(&path);

                // Skip if explicitly excluded
                if exclude_set.is_match(relative_path) {
                    continue;
                }

                // For files, check if they match include patterns
                if path.is_file() && !include_set.is_match(relative_path) {
                    continue;
                }

                if path.is_file() {
                    let plugin_workspace = marty_plugin_protocol::Workspace::from(&*workspace);
                    if let Some(project) = caller.on_file_found(&plugin_workspace, &path) {
                        let manifest_path = project.project_dir.join("marty.yml");

                        if manifest_path.exists() {
                            workspace.projects.push(Project {
                                name: project.name.clone(),
                                project_dir: project.project_dir.clone(),
                                file_path: Some(manifest_path),
                            });
                        }

                        workspace.inferred_projects.push(project);
                    }
                } else if path.is_dir() {
                    queue.push_back(path);
                }
            }
        }
    }
}

/// Build the dependency graph from the projects in the workspace
pub fn build_dependency_graph(workspace: &mut Workspace) -> Result<(), String> {
    let mut graph = DiGraph::<String, ()>::new();
    let mut node_indices = HashMap::new();

    // Add all projects as nodes
    for project in &workspace.projects {
        let node_index = graph.add_node(project.name.clone());
        node_indices.insert(project.name.clone(), node_index);
    }

    // Add edges for dependencies
    for project in &workspace.projects {
        let from_node = node_indices[&project.name];
        let inferred_project = workspace
            .inferred_projects
            .iter()
            .find(|p| p.name == project.name);
        if inferred_project.is_none() {
            continue;
        }
        for dep in &inferred_project.unwrap().workspace_dependencies {
            if let Some(&to_node) = node_indices.get(dep) {
                // Add edge: project -> dependency (dependency comes first)
                graph.add_edge(from_node, to_node, ());
            } else {
                return Err(format!(
                    "Project '{}' depends on '{}' which was not found",
                    project.name, dep
                ));
            }
        }
    }

    // Detect cycles using strongly connected components
    let mut cycles: Vec<Vec<String>> = kosaraju_scc(&graph)
        .into_iter()
        .filter_map(|component| {
            if component.len() > 1 {
                let mut cycle = component
                    .iter()
                    .map(|node| graph[*node].clone())
                    .collect::<Vec<_>>();
                cycle.sort();
                Some(cycle)
            } else {
                let node = component[0];
                if graph.contains_edge(node, node) {
                    Some(vec![graph[node].clone()])
                } else {
                    None
                }
            }
        })
        .collect();

    cycles.sort();

    workspace.dependency_cycles = cycles;
    workspace.dep_graph = Some(graph);
    Ok(())
}

/// Get all recursive dependencies for the given targets
/// Returns dependencies in topological order (dependencies first)
pub fn get_recursive_dependencies(
    workspace: &Workspace,
    targets: &[String],
) -> Result<Vec<String>, String> {
    if workspace.dep_graph.is_none() {
        return Err("Dependency graph not built. Call build_dependency_graph first.".to_string());
    }

    let graph = workspace.dep_graph.as_ref().unwrap();
    let mut visited = HashSet::new();
    let mut result = Vec::new();
    let mut stack = Vec::new();

    // Create a reverse mapping from project names to node indices
    let mut name_to_node = HashMap::new();
    for (node_index, node_weight) in graph.node_indices().zip(graph.node_weights()) {
        name_to_node.insert(node_weight.clone(), node_index);
    }

    // Resolve targets to node indices and prime traversal structures
    let mut start_nodes = Vec::new();
    for target in targets {
        if let Some(&node_index) = name_to_node.get(target) {
            start_nodes.push(node_index);
        } else {
            return Err(format!(
                "Target project '{}' not found in workspace",
                target
            ));
        }
    }

    // Determine all nodes reachable from the targets (dependencies)
    let mut reachable_nodes = HashSet::new();
    let mut queue: VecDeque<NodeIndex> = start_nodes.iter().copied().collect();
    while let Some(node_index) = queue.pop_front() {
        if !reachable_nodes.insert(node_index) {
            continue;
        }

        for neighbor in graph.neighbors(node_index) {
            queue.push_back(neighbor);
        }
    }

    // If cycles exist that involve reachable nodes, report them
    if !workspace.dependency_cycles.is_empty() {
        let reachable_names: HashSet<String> = reachable_nodes
            .iter()
            .map(|node| graph[*node].clone())
            .collect();

        let mut relevant_cycles: Vec<Vec<String>> = workspace
            .dependency_cycles
            .iter()
            .filter(|cycle| cycle.iter().any(|name| reachable_names.contains(name)))
            .cloned()
            .collect();

        if !relevant_cycles.is_empty() {
            relevant_cycles.sort();
            let message = relevant_cycles
                .into_iter()
                .map(|cycle| {
                    let mut cycle_path = cycle.clone();
                    if let Some(first) = cycle_path.first().cloned() {
                        cycle_path.push(first);
                    }
                    cycle_path.join(" -> ")
                })
                .collect::<Vec<_>>()
                .join("; ");

            return Err(format!("Circular dependency detected: {}", message));
        }
    }

    // Start DFS with target projects
    for node_index in start_nodes {
        stack.push(node_index);
    }

    // DFS to collect all dependencies
    while let Some(current_node) = stack.pop() {
        if visited.contains(&current_node) {
            continue;
        }
        visited.insert(current_node);

        // Add dependencies first (recursive)
        for neighbor in graph.neighbors(current_node) {
            if !visited.contains(&neighbor) {
                stack.push(neighbor);
            }
        }

        // Add current project to result
        if let Some(project_name) = graph.node_weight(current_node) {
            result.push(project_name.clone());
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestCaller;

    impl WorkspaceProvider for TestCaller {
        fn include_path_globs(&self) -> Vec<String> {
            // For testing, include everything except ignored paths
            vec!["**".to_string()]
        }
        fn exclude_path_globs(&self) -> Vec<String> {
            // Exclude the ignored directory for testing
            vec!["**/ignored/**".to_string()]
        }
        fn on_file_found(
            &self,
            _workspace: &marty_plugin_protocol::Workspace,
            path: &std::path::Path,
        ) -> Option<InferredProject> {
            // Example logic: treat directories containing "project_config.txt" as projects
            if path.file_name().unwrap_or_default() == "project_config.txt" {
                if let Some(parent) = path.parent() {
                    let name = String::from(parent.file_name()?.to_str()?);
                    let file_contents = std::fs::read_to_string(path).ok()?;
                    let mut dependencies = Vec::new();
                    for line in file_contents.lines() {
                        if let Some(dep) = line.strip_prefix("dep=") {
                            dependencies.push(dep.trim().to_string());
                        }
                    }
                    Some(InferredProject {
                        name,
                        project_dir: parent.to_path_buf(),
                        workspace_dependencies: dependencies,
                        discovered_by: "test".to_string(),
                    })
                } else {
                    None
                }
            } else {
                None
            }
        }
    }

    #[test]
    fn test_traverse_workspace_with_examples_traverse() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../examples/traverse")
            .canonicalize()
            .expect("examples/traverse directory should exist for tests");
        println!("Testing traversal in {:?}", root);
        let mut workspace = Workspace {
            root,
            projects: Vec::new(),
            inferred_projects: Vec::new(),
            dep_graph: None,
            dependency_cycles: Vec::new(),
        };
        let caller = TestCaller;

        traverse_workspace(&caller, &mut workspace);

        assert_eq!(
            workspace.inferred_projects.len(),
            3,
            "Should find 3 inferred projects in examples/traverse"
        );
        assert_eq!(
            workspace.inferred_projects[0].name, "traverse",
            "Project name should be 'traverse'"
        );
        assert_eq!(
            workspace.inferred_projects[1].name, "l1",
            "Project name should be 'project1'"
        );
        assert_eq!(
            workspace.inferred_projects[2].name, "l2",
            "Project name should be 'project2'"
        );
    }

    // Additional test for edge case: empty directory
    #[test]
    fn test_traverse_workspace_empty_dir() {
        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path().to_path_buf();
        let mut workspace = Workspace {
            root,
            projects: Vec::new(),
            inferred_projects: Vec::new(),
            dep_graph: None,
            dependency_cycles: Vec::new(),
        };
        let caller = TestCaller;

        traverse_workspace(&caller, &mut workspace);

        assert!(
            workspace.projects.is_empty(),
            "Should find no projects in empty directory"
        );
    }

    // Additional test for directory with non-matching files
    #[test]
    fn test_traverse_workspace_no_projects() {
        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path().to_path_buf();
        // Create a file that doesn't match
        std::fs::write(root.join("readme.txt"), "test").unwrap();
        let mut workspace = Workspace {
            root,
            projects: Vec::new(),
            inferred_projects: Vec::new(),
            dep_graph: None,
            dependency_cycles: Vec::new(),
        };
        let caller = TestCaller;

        traverse_workspace(&caller, &mut workspace);

        assert!(
            workspace.projects.is_empty(),
            "Should find no projects when no Cargo.toml present"
        );
    }

    #[test]
    fn test_cycle_detection_in_dependency_graph() {
        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path().to_path_buf();

        let project_a = root.join("a");
        let project_b = root.join("b");
        std::fs::create_dir_all(&project_a).unwrap();
        std::fs::create_dir_all(&project_b).unwrap();

        std::fs::write(project_a.join("project_config.txt"), "dep=b\n").unwrap();
        std::fs::write(project_b.join("project_config.txt"), "dep=a\n").unwrap();

        std::fs::write(project_a.join("marty.yml"), "name: a\n").unwrap();
        std::fs::write(project_b.join("marty.yml"), "name: b\n").unwrap();

        let mut workspace = Workspace {
            root,
            projects: Vec::new(),
            inferred_projects: Vec::new(),
            dep_graph: None,
            dependency_cycles: Vec::new(),
        };
        let caller = TestCaller;

        traverse_workspace(&caller, &mut workspace);

        assert_eq!(
            workspace.projects.len(),
            2,
            "Expected both projects to be tracked"
        );

        build_dependency_graph(&mut workspace).expect("Graph should build even with cycles");

        assert_eq!(
            workspace.dependency_cycles.len(),
            1,
            "One cycle should be detected"
        );
        let cycle = &workspace.dependency_cycles[0];
        assert_eq!(cycle, &vec!["a".to_string(), "b".to_string()]);

        let err = get_recursive_dependencies(&workspace, &["a".to_string()])
            .expect_err("Cycles should prevent dependency resolution");
        assert!(
            err.contains("Circular dependency detected"),
            "Error message should mention circular dependencies"
        );
        assert!(
            err.contains("a -> b -> a"),
            "Cycle should be reported in message"
        );
    }
}
