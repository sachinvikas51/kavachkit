---
name: kavachkit-tdd
description: Test-driven development workflow. Use when adding a new function or feature, fixing a bug, or changing how existing code behaves. Writes a failing test first, then the smallest code to pass it, then cleans up. Not for docs, config, or formatting-only changes.
---

# Test-driven development (TDD)

Follow these steps in order. Do not skip a step.

## 1. Understand the goal
- Restate in one sentence what the code must do.
- List the cases to cover: the normal case, edge cases (empty, zero, very large), and error cases.

## 2. RED: write a failing test
- Write ONE test for the first case, before writing any code for it.
- Run the tests and confirm the new test FAILS.
- Show the failing output.
- If the FIRST test for new code passes before you write the code, the test is wrong: fix the test.
- If a test for an EXTRA case passes right away, the code already handles it. Keep the test as a regression test and say so.

## 3. GREEN: make it pass
- Write the smallest amount of code that makes the test pass.
- Do not add features the test does not ask for.
- Run the tests and confirm ALL tests pass. Show the output.

## 4. REFACTOR: clean up
- Improve names, remove duplication, and keep functions small.
- Do not change behavior.
- Run the tests again and confirm they still pass.

## 5. Repeat
- Go back to step 2 for the next case, until every case from step 1 is covered.

## 6. Finish
- Run the full checks for the project (for Rust: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`).
- Summarize: which cases are covered, and the final test output.

## Bug fixes
- First write a test that reproduces the bug and FAILS.
- Then fix the code until that test passes.
- Keep the test, so the bug can never come back.