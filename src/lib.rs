use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

/// Adds `text` as a new libe at the end of the file at `path`.
///  Creates the file if it doesn't exist.
pub fn append_line(text: &str, path: &Path) -> std::io::Result<()> {
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{}", text)?;
    Ok(())
}
