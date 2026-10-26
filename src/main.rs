use kavachkit::Decision;

fn main() -> std::io::Result<()> {
    let job = std::env::args().nth(1);
    match job.as_deref() {
        Some("guard") => run_guard(),
        Some("warn") => {
            println!("warn: not build yet");
            Ok(())
        }
        _ => {
            eprintln!("usage: kavachkit <guard|warn>");

            std::process::exit(1);
        }
    }
}

fn run_guard() -> std::io::Result<()> {
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
    let file = hook.file_path().unwrap_or("");
    match kavachkit::check_hook(&hook) {
        Decision::Allow => {
            kavachkit::append_line(
                &format!(
                    "ALLOW tool={} command={command:?} file={file:?}",
                    hook.tool_name
                ),
                &log_path,
            )?;
            Ok(())
        }
        Decision::Block(reason) => {
            kavachkit::append_line(
                &format!(
                    "BLOCK tool={} command={command:?} file={file:?}",
                    hook.tool_name
                ),
                &log_path,
            )?;
            eprintln!("{reason}");
            std::process::exit(2);
        }
    }
}
