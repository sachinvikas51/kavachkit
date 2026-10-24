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

impl HookInput {
    ///The shell comman, if this is a Bash tool call.
    pub fn command(&self) -> Option<&str> {
        self.tool_input.get("command").and_then(Value::as_str)
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

#[cfg(test)]
mod tests {
    use super::*;

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
