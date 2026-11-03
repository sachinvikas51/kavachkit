use crate::memory::looks_like_secret;
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

/// Shows only the first 4 characters, so the report never leaks the secret again.
fn hint(secret: &str) -> String {
    let start: String = secret.chars().take(4).collect();
    format!("{start}...")
}

/// Shows only the first 4 characters, so the report never leaks the secret again.
pub fn find_secrets(file: &str, text: &str) -> Vec<Finding> {
    let mut findings = Vec::new();
    for (index, line) in text.lines().enumerate() {
        for word in line.split_ascii_whitespace() {
            if looks_like_secret(word) {
                findings.push(Finding {
                    level: Level::High,
                    file: file.to_string(),
                    line: index + 1,
                    what: format!("looks like a secret: {}", hint(word)),
                    fix: "remove it, and make a new key (the old one may already be leaked)"
                        .to_string(),
                });
            }
        }
    }
    findings
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

    #[test]
    fn finds_secrets_with_line_numbers_without_showing_them() {
        let text = "# Deploy\nRun the script.\nUse key sk-live-abc123xyz to log in.";
        let findings = find_secrets("SKILL.md", text);

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].level, Level::High);
        assert_eq!(findings[0].line, 3);
        // The report must never contain the full secret.
        assert!(!findings[0].what.contains("abc123xyz"));
    }

    #[test]
    fn normal_text_has_no_secrets() {
        assert!(find_secrets("notes.md", "keep this secret\ncount=5\ntask-list").is_empty());
    }
}
