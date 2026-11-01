use crate::HookInput;

const SECRET_FILE_MESSAGE: &str = "kavachkit blocked this: .env files hold secrets. Ask the user instead of reading or changing them.";
const CLAUDE_CONFIG_MESSAGE: &str = "kavachkit blocked this: shell commands and edits on .claude/ are not allowed, because it holds Claude's rules and safety settings. To look inside .claude/, use the Read, Grep or Glob tools instead. To change these files, ask the user.";
// Folders inside `.claude/` that Claude may write to: plans and memory notes.
const CLAUDE_WRITABLE_FOLDERS: &[&str] = &["plans", "memory"];

#[derive(Debug, PartialEq)]
pub enum Decision {
    Allow,
    Block(String),
}

fn is_secret_env_file(file_name: &str) -> bool {
    file_name == ".env" || (file_name.starts_with(".env.") && file_name != ".env.example")
}

pub(crate) fn file_name_of(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

pub fn check_command(command: &str) -> Decision {
    for word in command.split_whitespace() {
        let word = word.trim_matches(|c| {
            matches!(
                c,
                '"' | '\'' | ';' | '>' | '<' | '|' | '&' | '(' | ')' | '`'
            )
        });

        if is_secret_env_file(file_name_of(word)) {
            return Decision::Block(SECRET_FILE_MESSAGE.to_string());
        }
        if is_protected_claude_file(word) {
            return Decision::Block(CLAUDE_CONFIG_MESSAGE.to_string());
        }
    }
    Decision::Allow
}

/// True for files inside a `.claude` folder, except the writable folders above.
fn is_protected_claude_file(path: &str) -> bool {
    let parts: Vec<&str> = path.split(['/', '\\']).collect();
    let Some(index) = parts.iter().position(|part| *part == ".claude") else {
        return false;
    };
    match parts.get(index + 1) {
        Some(next) => !CLAUDE_WRITABLE_FOLDERS.contains(next),
        None => true,
    }
}

/// True for tools that change files.
pub(crate) fn is_file_change_tool(tool_name: &str) -> bool {
    matches!(tool_name, "Edit" | "Write" | "MultiEdit" | "NotebookEdit")
}

///Checks one action from Claude, whatever tool it uses.
pub fn check_hook(hook: &HookInput) -> Decision {
    if let Some(command) = hook.command() {
        if let Decision::Block(reason) = check_command(command) {
            return Decision::Block(reason);
        }
    }
    if let Some(path) = hook.file_path() {
        if is_secret_env_file(file_name_of(path)) {
            return Decision::Block(SECRET_FILE_MESSAGE.to_string());
        }
        if is_file_change_tool(&hook.tool_name) && is_protected_claude_file(path) {
            return Decision::Block(CLAUDE_CONFIG_MESSAGE.to_string());
        }
    }

    Decision::Allow
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse_hook_input;

    fn hook(json: &str) -> HookInput {
        parse_hook_input(json).unwrap()
    }

    #[test]
    fn blocks_env_files() {
        for command in [
            "cat .env",
            "cat ./.env",
            "cat config/.env.local",
            "cat '.env'",
            "source .env;",
        ] {
            assert!(
                matches!(check_command(command), Decision::Block(_)),
                "should block: {command}"
            );
        }
    }

    #[test]
    fn allows_safe_commands() {
        for command in [
            "ls",
            "cat README.md",
            "cat .env.example",
            "echo environment",
        ] {
            assert_eq!(
                check_command(command),
                Decision::Allow,
                "should allow: {command}"
            );
        }
    }

    #[test]
    fn blocks_secret_files_for_file_tools() {
        let cases = [
            r#"{"tool_name":"Read","tool_input":{"file_path":"/p/sandbox/.env"}}"#,
            r#"{"tool_name":"Edit","tool_input":{"file_path":"config/.env.local"}}"#,
            r#"{"tool_name":"Write","tool_input":{"file_path":".env"}}"#,
            r#"{"tool_name":"Grep","tool_input":{"pattern":"KEY","path":".env"}}"#,
        ];
        for json in cases {
            assert!(
                matches!(check_hook(&hook(json)), Decision::Block(_)),
                "should block: {json}"
            );
        }
    }

    #[test]
    fn allows_safe_files_for_file_tools() {
        let cases = [
            r#"{"tool_name":"Read","tool_input":{"file_path":".env.example"}}"#,
            r#"{"tool_name":"Read","tool_input":{"file_path":"README.md"}}"#,
        ];
        for json in cases {
            assert_eq!(
                check_hook(&hook(json)),
                Decision::Allow,
                "should allow: {json}"
            );
        }
    }

    #[test]
    fn blocks_changes_to_claude_config() {
        let cases = [
            r#"{"tool_name":"Edit","tool_input":{"file_path":".claude/settings.json"}}"#,
            r#"{"tool_name":"Write","tool_input":{"file_path":"/p/.claude/agents/new.md"}}"#,
            r#"{"tool_name":"MultiEdit","tool_input":{"file_path":".claude/rules/kavachkit/common/coding-style.md"}}"#,
        ];
        for json in cases {
            assert!(
                matches!(check_hook(&hook(json)), Decision::Block(_)),
                "should block: {json}"
            );
        }
    }

    #[test]
    fn allows_plans_and_reading_claude_config() {
        let cases = [
            r#"{"tool_name":"Write","tool_input":{"file_path":".claude/plans/add-login.md"}}"#,
            r#"{"tool_name":"Read","tool_input":{"file_path":".claude/settings.json"}}"#,
            r#"{"tool_name":"Edit","tool_input":{"file_path":"src/main.rs"}}"#,
            r#"{"tool_name":"Write","tool_input":{"file_path":".claude/memory/summary-login.md"}}"#,
        ];
        for json in cases {
            assert_eq!(
                check_hook(&hook(json)),
                Decision::Allow,
                "should allow: {json}"
            );
        }
    }

    #[test]
    fn blocks_shell_changes_to_claude_config() {
        for command in [
            "echo '# test' >> .claude/settings.json",
            "echo x >>.claude/settings.json",
            "sed -i '' 's/a/b/' ~/.claude/settings.json",
            "rm -rf .claude",
        ] {
            assert!(
                matches!(check_command(command), Decision::Block(_)),
                "should block: {command}"
            );
        }
    }

    #[test]
    fn allows_shell_commands_on_plans_and_normal_files() {
        for command in ["cat .claude/plans/add-login.md", "ls src", "cargo test"] {
            assert_eq!(
                check_command(command),
                Decision::Allow,
                "should allow: {command}"
            );
        }
    }
}
