+++
id = "62"
title = "Support hooks API in codex-rs"
status = "open"
freeform_status = ""
dependencies = [] # Manager rationale: independent codex-rs hooks feature; no prerequisite tasks
last_updated = "2025-07-07"
+++

> *This task is specific to codex-rs.*

# Task 62: Support hooks API in codex-rs

## Acceptance Criteria

- Provide a registration mechanism for external hooks at session start, supplying the session ID.
- Expose hook events:
  - session start
  - session end
  - user message sent
  - model turn ended (user's turn begins)
  - command executed (with exit code and details)
  - patch applied or failed (with outcome)
  - approval dialog presented for commands and patches
  - pre-approval predicate evaluation (auto-approve/auto-deny)
- Allow hooks to register slash commands at runtime, with invocation IDs for correlation.
- Enable hook responses to slash commands to inject messages back into the LLM.
- Allow hooks to augment the approval dialog with custom menu items, receiving approval request IDs.
- Provide access to session state: conversation history, current working directory, last executed command, and any context fields needed.
- Allow hooks to provide feedback to the model on both approved and unapproved command executions and patch applications.
- Allow hooks to insert arbitrary user-visible messages into the session.

## Implementation

**How it was implemented**  

<!-- Developer: outline design and code modules here before implementation. -->

**How it works**  

<!-- Developer: describe runtime behavior here. -->

## Notes

<!-- Any additional notes or references. -->
