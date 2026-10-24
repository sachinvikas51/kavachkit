fn main() -> std::io::Result<()> {
    let input = std::io::read_to_string(std::io::stdin())?;
    let log_path = std::env::temp_dir().join("kavachkit-hook.log");
    let summary = match kavachkit::parse_hook_input(&input) {
        Ok(hook) => format!("tool={} command={:?}", hook.tool_name, hook.command()),
        Err(error) => format!("could not read hook input: {error}"),
    };
    kavachkit::append_line(&summary, &log_path)?;
    Ok(())
}
