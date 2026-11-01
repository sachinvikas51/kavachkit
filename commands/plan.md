---
description: Write a step-by-step plan to .claude/plans/ before writing any code
argument-hint: <what to build or change>
---

Create a plan for: $ARGUMENTS

Do NOT write or change any code. Only create the plan file.

## Steps

1. Read the relevant code so the plan fits the project.
2. If the request is unclear, ask the user up to 3 questions before writing the plan.
3. Save the plan to `.claude/plans/<short-name>.md`, where `<short-name>` is lowercase words joined by hyphens (for example `add-login-page`). If that file already exists, add `-2`, `-3`, and so on. Never overwrite an existing plan.
4. Use this format:

   # Plan: <title>

   ## Goal
   One or two sentences: what will be true when this is done.

   ## Steps
   Numbered, small steps. Each step names the files it touches.

   ## Tests
   Which tests will prove each step works.

   ## Risks
   What could go wrong, and how to avoid it.

   ## Out of scope
   What this plan will NOT do.

5. Show the user the file path and a short summary. Tell them to review or edit the plan, and to say when to start.