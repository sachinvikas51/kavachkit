# Changelog

All notable changes to this project are listed here.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project uses [Semantic Versioning](https://semver.org/).

## [0.5.0] - 2026-11-06

First public release.

### Added
- **Guard hook**: blocks reading or changing `.env` secrets and changing Claude's own
  `.claude/` config, across Bash, Read, Edit, Write, MultiEdit, NotebookEdit, Grep and Glob.
- **Warn hook**: points out leftover debug lines after edits.
- **Memory**: a short note after each reply, loaded at session start, with secrets hidden;
  plus `/kavachkit:save-session` for richer summaries.
- **Scanner** (`kavachkit scan`): finds secrets, risky hooks, over-wide permissions,
  risky MCP servers, prompt-injection phrases, invisible characters and hidden comments.
- **Installer**: `install`, `uninstall` and `doctor`, with `--dry-run`.
- **Claude Code plugin** with a marketplace, and fail-closed hooks.
- Rules, the `kavachkit-tdd` and `kavachkit-review` skills, and a read-only reviewer agent.
