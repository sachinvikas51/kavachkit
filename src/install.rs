use std::io::ErrorKind;
use std::path::{Path, PathBuf};
/// Finds the project folder: the nearest folder (this one or a parent) that has `.git`.
/// Returns `None` if there isn't one, so we never install in the wrong place.
pub fn find_project_root(start: &Path) -> Option<PathBuf> {
    for folder in start.ancestors() {
        if folder.join(".git").exists() {
            return Some(folder.to_path_buf());
        }
    }
    None
}

/// The rule files kavachkit installs, packed inside the program when it's built.
/// Each item is: (where to put it inside the project, the file's text).
pub const RULES: &[(&str, &str)] = &[
    (
        ".claude/rules/kavachkit/common/coding-style.md",
        include_str!("../rules/common/coding-style.md"),
    ),
    (
        ".claude/rules/kavachkit/rust/rust-style.md",
        include_str!("../rules/rust/rust-style.md"),
    ),
];

/// The line that keeps memory notes out of git.
pub const MEMORY_GITIGNORE_LINE: &str = ".claude/memory/";
/// Adds `line` to the end of a `.gitignore` text, if it isn't there already.
/// Returns `None` if nothing needs to change.
pub fn add_gitignore_line(current: &str, line: &str) -> Option<String> {
    if current.lines().any(|existing| existing.trim() == line) {
        return None;
    }
    let mut updated = current.to_string();
    if !updated.is_empty() && !updated.ends_with('\n') {
        updated.push('\n');
    }
    updated.push_str(line);
    updated.push('\n');
    Some(updated)
}

/// Installs kavachkit's rules into a project, and keeps memory private if asked.
/// With `dry_run`, it checks everything and reports, but writes nothing.
/// Returns one report line per thing it looked at.
pub fn install(
    root: &Path,
    keep_memory_private: bool,
    dry_run: bool,
) -> std::io::Result<Vec<String>> {
    let mut report = Vec::new();

    // Each rule file: create it, leave it, or keep the user's version.
    for (place, text) in RULES {
        let path = root.join(place);
        match std::fs::read_to_string(&path) {
            Ok(existing) if existing == *text => {
                report.push(format!("  = {place} (already up to date)"));
            }
            Ok(_) => {
                report.push(format!("  ! {place} (you changed it, so it was kept)"));
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {
                if !dry_run {
                    if let Some(folder) = path.parent() {
                        std::fs::create_dir_all(folder)?;
                    }
                    std::fs::write(&path, text)?;
                }
                report.push(format!("  + {place}"));
            }
            Err(error) => return Err(error),
        }
    }

    //Keep memory notes out of git.
    if keep_memory_private {
        let gitignore = root.join(".gitignore");
        let current = match std::fs::read_to_string(&gitignore) {
            Ok(text) => text,
            Err(error) if error.kind() == ErrorKind::NotFound => String::new(),
            Err(error) => return Err(error),
        };
        match add_gitignore_line(&current, MEMORY_GITIGNORE_LINE) {
            Some(updated) => {
                if !dry_run {
                    std::fs::write(&gitignore, updated)?;
                }
                report.push(format!("  + .gitignore: added {MEMORY_GITIGNORE_LINE}"));
            }
            None => report.push("  = .gitignore already keeps memory private".to_string()),
        }
    }

    Ok(report)
}

pub fn uninstall(root: &Path, dry_run: bool) -> std::io::Result<Vec<String>> {
    let mut report = Vec::new();
    for (place, text) in RULES {
        let path = root.join(place);
        match std::fs::read_to_string(&path) {
            Ok(existing) if existing == *text => {
                if !dry_run {
                    std::fs::remove_file(&path)?;
                }
                report.push(format!("  - {place}"));
            }
            Ok(_) => {
                report.push(format!("  ! {place} (you changed it, so it was kept)"));
            }
            Err(error) if error.kind() == ErrorKind::NotFound => {
                report.push(format!("  = {place} (not installed)"));
            }
            Err(error) => return Err(error),
        }
    }
    if !dry_run {
        remove_empty_rule_folders(root);
    }

    report.push(
        "  = kept .claude/memory/, .claude/plans/ and the .gitignore line (they hold your work)"
            .to_string(),
    );

    Ok(report)
}

fn remove_empty_rule_folders(root: &Path) {
    let folders = [
        ".claude/rules/kavachkit/common",
        ".claude/rules/kavachkit/rust",
        ".claude/rules/kavachkit",
    ];

    for folder in folders {
        let _ = std::fs::remove_dir(root.join(folder));
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn finds_the_folder_with_git() {
        // Make a fake project: <temp>/kavachkit-test-<id>/.git and …/src/utils
        let project = std::env::temp_dir().join(format!("kavachkit-test-{}", std::process::id()));
        let deep_folder = project.join("src").join("utils");
        std::fs::create_dir_all(project.join(".git")).unwrap();
        std::fs::create_dir_all(&deep_folder).unwrap();
        // From deep inside, we find the project folder.
        assert_eq!(find_project_root(&deep_folder), Some(project.clone()));

        //clean up
        std::fs::remove_dir_all(&project).unwrap();
    }

    #[test]
    fn rules_are_packed_inside_the_program() {
        assert_eq!(RULES.len(), 2);
        for (place, text) in RULES {
            assert!(
                place.starts_with(".claude/rules/kavachkit/"),
                "wrong place:{place}"
            );
            assert!(!text.is_empty(), "empty rule file: {place}");
        }
    }

    #[test]
    fn adds_gitignore_line_safely() {
        let line = MEMORY_GITIGNORE_LINE;
        // Empty file: just our line.
        assert_eq!(
            add_gitignore_line("", line),
            Some(".claude/memory/\n".to_string())
        );
        // Normal file: added at the end.
        assert_eq!(
            add_gitignore_line("target\n", line),
            Some("target\n.claude/memory/\n".to_string())
        );

        // No new line at the end: one is added first, so lines don't stick together.
        assert_eq!(
            add_gitignore_line("node_modules", line),
            Some("node_modules\n.claude/memory/\n".to_string())
        );

        // Already there: nothing to change.
        assert_eq!(add_gitignore_line("target\n.claude/memory/\n", line), None);
    }
    /// Makes a fake project folder with .git, for tests.
    fn fake_project(name: &str) -> PathBuf {
        let project = std::env::temp_dir().join(format!("kavachkit-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&project);
        std::fs::create_dir_all(project.join(".git")).unwrap();
        project
    }

    #[test]
    fn install_creates_rules_and_is_safe_to_run_twice() {
        let project = fake_project("install-twice");
        install(&project, true, false).unwrap();
        let second = install(&project, true, false).unwrap();
        // Rules are there.
        assert!(
            project
                .join(".claude/rules/kavachkit/common/coding-style.md")
                .exists()
        );
        // Second run changes nothing.
        assert!(second.iter().all(|line| line.trim_start().starts_with('=')));
        // The .gitignore line appears only once.
        let gitignore = std::fs::read_to_string(project.join(".gitignore")).unwrap();
        assert_eq!(gitignore.matches(MEMORY_GITIGNORE_LINE).count(), 1);
        std::fs::remove_dir_all(&project).unwrap();
    }

    #[test]
    fn install_keeps_files_the_user_changed() {
        let project = fake_project("install-keep");
        install(&project, false, false).unwrap();
        let rule = project.join(".claude/rules/kavachkit/common/coding-style.md");
        std::fs::write(&rule, "my own rules").unwrap();
        install(&project, false, false).unwrap();
        assert_eq!(std::fs::read_to_string(&rule).unwrap(), "my own rules");
        std::fs::remove_dir_all(&project).unwrap();
    }

    #[test]
    fn dry_run_changes_nothing() {
        let project = fake_project("install-dry");
        let report = install(&project, true, true).unwrap();

        assert!(!report.is_empty());
        assert!(!project.join(".claude").exists());
        assert!(!project.join(".gitignore").exists());
        std::fs::remove_dir_all(&project).unwrap();
    }

    #[test]
    fn uninstall_removes_only_our_unchanged_files() {
        let project = fake_project("uninstall-clean");
        install(&project, true, false).unwrap();
        std::fs::create_dir_all(project.join(".claude/memory")).unwrap();
        std::fs::write(project.join(".claude/memory/note.md"), "my note").unwrap();

        uninstall(&project, false).unwrap();

        // Our rules and our folder are gone.
        assert!(!project.join(".claude/rules/kavachkit").exists());
        // The user's note and the .gitignore line are still there.
        assert!(project.join(".claude/memory/note.md").exists());
        let gitignore = std::fs::read_to_string(project.join(".gitignore")).unwrap();
        assert!(gitignore.contains(MEMORY_GITIGNORE_LINE));

        std::fs::remove_dir_all(&project).unwrap();
    }

    #[test]
    fn uninstall_keeps_files_the_user_changed() {
        let project = fake_project("uninstall-keep");
        install(&project, false, false).unwrap();
        let rule = project.join(".claude/rules/kavachkit/common/coding-style.md");
        let my_text = "my own rules";
        std::fs::write(&rule, my_text).unwrap();

        uninstall(&project, false).unwrap();

        assert_eq!(std::fs::read_to_string(&rule).unwrap(), my_text);
        std::fs::remove_dir_all(&project).unwrap();
    }

    #[test]
    fn uninstall_dry_run_removes_nothing() {
        let project = fake_project("uninstall-dry");
        install(&project, false, false).unwrap();

        uninstall(&project, true).unwrap();

        assert!(
            project
                .join(".claude/rules/kavachkit/common/coding-style.md")
                .exists()
        );
        std::fs::remove_dir_all(&project).unwrap();
    }
}
