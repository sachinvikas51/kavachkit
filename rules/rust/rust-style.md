---
paths:
  - "**/*.rs"
---

# Rust style

- Never use `.unwrap()` or `.expect()` outside tests. Return the error with `?` instead.
- Functions that can fail must return `Result`, never panic.
- Never use `unsafe` code.
- Pass references (`&T`) by default. Only use `.clone()` when the code needs its own copy.
- Code must pass `cargo fmt --check` and `cargo clippy -- -D warnings` with zero warnings.
- Every new public function must have at least one test.
- Never add a new crate to `Cargo.toml` without asking the user first.