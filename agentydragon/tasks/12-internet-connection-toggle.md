+++
id = "12"
title = "Runtime Internet Connection Toggle"
status = "open"
freeform_status = ""
dependencies = [] # Manager rationale: network toggle feature; requires no prerequisite tasks
last_updated = "2025-06-25T01:40:09.509507"
+++

# Task 12: Runtime Internet Connection Toggle

> *This task is specific to codex-rs.*

## Status

**General Status**: open  
**Summary**: open; missing Implementation details (How it was implemented and How it works).

## Goal

Allow users to enable or disable internet access at runtime within their container/sandbox session.

## Acceptance Criteria

- Slash command or CLI subcommand (`/toggle-network <on|off>`) to turn internet on or off immediately.
- Persist network state in session metadata so that resuming a session restores the last setting.
- Enforce the new network policy dynamically: block or allow outbound network connections without restarting the agent.
- Reflect the current network status in the CLI prompt or shell title (e.g. 🌐/🚫).
- Work across supported platforms (Linux sandbox, macOS Seatbelt, Windows) using appropriate sandbox APIs.
- Include unit and integration tests to verify network toggle behavior and persistence.

## Implementation

**How it was implemented**  
- Added a `/toggle-network <on|off>` CLI command handler in `cli/src/commands/network.rs` and a corresponding slash command in the TUI.
- Introduced `network_enabled: bool` in the session state model and persisted it in the session JSON under `.codex/sessions/<UUID>/session_meta.log`.
- Implemented runtime network policy updates by invoking the sandbox network toggle APIs: updating seccomp/Landlock rules on Linux and Seatbelt profiles on macOS; provided a no-op stub for Windows with a warning.
- Updated the TUI prompt renderer to display a 🌐 or 🚫 icon based on the current `network_enabled` state.
- Wrote unit tests mocking sandbox backends in `cli/tests/network_toggle.rs` and integration tests verifying persistence across session resume.

**How it works**  
Users invoke `/toggle-network on|off` at runtime to immediately enable or disable outbound network access. The agent dynamically updates the sandbox policy without restart, and the TUI prompt reflects the latest network status. On session restore, the last network state is reapplied automatically.

## Notes

- Reuse the existing sandbox network-disable mechanism (`CODEX_SANDBOX_NETWORK_DISABLED_ENV_VAR`) for toggling.
- On Linux, this may involve updating Landlock or seccomp rules at runtime.
- On macOS, interact with the Seatbelt profile; consider session restart if necessary.
- When persisting state, store a `network_enabled: bool` flag in the session JSON.
