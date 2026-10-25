use serde::Deserialize;
use serde_json::Value;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

/// The part of the hook message that kavachkit cares about.
#[derive(Debug, Deserialize)]
pub struct HookInput {
    pub tool_name: String,
    #[serde(default)]
    pub tool_input: Value,
}

#[derive(Debug, PartialEq)]
pub enum Decision {
    Allow,
    Block(String),
}

const SECRET_FILE_MESSAGE: &str = "kavachkit blocked this: .env files hold secrets. Ask the user instead of reading or changing them.";
const CLAUDE_CONFIG_MESSAGE: &str = "kavachkit blocked this: files in .claude/ control Claude's rules and safety settings. Ask the user to change them.";

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

/// True for tools that change files.
fn is_file_change_tool(tool_name: &str) -> bool {
    matches!(tool_name, "Edit" | "Write" | "MultiEdit" | "NotebookEdit")
}

fn is_protected_claude_file(path: &str) -> bool {
    let parts: Vec<&str> = path.split(['/', '\\']).collect();
    match parts.iter().position(|part| *part == ".claude") {
        Some(index) => parts.get(index + 1) != Some(&"plans"),
        None => false,
    }
}

fn is_secret_env_file(file_name: &str) -> bool {
    file_name == ".env" || (file_name.starts_with(".env.") && file_name != ".env.example")
}

impl HookInput {
    ///The shell comman, if this is a Bash tool call.
    pub fn command(&self) -> Option<&str> {
        self.tool_input.get("command").and_then(Value::as_str)
    }

    /// The file path, if this tool works on a file.
    pub fn file_path(&self) -> Option<&str> {
        self.tool_input
            .get("file_path")
            .or_else(|| self.tool_input.get("path"))
            .and_then(Value::as_str)
    }
}
/// Turns the raw JSON text from Claude Code into a `HookInput`.
pub fn parse_hook_input(text: &str) -> Result<HookInput, serde_json::Error> {
    serde_json::from_str(text.trim())
}
/// Adds `text` as a new libe at the end of the file at `path`.
///  Creates the file if it doesn't exist.
pub fn append_line(text: &str, path: &Path) -> std::io::Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{}", text)?;
    Ok(())
}

fn file_name_of(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hook(json: &str) -> HookInput {
        parse_hook_input(json).unwrap()
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
    fn read_bash_command() {
        let text = r#"{"tool_name":"Bash","tool_input":{"command":"ls"}}"#;
        let hook = parse_hook_input(text).unwrap();
        assert_eq!(hook.tool_name, "Bash");
        assert_eq!(hook.command(), Some("ls"));
    }

    #[test]
    fn ignores_extra_fields() {
        let text =
            r#"{"session_id":"x","cwd":"/tmp","tool_name":"Bash","tool_input":{"command":"ls"}}"#;
        assert!(parse_hook_input(text).is_ok());
    }

    #[test]
    fn no_command_for_other_tools() {
        let text = r#"{"tool_name":"Read","tool_input":{"file_path":"a.txt"}}"#;
        assert_eq!(parse_hook_input(text).unwrap().command(), None);
    }

    #[test]
    fn broken_json_is_an_error() {
        assert!(parse_hook_input("not json").is_err());
    }
}
