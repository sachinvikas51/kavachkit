use std::path::{Path, PathBuf};

/// How serious a finding is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    Low,
    Medium,
    High,
}

/// One possible danger found in a project's Claude setup.
#[derive(Debug)]
pub struct Finding {
    pub level: Level,
    pub file: String,
    pub line: usize,
    pub what: String,
    pub fix: String,
}

/// The files the scanner looks at: CLAUDE.md, .mcp.json and everything inside .claude/.
pub fn files_to_scan(root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for name in ["CLAUDE.md", ".mcp.json"] {
        let path = root.join(name);
        if path.is_file() {
            files.push(path);
        }
    }
    collect_files(&root.join(".claude"), &mut files);
    files.sort();
    files
}

/// Adds every real file inside `folder` (and its sub-folders) to `files`.
/// Skips shortcuts (symlinks), so we never leave the project or loop forever.
fn collect_files(folder: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entires) = std::fs::read_dir(folder) else {
        return;
    };
    for entry in entires.filter_map(|entry| entry.ok()) {
        let Ok(kind) = entry.file_type() else {
            continue;
        };

        let path = entry.path();
        if kind.is_symlink() {
            continue;
        } else if kind.is_dir() {
            collect_files(&path, files);
        } else if kind.is_file() {
            files.push(path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_claude_files_in_sub_folders_and_skips_others() {
        let project =
            std::env::temp_dir().join(format!("kavachkit-scan-files-{}", std::process::id()));
        let _ = std::fs::remove_dir(&project);
        std::fs::create_dir_all(project.join(".claude/skills/helper")).unwrap();
        std::fs::write(project.join(".claude/settings.json"), "{}").unwrap();
        std::fs::write(project.join(".claude/skills/helper/SKILL.md"), "hi").unwrap();
        std::fs::write(project.join("CLAUDE.md"), "hi").unwrap();
        std::fs::write(project.join("README.md"), "not scanned").unwrap();
        let files = files_to_scan(&project);
        let names: Vec<String> = files
            .iter()
            .map(|file| file.strip_prefix(&project).unwrap().display().to_string())
            .collect();

        assert!(names.contains(&"CLAUDE.md".to_string()));
        assert!(names.contains(&".claude/settings.json".to_string()));
        assert!(names.contains(&".claude/skills/helper/SKILL.md".to_string()));
        assert!(!names.contains(&"README.md".to_string()));

        std::fs::remove_dir_all(&project).unwrap();
    }
}
