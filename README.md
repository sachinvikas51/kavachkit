# kavachkit

**Armor for your AI coding agent.** kavachkit adds safety hooks, memory, rules, skills and a security scanner to [Claude Code], in one small, fast Rust tool.

*Kavach* (कवच) means "armor" in Hindi.

> **Status: v0.5.0, early release.** It works, and it's tested, but it's young. Please report anything odd.

## Why

AI coding agents are powerful, but out of the box they:

- **forget** everything between sessions,
- **don't always follow** your instructions (a rule is only a request),
- can make **risky moves**, like reading a `.env` file full of secrets.

kavachkit adds real checks that run outside the AI, so they can't be ignored.

## What it does

| Part | What you get |
|---|---|
| 🛡️ **Guard hook** | Blocks Claude from reading or changing `.env` secrets, or changing its own `.claude/` config, whatever tool it tries (Bash, Read, Edit, Write, Grep…) |
| ⚠️ **Warn hook** | After an edit, points out leftover debug lines (`dbg!`, `console.log`) |
| 🧠 **Memory** | Saves a short note after each reply, and loads recent notes when a session starts. Secrets are hidden automatically |
| 🔍 **Scanner** | `kavachkit scan` checks any project's Claude setup for leaked secrets, risky hooks, over-wide permissions, risky MCP servers and prompt-injection tricks |
| 📜 **Rules** | Coding standards Claude follows; Rust rules load only for `.rs` files |
| 🧰 **Skills, commands, agent** | `kavachkit-tdd` and `kavachkit-review` skills, `/kavachkit:plan` and `/kavachkit:save-session`, and a read-only reviewer agent |

## Install

You need [Rust](https://rustup.rs) and [Claude Code].

**1. Install the program:**

```bash
cargo install --git https://github.com/sachinvikas51/kavachkit
```

**2. Install the Claude Code plugin** (inside Claude Code):

```
/plugin marketplace add sachinvikas51/kavachkit
/plugin install kavachkit@kavachkit
```

**3. In your project, add the rules and keep memory private:**

```bash
kavachkit install
kavachkit doctor
```

Then restart Claude Code.

## Commands

| Command | What it does |
|---|---|
| `kavachkit install [--memory shared\|private] [--dry-run]` | Adds the rules and keeps memory notes out of git (private is the default) |
| `kavachkit uninstall [--dry-run]` | Removes only the files kavachkit added, and only if you haven't changed them |
| `kavachkit doctor` | Checks that everything is set up correctly, and tells you how to fix anything |
| `kavachkit scan [folder]` | Looks for danger in a Claude setup. Great before you trust a downloaded project |

## Design principles

- **Fail safe.** If the guard can't run, it blocks; it never silently lets things through.
- **Never touch your work.** Install and uninstall only change files they can prove they created.
- **Fast and offline.** Hooks never use the network.
- **Few dependencies.** Less code running on your machine.

## Privacy

**No telemetry. Nothing leaves your machine.** Memory notes stay in your project, and are kept out of git unless you choose to share them.

## Security

Found a security problem? Please don't open a public issue. See [SECURITY.md](SECURITY.md).

## Credits

Inspired by [ECC](https://github.com/affaan-m/ECC). kavachkit is an independent project written from scratch in Rust.

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.

[Claude Code]: https://docs.claude.com/en/docs/claude-code
