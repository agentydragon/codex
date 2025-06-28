+++
id = "58"
title = "Approval panel above compose panel"
status = "open"
freeform_status = ""
dependencies = [26] # Rationale: depends on Task 26 for separate approval dialog positioning
last_updated = "2025-06-28T02:20:56Z"
+++

# Task 58: Approval panel rendered above compose panel, focus with delay

> *UI behavior improvement specific to codex-rs.*

## Goal

Ensure the approval panel appears above the prompt composer instead of replacing it, and only grabs focus after a configurable idle period:

1. Layout order: conversation history, approval dialog (if present), then prompt composer.
2. Approval dialog auto-focus only if no prompt typing occurred within the last `n` seconds (default 1s, configurable).

## Acceptance Criteria

- Approval panel displayed above the compose area, without hiding or replacing it.
- Layout sequence maintained: conversation history → approval panel → prompt composer.
- Approval panel only receives focus when time since last key entry in composer exceeds the configured threshold.
- Configurable idle threshold exposed via settings (default 1 second).
- Unit/integration tests validate layout ordering and focus-delay behavior.

## Implementation

- Adjust TUI layout code (e.g. in `codex-rs/tui/src/...`) to insert approval panel above composer.
- Track timestamp of last composer key event and compare against idle-threshold before shifting focus.
- Add configuration option `approval_focus_delay_secs` (default = 1.0) and wire into focus logic.
- Update relevant tests to assert correct panel ordering and focus behavior under different timing scenarios.

## Notes

- Depends on the separation of the approval dialog (Task 26).
