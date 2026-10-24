use std::path::Path;

fn main() -> std::io::Result<()> {
    let input = std::io::read_to_string(std::io::stdin())?;
    let log_path = std::env::temp_dir().join("kavachkit-hook.log");
    kavachkit::append_line(&input, &log_path)?;
    Ok(())
}
