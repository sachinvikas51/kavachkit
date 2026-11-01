/// Folders with downloaded or generated code that we never check.
const SKIP_FOLDERS: &[&str] = &["node_modules", "target", ".git", "dist", "build", "vendor"];

/// Finds leftover debug lines in a file, based on its file name.
pub fn find_debug_lines(file_name: &str, conetent: &str) -> Vec<String> {
    let extension = file_name.rsplit(".").next().unwrap_or("");
    let patters: &[&str] = match extension {
        "rs" => &["dbg!("],
        "js" | "ts" | "jsx" | "tsx" => &["console.log("],
        _ => &[],
    };

    let mut found = Vec::new();
    for (index, line) in conetent.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }
        if patters.iter().any(|pattern| line.contains(pattern)) {
            found.push(format!("line {}: {}", index + 1, line.trim()));
        }
    }
    found
}

/// True if the warn hook should look inside this file.
pub fn should_check_for_debug(path: &str) -> bool {
    let in_skipped_folder = path
        .split(['/', '\\'])
        .any(|part| SKIP_FOLDERS.contains(&part));
    if in_skipped_folder {
        return false;
    }

    let extension = path.rsplit(".").next().unwrap_or("");
    matches!(extension, "rs" | "js" | "ts" | "jsx" | "tsx")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_debug_lines() {
        let found = find_debug_lines("main.rs", "let x = 1;\ndbg!(x);");
        assert_eq!(found.len(), 1);
        assert!(found[0].starts_with("line 2"));

        for name in ["main.js", "main.ts", "main.jsx", "main.tsx"] {
            let found = find_debug_lines(name, "let x = 1;\nconsole.log(x);");
            assert_eq!(found.len(), 1);
            assert!(found[0].starts_with("line 2"));
        }

        assert!(find_debug_lines("app.js", "// console.log(a);").is_empty());
        assert!(find_debug_lines("notes.md", "dbg!(x)").is_empty());
        assert!(find_debug_lines("main.rs", "let x = 1;").is_empty());
    }

    #[test]
    fn checks_only_own_code_files() {
        assert!(should_check_for_debug("src/main.rs"));
        assert!(should_check_for_debug("web/app.tsx"));

        assert!(!should_check_for_debug("node_modules/lib/index.js"));
        assert!(!should_check_for_debug("target/debug/build.rs"));
        assert!(!should_check_for_debug("/p/.git/hooks/pre-commit.js"));
        assert!(!should_check_for_debug("README.md"));
        assert!(!should_check_for_debug("logo.png"));
    }
}
