# Security Policy

kavachkit runs on your machine and helps protect your AI coding setup, so we take security reports seriously.

## Reporting a vulnerability

**Please do not open a public issue.**

Report it privately through GitHub:
**Security tab → "Report a vulnerability"** on this repository.

Please include:
- what the problem is and what an attacker could do with it,
- steps to reproduce it,
- your kavachkit version (`kavachkit --version`) and operating system.

## What happens next

- We aim to reply within **7 days**.
- We will keep you updated while we work on a fix.
- Once a fix is released, we will publish a security advisory and credit you (unless you prefer to stay anonymous).

## Supported versions

kavachkit is in early development. Only the **latest release** gets security fixes.

## Scope

In scope:
- the `kavachkit` binary (hooks, installer, scanner, MCP server),
- the rules, skills, commands and agents it installs.

Out of scope:
- bugs in Claude Code itself (report those to Anthropic),
- problems in third-party plugins or MCP servers that kavachkit did not install.