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
}
