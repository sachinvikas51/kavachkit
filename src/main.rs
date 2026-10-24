use kavachkit::Decision;

fn main() -> std::io::Result<()> {
    let input = std::io::read_to_string(std::io::stdin())?;
    let log_path = std::env::temp_dir().join("kavachkit-hook.log");
    let hook = match kavachkit::parse_hook_input(&input) {
        Ok(hook) => hook,
        Err(error) => {
            kavachkit::append_line(&format!("Block unreadable input:{error}"), &log_path)?;
            eprintln!("kavachkit could not read this action, so it blocked it to be safe.");
            std::process::exit(2);
        }
    };

    let command = hook.command().unwrap_or("");
    match kavachkit::check_command(command) {
        Decision::Allow => {
            kavachkit::append_line(
                &format!("ALLOW tool={} command={command:?}", hook.tool_name),
                &log_path,
            )?;
            Ok(())
        }
        Decision::Block(reason) => {
            kavachkit::append_line(
                &format!("BLOCK tool={} command={command:?}", hook.tool_name),
                &log_path,
            )?;
            eprintln!("{reason}");
            std::process::exit(2);
        }
    }
}
