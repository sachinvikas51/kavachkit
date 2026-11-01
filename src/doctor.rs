use std::ffi::OsStr;
use std::path::Path;
use crate::install::{MEMORY_GITIGNORE_LINE, RULES};

/// How healthy one check is.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Status {
    Ok,
    Warning,
    Problem,
}

/// The result of one doctor check.
#[derive(Debug)]
pub struct Check {
    pub status: Status,
    pub message: String,
    pub fix: Option<String>,
}

impl Check {
    fn ok(message: &str) -> Self {
        Check {
            status: Status::Ok,
            message: message.to_string(),
            fix: None,
        }
    }

    fn warning(message: &str, fix: &str) -> Self {
        Check {
            status: Status::Warning,
            message: message.to_string(),
            fix: Some(fix.to_string()),
        }
    }

    fn problem(message: &str, fix: &str) -> Self {
        Check {
            status: Status::Problem,
            message: message.to_string(),
            fix: Some(fix.to_string()),
        }
    }
}

/// True if a program with this name is in any folder on the PATH.
pub fn is_on_path(program: &str, path_var: &OsStr) -> bool {
    std::env::split_paths(path_var).any(|folder| {
        folder.join(program).is_file() || folder.join(format!("{program}.exe")).is_file()
    })
}

/// Check 1: the plugin's hooks call plain `kavachkit`, so it must be on the PATH.
pub fn check_program_on_path(path_var: Option<&OsStr>) -> Check {
    let found = path_var.is_some_and(|path| is_on_path("kavachkit", path));
    if found {
        Check::ok("kavachkit is on your PATH")
    } else {
        Check::problem(
            "kavachkit is not on your PATH, so the plugin's guard will block every action",
            "run: cargo install kavachkit",
        )
    }
}

/// Checks 2–5: everything inside the project. `root` is `None` when there's no git project.
pub fn check_project(root: Option<&Path>) -> Vec<Check> {
    let Some(root) = root else {
        return vec![Check::problem(
            "not inside a git project",
            "run kavachkit inside a project folder (one with .git)",
        )];
    };
    vec![
        Check::ok("inside a git project"),
        check_rules(root),
        check_memory_private(root),
        check_hand_made_hooks(root),
    ]
}

/// Check 3: are our rule files there, and unchanged?
fn check_rules(root: &Path) -> Check {
    let mut missing = 0;
    let mut changed = 0;
    for (place, text) in RULES {
        match std::fs::read_to_string(root.join(place)) {
            Ok(existing) if existing == *text => {}
            Ok(_) => changed += 1,
            Err(_) => missing += 1,
        }
    }
    if missing > 0 {
        Check::warning(
            &format!("{missing} rule file(s) are not installed"),
            "run: kavachkit install",
        )
    } else if changed > 0 {
        Check::warning(
            &format!("you changed {changed} rule file(s), so they may be out of date"),
            "nothing to do if you meant to. For the latest rules, delete them and run: kavachkit install",
        )
    } else {
        Check::ok("rules installed and up to date")
    }
}

/// Check 4: is .claude/memory/ kept out of git?
fn check_memory_private(root: &Path) -> Check {
    let gitignore = std::fs::read_to_string(root.join(".gitignore")).unwrap_or_default();
    if gitignore
        .lines()
        .any(|line| line.trim() == MEMORY_GITIGNORE_LINE)
    {
        Check::ok("memory notes are private (in .gitignore)")
    } else {
        Check::warning(
            "memory notes are shared in git (.claude/memory/ is not in .gitignore)",
            "run: kavachkit install   (or ignore this if you chose --memory shared)",
        )
    }
}

/// Check 5: kavachkit hooks written by hand would run twice next to the plugin's hooks.
fn check_hand_made_hooks(root: &Path) -> Check {
    for name in ["settings.json", "settings.local.json"] {
        let settings = std::fs::read_to_string(root.join(".claude").join(name)).unwrap_or_default();
        if settings.contains("kavachkit") {
            return Check::warning(
                &format!("found kavachkit hooks in .claude/{name}"),
                "if you installed the kavachkit plugin, remove these hooks, or every hook runs twice",
            );
        }
    }
    Check::ok("no hand-made kavachkit hooks in .claude/settings")
}

#[cfg(test)]
mod tests {
    use std::*;

    use super::*;

    #[test]
    fn finds_program_only_when_it_is_on_the_path() {
        // A fake bin folder with a file called "kavachkit".
        let bin = std::env::temp_dir().join(format!("kavachkit-doctor-bin-{}", std::process::id()));
        std::fs::create_dir_all(&bin).unwrap();
        std::fs::write(bin.join("kavachkit"), "").unwrap();
        // On the PATH → OK.
        let path = std::env::join_paths([bin.clone()]).unwrap();
        assert_eq!(check_program_on_path(Some(&path)).status, Status::Ok);
        // Not on the PATH → problem.
        let empty_bin = std::env::temp_dir();
        let other_path = std::env::join_paths([empty_bin.join("no-such-folder")]).unwrap();
        assert_eq!(
            check_program_on_path(Some(&other_path)).status,
            Status::Problem
        );

        // No PATH at all → problem.
        assert_eq!(check_program_on_path(None).status, Status::Problem);

        std::fs::remove_dir_all(&bin).unwrap();
    }

    fn fake_project(name: &str) -> std::path::PathBuf {
        let project =
            std::env::temp_dir().join(format!("kavachkit-doctor-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&project);
        std::fs::create_dir_all(project.join(".git")).unwrap();
        project
    }

    #[test]
    fn no_project_is_a_problem() {
        let checks = check_project(None);
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].status, Status::Problem);
    }

    #[test]
    fn healthy_project_is_all_ok() {
        let project = fake_project("healthy");
        crate::install(&project, true, false).unwrap();

        let checks = check_project(Some(&project));
        assert!(checks.iter().all(|check| check.status == Status::Ok));

        std::fs::remove_dir_all(&project).unwrap();
    }

    #[test]
    fn warns_about_missing_rules_shared_memory_and_hand_made_hooks() {
        let project = fake_project("warnings");
        std::fs::create_dir_all(project.join(".claude")).unwrap();
        std::fs::write(
            project.join(".claude/settings.json"),
            r#"{"command":"kavachkit guard"}"#,
        )
        .unwrap();

        let checks = check_project(Some(&project));
        let warnings = checks
            .iter()
            .filter(|check| check.status == Status::Warning)
            .count();
        assert_eq!(warnings, 3);

        std::fs::remove_dir_all(&project).unwrap();
    }
}
