# Coding style

- Functions must be under 40 lines. Split longer ones into smaller functions.
- Names must say what the things is or does. No single letters, except loop counters like `i`.
- Never ignore an error. Handle it, or return it to the caller.
- Never put secrets (API keys, passwords, tokens) in code. Read them from environment variables.
- Comments must explain *why* the code does something, not *what* it does.
- Never leave commented-out code or unused code. Delete it.
- Every new function must have a comment above it that starts with `kavachkit:`.