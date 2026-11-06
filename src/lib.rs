use serde::Deserialize;
use serde_json::Value;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
mod warn;
pub use warn::{find_debug_lines, should_check_for_debug};
mod guard;
pub use guard::{Decision, check_command, check_hook};
mod memory;
pub use memory::{SessionFacts, build_memory_context, short_session_id};
mod install;
pub use install::{
    MEMORY_GITIGNORE_LINE, RULES, add_gitignore_line, find_project_root, install, uninstall,
};
mod doctor;
pub use doctor::{Check, Status, check_program_on_path, check_project};
mod scan;
pub use scan::{
    Finding, Level, files_to_scan, find_risky_hooks, find_risky_mcp, find_secrets,
    find_wide_permissions,
};

/// The part of the hook message that kavachkit cares about.
#[derive(Debug, Deserialize)]
pub struct HookInput {
    pub tool_name: String,
    #[serde(default)]
    pub tool_input: Value,
}

/// The parts of a Stop hook message that memory needs.
#[derive(Debug, Deserialize)]
pub struct SessionInput {
    pub session_id: String,
    pub transcript_path: String,
    pub cwd: String,
}

/// Turns the raw JSON text from Claude Code into a `SessionInput`.
pub fn parse_session_input(text: &str) -> Result<SessionInput, serde_json::Error> {
    serde_json::from_str(text.trim())
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

    #[test]
    fn finds_wide_permissions() {
        let text = r#"{
  "permissions": {
    "allow": [
      "Bash",
      "Bash(cargo test:*)"
    ],
    "defaultMode": "bypassPermissions"
  }
}"#;
        let findings = find_wide_permissions("settings.json", text);
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].line, 4); // "Bash" (unlimited)
        assert_eq!(findings[1].line, 7); // bypassPermissions
        assert_eq!(findings[1].level, Level::High);
    }

    #[test]
    fn limited_bash_permission_is_fine() {
        let text = r#""allow": ["Bash(cargo test:*)"]"#;
        assert!(find_wide_permissions("settings.json", text).is_empty());
    }

    #[test]
    fn finds_risky_mcp_servers() {
        let text = r#"{
                            "mcpServers": {
                                "tool": { "command": "npx", "args": ["some-tool@latest"] },
                                "web": { "url": "http://example.com/mcp" },
                                "safe": { "command": "npx", "args": ["other-tool@1.4.2"] }
                            }
                            }"#;
        let findings = find_risky_mcp(".mcp.json", text);
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].line, 3);
        assert_eq!(findings[1].line, 4);
    }
}
