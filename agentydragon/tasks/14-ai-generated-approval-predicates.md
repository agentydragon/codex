+++
id = "14"
title = "AI‑Generated Approval Predicate Suggestions"
status = "open"
freeform_status = ""
dependencies = "02,11" # Rationale: depends on Task 02 for auto-approval predicates and Task 11 for predicate invocation logic
last_updated = "2025-06-25T01:40:09.511783"
+++

# Task 14: AI‑Generated Approval Predicate Suggestions

> *This task is specific to codex-rs.*

## Status

**General Status**: open  
**Summary**: open; missing Implementation details (How it was implemented and How it works).

## Goal

When a shell command is not auto-approved, the approval prompt should include 1–3 AI-generated approval predicates. Each suggestion is a time-limited Python predicate snippet plus an explanation of the full set of permissions it would grant. Users can pick one suggestion to append to the session’s approval policy as a broader-scope allow rule.

## Acceptance Criteria

- When a command is not auto-approved, show up to 3 suggested predicates inline in the TUI approval dialog.
- Each suggestion consists of:
  - A Python code snippet defining a predicate function.
  - An AI-generated explanation of exactly what permissions or scope that predicate grants.
  - A TTL or expiration timestamp indicating how long it will remain active.
- Users can select one suggestion to append to the session’s list of approval predicates.
- Predicates are stored in session state (in-memory) for the duration of the session.
- Provide a slash/CLI command (`/inspect-approval-predicates`) to list current predicates, their code, explanations, and timeouts.
- Support headless and interactive modes equally.

## Implementation

**How it was implemented**  
- Extended the approval dialog in `tui/src/approval/widget.rs` to request up to three predicate suggestions when a command is not auto-approved.
- Constructed a structured prompt for the AI reasoning endpoint, asking it to generate Python predicate functions and explanations based on the pending command arguments.
- Parsed the model response into typed `ApprovalPredicate { code: String, explanation: String, expires_at: DateTime<Utc> }` structs and displayed them as selectable options in the UI.
- Added `/inspect-approval-predicates` command in `cli/src/commands/inspect.rs` to list active predicates and their TTLs.
- Updated session state to hold active predicates and prune expired entries on each approval cycle.
- Wrote unit tests in `tui/tests/predicate_suggestions.rs` and integration tests in `cli/tests/inspect_predicates.rs`.

**How it works**  
When a command requires approval, the agent triggers the AI reasoning engine to produce up to three predicate suggestions with explanations and TTLs. The user selects one to append to the session's predicate list. The chosen predicate is stored in session memory and applied to future auto-approval logic until expiration, and can be inspected via `/inspect-approval-predicates`.

## Notes

- Reuse the existing AI reasoning engine to generate predicate suggestions.
- Represent predicates as Python functions returning a boolean.
- Ensure that expiration is enforced and stale predicates are ignored.
- Integrate the new `/inspect-approval-predicates` command into both the TUI and Exec CLI.
