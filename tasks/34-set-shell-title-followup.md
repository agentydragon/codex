+++
id = "34"
title = "Complete Set Shell Title to Reflect Session Status"
status = "open"
freeform_status = ""
dependencies = [8] # Rationale: depends on Task 08 for initial shell title change
last_updated = "2025-06-28T02:20:56Z"
+++

> *This task is specific to codex-rs.*

## Status

**General Status**: open  
**Summary**: Follow-up to Task 08; implement dynamic state-based titles with ANSI persistence, LLM-based summaries, and user override support.

## Goal

Implement dynamic and persistent shell title updates with advanced features:

1. Display the Codex session state (e.g., running command, sampling, idle) in the terminal title.
2. Optionally request a concise conversation summary from a small LLM (e.g., gpt-4o-mini) and allow it to update the title based on context.
3. Provide a `/title <custom>` slash command for users to set a custom title, while still updating the state icon automatically.
4. Define `SessionUpdatedTitleEvent` and add a `title` field to `SessionConfiguredEvent` in the core protocol.
5. Introduce an `Op::SetTitle(String)` variant and handle it in the core agent loop, persisting titles and emitting update events.
6. Update TUI and exec clients to emit ANSI escape sequences (`\x1b]0;<title>\x07`) on title events and lifecycle changes.
7. Restore the persisted title on session resume via the `SessionConfiguredEvent`.

## Acceptance Criteria

- Title displays current Codex state icon (running, sampling, idle, etc.) in the terminal title.
- Support for optional LLM-based summaries: call a small LLM to generate a conversation summary and update the title.
- `/title <custom>` slash command for user-defined titles, with automatic state icon overlays.
- New `SessionUpdatedTitleEvent` and added `title: String` field to `SessionConfiguredEvent` in `codex_core::protocol`.
- `Op::SetTitle(String)` variant in the protocol and core event loop, with persisted session metadata.
- TUI and exec clients emit ANSI escape sequences (`\x1b]0;<title>\x07`) for live title updates and restore on resume.
- Unit and integration tests covering protocol serialization, ANSI emission, LLM summary logic, and slash-command parsing.

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
