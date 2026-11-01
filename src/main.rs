use clap::{Parser, Subcommand, ValueEnum};
use kavachkit::Decision;
use std::io::BufRead;
use std::path::Path;

/// Armor for your AI coding agent.
#[derive(Parser)]
#[command(name = "kavachkit", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

/// Whether memory notes are shared in git or kept private.
#[derive(Clone, Copy, Debug, ValueEnum)]
enum MemoryMode {
    Shared,
    Private,
}

#[derive(Subcommand)]
enum Command {
    /// Add kavachkit's rules to this project and keep memory private.
    Install {
        /// Share memory notes with your team in git, or keep them private.
        #[arg(long,value_enum, default_value_t=MemoryMode::Private)]
        memory: MemoryMode,
        /// Show what would change, without changing anything.
        #[arg(long)]
        dry_run: bool,
    },
    /// Remove only the files kavachkit added.
    Uninstall {
        /// Show what would be removed, without removing anything.
        #[arg(long)]
        dry_run: bool,
    },
    /// Check that kavachkit is set up correctly.
    Doctor,
    // Called by Claude Code hooks, not by people, so hidden from --help.
    #[command(hide = true)]
    Guard,
    #[command(hide = true)]
    Warn,
    #[command(hide = true)]
    MemorySave,
    #[command(hide = true)]
    MemoryLoad,
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::Guard => run_guard(),
        Command::Warn => run_warn(),
        Command::MemorySave => run_memory_save(),
        Command::MemoryLoad => run_memory_load(),
        Command::Install { memory, dry_run } => run_install(memory, dry_run),
        Command::Uninstall { dry_run } => run_uninstall(dry_run),
        Command::Doctor => {
            eprintln!("doctor: not built yet");
            Ok(())
        }
    }
}

fn run_uninstall(dry_run: bool) -> std::io::Result<()> {
    let current_folder = std::env::current_dir()?;
    let Some(root) = kavachkit::find_project_root(&current_folder) else {
        eprintln!("kavachkit uninstall: no git project found here.");
        eprintln!("Run it inside a project folder (one with .git).");
        std::process::exit(1);
    };

    let mode = if dry_run { " --dry-run" } else { "" };
    println!("kavachkit uninstall{mode} (project: {})", root.display());

    for line in kavachkit::uninstall(&root, dry_run)? {
        println!("{line}");
    }

    if dry_run {
        println!("Dry run: nothing was changed.");
    } else {
        println!("Done. Restart Claude Code to unload the rules.");
    }

    Ok(())
}

/// `kavachkit install`: adds the rules to this project and keeps memory private.
fn run_install(memory: MemoryMode, dry_run: bool) -> std::io::Result<()> {
    //Find the project folder (the one with .git).
    let current_folder = std::env::current_dir()?;
    let Some(root) = kavachkit::find_project_root(&current_folder) else {
        eprintln!("kavachkit install: no git project found here.");
        eprintln!("Run it inside a project folder (one with .git).");
        std::process::exit(1);
    };
    // Show which project.
    let mode = if dry_run { " --dry-run" } else { "" };
    println!("kavachkit install{mode} (project: {})", root.display());

    // Do the install, and show the report.
    let keep_private = matches!(memory, MemoryMode::Private);
    for line in kavachkit::install(&root, keep_private, dry_run)? {
        println!("{line}");
    }

    // Finish.
    if dry_run {
        println!("Dry run: nothing was changed.");
    } else {
        println!("Done. Restart Claude Code to load the rules.");
    }

    Ok(())
}

fn run_memory_load() -> std::io::Result<()> {
    if let Ok(text) = load_memory_text() {
        print!("{text}");
    }

    Ok(())
}

fn load_memory_text() -> std::io::Result<String> {
    const MAX_NOTES: usize = 3;
    const MAX_CHARS: usize = 4000;

    // Read the message and get the project folder.
    let input = std::io::read_to_string(std::io::stdin())?;
    let event: serde_json::Value = serde_json::from_str(input.trim()).unwrap_or_default();
    let Some(cwd) = event.get("cwd").and_then(|value| value.as_str()) else {
        return Ok(String::new());
    };

    // Open .claude/memory. No folder → nothing to load.
    let memory_dir = Path::new(cwd).join(".claude").join("memory");
    let Ok(entries) = std::fs::read_dir(&memory_dir) else {
        return Ok(String::new());
    };

    // List the .md files with their last-changed time.
    let mut notes: Vec<(std::time::SystemTime, std::path::PathBuf)> = entries
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "md"))
        .filter_map(|path| {
            let changed = path.metadata().ok()?.modified().ok()?;
            Some((changed, path))
        })
        .collect();

    // Newest first
    notes.sort_by_key(|note| std::cmp::Reverse(note.0));

    // Read the newest few.
    let texts: Vec<String> = notes
        .iter()
        .take(MAX_NOTES)
        .filter_map(|(_, path)| std::fs::read_to_string(path).ok())
        .collect();
    // Build the text, with the heading and size limit.
    Ok(kavachkit::build_memory_context(&texts, MAX_CHARS))
}

fn run_memory_save() -> std::io::Result<()> {
    let _ = save_session_note();
    Ok(())
}

fn save_session_note() -> std::io::Result<()> {
    // Read the message from Claude Code.
    let input = std::io::read_to_string(std::io::stdin())?;
    let Ok(session) = kavachkit::parse_session_input(&input) else {
        return Ok(());
    };

    // Read the chat history line by line, and fill the box.
    let file = std::fs::File::open(&session.transcript_path)?;
    let reader = std::io::BufReader::new(file);
    let mut facts = kavachkit::SessionFacts::default();
    for line in reader.lines() {
        let Ok(line) = line else {
            continue;
        };
        facts.add_line(&line, &session.cwd);
    }

    // 3. Write the note to <project>/.claude/memory/<id>.md
    let id = kavachkit::short_session_id(&session.session_id);
    let memory_dir = Path::new(&session.cwd).join(".claude").join("memory");
    std::fs::create_dir_all(&memory_dir)?;
    std::fs::write(memory_dir.join(format!("{id}.md")), facts.to_note(&id))?;
    Ok(())
}

fn run_warn() -> std::io::Result<()> {
    // Real code files are small, and hooks must be fast.
    const MAX_FILE_SIZE: u64 = 1_000_000; // 1 MB

    let input = std::io::read_to_string(std::io::stdin())?;
    // 1. Read the message. If it's broken, stay quiet.
    let Ok(hook) = kavachkit::parse_hook_input(&input) else {
        return Ok(());
    };

    // 2. Get the file path. If there's none, stay quiet.
    let Some(path) = hook.file_path() else {
        return Ok(());
    };

    // Only our own code files: no node_modules, target, README, images…
    //  This check doesn't touch the disk, so it goes first.
    if !kavachkit::should_check_for_debug(path) {
        return Ok(());
    }

    // Skip big files. This only asks the disk for the size.
    let Ok(metadata) = std::fs::metadata(path) else {
        return Ok(());
    };
    if metadata.len() > MAX_FILE_SIZE {
        return Ok(());
    }

    // Read the file. If we can't, stay quiet.
    let Ok(content) = std::fs::read_to_string(path) else {
        return Ok(());
    };

    // Find debug lines.
    let found = kavachkit::find_debug_lines(path, &content);

    // Nothing found → quiet. Found → warn Claude and exit 2.
    if found.is_empty() {
        return Ok(());
    }
    eprintln!("kavachkit: leftover debug lines in {path}:");
    for line in &found {
        eprintln!("  {line}");
    }
    eprintln!("Please remove them before finishing.");
    std::process::exit(2);
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
