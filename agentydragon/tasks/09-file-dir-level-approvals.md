+++
id = "09"
title = "File- and Directory-Level Approvals"
status = "open"
freeform_status = ""
dependencies = "11" # Rationale: depends on Task 11 for custom approval predicate infrastructure
last_updated = "2025-06-25T01:40:09.507043"
+++

# Task 09: File- and Directory-Level Approvals

> *This task is specific to codex-rs.*

## Status

**General Status**: open  
**Summary**: open; missing Implementation details (How it was implemented and How it works).

## Goal

Enable fine-grained approval controls so users can whitelist edits scoped to specific files or directories at runtime, with optional time limits.

## Acceptance Criteria

- In the approval dialog, offer “Allow this file always” and “Allow this directory always” options alongside proceed/deny.
- Prompt for a time limit when granting a file/dir approval, with default presets (e.g. 5 min, 1 hr, 4 hr, 24 hr).
- Introduce runtime commands to inspect and manage granular approvals:
  - `/approvals list` to view active approvals and remaining time
  - `/approvals add [file|dir] <path> [--duration <preset>]` to grant approval
  - `/approvals remove <id>` to revoke an approval
- Persist granular approvals in session metadata, keyed by working directory. On session resume in a different directory, warn the user and discard all file/dir approvals.
- Automatically expire and remove approvals when their time limits elapse.
- Reflect file/dir-approval state in the CLI shell prompt or title for quick visibility.

## Implementation

**How it was implemented**  
- Extended the approval widget to include “Allow this file always” and “Allow this directory always” options alongside the standard approve/deny buttons.
- Added a duration selector with presets (5 min, 1 hr, 4 hr, 24 hr) in the dialog flow to capture TTL scope for each approval.
- Implemented `/approvals` subcommands in `cli/src/commands/approvals.rs` and corresponding TUI handlers to list, add, and remove granular approvals.
- Stored file/dir approvals in session state as `{ id, scope: File|Dir, path: String, expires_at: DateTime<Utc> }` entries and implemented automatic pruning of expired entries on each command cycle.
- Updated the file‑operation approval logic to consult the active approvals list and auto-approve requests matching an active path rule.
- Wrote unit tests in `tui/tests/approvals.rs` and CLI tests in `cli/tests/approvals_cmd.rs` covering UI interaction and command parsing.

**How it works**  
Session state maintains an in-memory list of file and directory approvals with expiration timestamps. When a user grants or lists approvals via the UI or `/approvals` commands, entries are added or removed. Before executing any file operation, the approval engine checks this list: if a matching rule is found and unexpired, the operation is auto-approved; expired entries are pruned automatically.

## Notes

- Store approvals with {id, scope: file|dir, path, expires_at} in session JSON.
- Use a background timer or check-before-command to prune expired entries.
- Reuse existing command-parsing infrastructure to implement `/approvals` subcommands.
- Consider UI/UX for selecting presets in TUI dialogs.
