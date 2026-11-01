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
}
