# kavachkit

**Armor for your AI coding agent.** kavachkit adds rules, skills, safety hooks, memory and a security scanner to [Claude Code], in one small, fast Rust tool.

*Kavach* (कवच) means "armor" in Hindi.

> **Status: early development.** Not ready for use yet. Star or watch the repo to follow progress.

## Why

AI coding agents are powerful, but out of the box they:

- **forget** everything between sessions,
- **don't always follow** your instructions (a rule is only a request),
- **review their own work**, with the same blind spots,
- can make **risky mistakes**, like reading or leaking a secret.

kavachkit fixes these with real checks that run outside the AI, so they can't be ignored.

## What it does (planned)

| Part | What it gives you |
|---|---|
| **Rules** | Short coding standards Claude follows in every session |
| **Skills** | Step-by-step guides (like TDD) that Claude loads only when needed |
| **Commands** | Shortcuts like `/plan` that run the same steps every time |
| **Reviewer agent** | A fresh reviewer that can read your code but never edit it |
| **Hooks** | Automatic guards, e.g. block commands that would expose secrets |
| **Memory** | Remembers what you did last session, stored privately on your machine |
| **Scanner** | `kavachkit scan` checks your setup for secrets, risky hooks and too much permission |
| **MCP server** | Lets Claude search its own memory on demand |

## Design principles

- **One small binary.** Fast, no runtime to install.
- **Safe with your files.** It never overwrites or deletes files it didn't create.
- **Hooks are fast and offline.** They never use the network.
- **Few dependencies.** Less code running on your machine.

## Privacy

**No telemetry. Nothing leaves your machine.** Memory files are stored locally and readable only by you.

## Install

Coming soon: Claude Code plugin, `cargo install`, and ready-made binaries for Linux, macOS and Windows.

## Contributing

Not open for contributions yet. A `CONTRIBUTING.md` will be added before the first release.

## Security

Found a security problem? Please don't open a public issue. See [SECURITY.md](SECURITY.md) for how to report it privately.

## Credits

Inspired by [ECC](https://github.com/affaan-m/ECC). kavachkit is an independent project written from scratch in Rust.

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE), at your option.

[Claude Code]: https://docs.claude.com/en/docs/claude-code