use anyhow::Result;
use colored::*;
use marty_core::workspace_manager::WorkspaceManager;

pub fn execute(manager: &WorkspaceManager, inferred: bool) -> Result<()> {
    let result = manager.list_projects(inferred)?;

    let heading = if inferred {
        "Projects (inferred)"
    } else {
        "Projects"
    };
    println!("{}", heading.bold().underline());

    if inferred {
        let mut inferred_projects: Vec<_> = result.inferred_projects.iter().collect();
        inferred_projects.sort_by(|a, b| a.name.cmp(&b.name));

        if inferred_projects.is_empty() {
            println!("  {}", "No projects found".dimmed());
            return Ok(());
        }

        for project in inferred_projects {
            let is_tracked = result
                .explicit_projects
                .iter()
                .any(|tracked| tracked.name == project.name);

            if is_tracked {
                println!("{} {}", project.name.blue().bold(), "[marty.yml]".green());
            } else {
                println!(
                    "{} {}",
                    project.name.cyan(),
                    format!("Inferred Project ({} plugin)", project.discovered_by).dimmed()
                );
            }
        }
    } else {
        let mut tracked_projects: Vec<_> = result.explicit_projects.iter().collect();
        tracked_projects.sort_by(|a, b| a.name.cmp(&b.name));

        if tracked_projects.is_empty() {
            println!("  {}", "No projects found".dimmed());
            return Ok(());
        }

        for project in tracked_projects {
            println!("{}", project.name.blue().bold());
        }
    }

    Ok(())
}
