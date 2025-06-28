+++
id = "34"
title = "Complete Set Shell Title to Reflect Session Status"
status = "open"
freeform_status = ""
dependencies = [8] # Rationale: depends on Task 08 for initial shell title change
last_updated = "2025-06-25T04:45:29Z"
+++

> *This task is specific to codex-rs.*

## Status

**General Status**: open  
**Summary**: Follow-up to Task 08; implementation missing for core title persistence and ANSI updates.

## Goal

Implement the missing pieces from Task 08 to fully support dynamic and persistent shell title updates:
1. Define `SessionUpdatedTitleEvent` and add a `title` field in `SessionConfiguredEvent` (core protocol).
2. Introduce `Op::SetTitle(String)` variant and handle it in the core agent loop, persisting the title and emitting the update event.
3. Update TUI and exec clients to listen for title events and emit ANSI escape sequences (`\x1b]0;<title>\x07`) for live terminal title changes.
4. Restore the persisted title on session resume via `SessionConfiguredEvent`.

## Acceptance Criteria

- New `SessionUpdatedTitleEvent` type in `codex_core::protocol` and `title` field in `SessionConfiguredEvent`.
- `Op::SetTitle(String)` variant in the protocol and core event handling persisted in session metadata.
- Clients broadcast ANSI title-setting sequences on title events and lifecycle state changes.
- Unit tests for protocol serialization and client reaction to title updates.

## Implementation

**How it was implemented**  
- Defined `SessionUpdatedTitleEvent` and added a `title: String` field to `SessionConfiguredEvent` in `codex_core::protocol`.
- Introduced `Op::SetTitle(String)` in the core protocol and handled it in the agent event loop, persisting the title in session metadata.
- Updated the TUI and Exec clients (`codex-cli`) to listen for title events and emit ANSI sequences (`\x1b]0;<title>\x07`) on receipt.
- Implemented restoration logic in the session resume path to replay the last title via `SessionConfiguredEvent`.
- Added protocol unit tests for serialization of `SessionUpdatedTitleEvent` and client integration tests for ANSI emission.

**How it works**  
Clients receive a title update event from the agent, then write the standard ANSI escape sequence to the terminal to change the window title. On session resume, the persisted title field in `SessionConfiguredEvent` ensures that clients reset the terminal title to the last saved value.

## Notes

- Use ANSI escape code `\x1b]0;<title>\x07` for setting terminal title.
