---
name: kavachkit-review
description: Code review checklist. Use when the user asks to review, check, or look over code, a diff, a file, or a pull request. Finds bugs, security problems, missing tests, and style issues, and reports them by severity. Does not edit code. Not for writing new code or fixing bugs unless the user asks for fixes after the review.
---

# Code review

Review only. Do NOT change any files. Report what you find; the user decides what to fix.

## 1. Understand the change
- Say in one or two sentences what the code is meant to do.
- If you can't tell, ask the user before reviewing.

## 2. Check, in this order

### Correctness
- Does the code do what it is meant to do?
- Edge cases: empty input, zero, very large values, missing files, bad data.
- Off-by-one errors, wrong conditions, wrong return values.

### Errors
- Every error is handled or returned. None are ignored or hidden.
- No crashes on bad input (in Rust: no `unwrap`/`expect` outside tests).

### Security
- No secrets (keys, passwords, tokens) in code or logs.
- Outside input (files, user input, JSON, environment) is checked before it is used.
- Outside input is never put into a shell command, SQL query, or file path without checks.
- No new dependency without a clear reason.

### Tests
- New behavior has tests.
- Tests cover edge cases and error cases, not only the happy path.
- Tests would actually fail if the code broke.

### Readability
- Follows the project rules in `.claude/rules/`.
- Clear names, small functions, comments explain why.
- No dead or commented-out code.

## 3. Report

Group findings by severity:

- **🔴 Must fix**: bugs, security problems, crashes, data loss.
- **🟡 Should fix**: missing tests, unclear code, rule violations.
- **🟢 Nice to have**: small style or naming ideas.

For each finding, give:
- the file and line,
- what is wrong (one sentence),
- why it matters (one sentence),
- a suggested fix (in words; do not edit the file).

End with a one-line verdict. Pick exactly one:
- **Ready to merge**: there are zero 🔴 items. List any 🟡 items as follow-ups.
- **Ready after must-fix items**: there are 1–2 small 🔴 items with simple, clear fixes.
- **Needs another review**: there are 3 or more 🔴 items, or any 🔴 item needs a big change (a redesign, a security fix, or touching many files).

If you find nothing in a category, say "None found". Do not invent problems.