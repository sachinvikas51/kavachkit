---
description: Write a short summary of this session to .claude/memory/ so the next session can continue
---

Write a short summary of THIS session so a future session can pick up the work.

## Rules

- Only use facts from this conversation. Do not guess or invent anything.
- Never include secrets: no API keys, passwords, tokens, or values from .env files.
- Keep it under 40 lines. Short lines, simple words.
- Do not change any code. Only create the summary file.

## Where to save

Save it to `.claude/memory/summary-<short-topic>.md`, where `<short-topic>` is 2 to 4 lowercase words joined by hyphens (for example `summary-add-login-page.md`). If that file already exists, add `-2`, `-3`, and so on. Never overwrite an existing file.

## Format

# Summary: <title>

## Goal
One or two sentences: what we were trying to do.

## Done
- What is finished.

## Decisions
- What we chose, and why.

## Problems
- What went wrong, and how it was fixed. Write "None" if nothing.

## Next steps
- What to do next, in order.

## Files
- The main files we worked on.

After saving, show the file path and a one-line summary.