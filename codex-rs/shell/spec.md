# Codex Shell: Inline Append Mode Spec

## Summary

Provide a standalone `codex-shell` binary that runs in the normal terminal buffer and implements an inline, append-only chat UI with native scrollback.  It merges features from non-fullscreen scrollback, inline patch rendering, hooks, approval UI, event layering, draft-focus preservation, title updates, context display, interactive prompts, container inspection, and editor integration.

## Detail

The shell mode must behave like a regular shell session:

- **Append-only history**: every user input or model response is printed via `println!` so it lands in the terminal’s scrollback.  No alternate-screen or full-screen clears.
- **Bottom-only redraw**: ratatui (if used) is restricted to re-rendering just the prompt line(s) at the bottom; history above is never cleared or redrawn.
- **Inline patch/markdown**: diffs and markdown must render inline with syntax highlighting.
- **Codex-rs hooks**: pre- and post-command hooks inject events into a unified event layer.
- **Approval panel**: approval requests appear in a panel immediately above the compose prompt, without moving the prompt.
- **Event-layer consolidation**: all lifecycle events (hooks, approvals, logs) use a single, consistent display layer.
- **Draft-focus preservation**: user draft input never loses focus or cursor position across redraws and command output.
- **Terminal title updates**: update the terminal title bar with the session identifier.
- **Context-remaining display**: show the percentage of context left in the prompt.
- **Interactive execution prompts**: while commands run, the prompt stays visible and accepts queued messages shown above it.
- **Queued-message semantics**: messages typed during execution display as “queued” above the prompt and only insert into the conversation at the actual send position.
- **Shell vs Message mode**: user can toggle between shell-command mode and text-message mode with <Ctrl-X>.  The prompt’s color/icon updates to reflect current mode.
- **Inspect-env integration**: support an `/inspect-env` command to emit container state inline.
- **External-editor prompt**: support an `/edit` command that launches the external editor inline and returns the result to the draft.
- **Queued-message editing**: when the prompt is empty and the user presses ↑, dequeue the last queued message back into the prompt edit buffer for revision.
- **Auto-approved command collapsing**: collapse output of any command auto-approved by the model into a single summary line; similarly collapse any completed patch block into `+<added> -<deleted>` counts (omit zero counts).  Collapse styling and summary formatting must be configurable; apply syntax coloring only to patch summaries and markdown output from Codex, not to arbitrary command output.
- **Progress spinners**: show a spinner icon for commands in progress and model sampling in progress; once complete, replace spinner with success/failure icons as in the TUI spec.  Spinner and icon styles must be configurable.

## Acceptance Criteria

- Launching `codex-shell` does not switch to the alternate-screen or clear existing buffer.
- All history lines append to stdout and appear in the terminal scrollback.
- Diffs and markdown content render inline with correct styling.
- Hook events appear in a unified event layer.
- Approval requests render above the prompt without displacing it.
- User draft focus and cursor position persist across all events.
- Terminal title contains the session ID.
- Prompt shows real-time context-remaining percentage.
- Prompt remains interactive during long-running commands; queued messages are shown above.
- Queued messages are only added to the conversation when actually sent, at the correct chronological point.
- `/inspect-env` prints container state inline.
- `/edit` opens the external editor, then returns to prompt with updated draft.
- Upon approving or rejecting a patch diff, the displayed diff block is removed from the UI.
- <Ctrl-X> toggles shell-command vs text-message mode; prompt color/icon changes to indicate mode.
- Pressing ↑ on an empty prompt restores the last queued message into the prompt for editing.

## Implementation Sketch

1. Parse config overrides and initialize codex client as in `codex-tui`.
2. In `run_main`, print banner lines via `println!` instead of switching to alt-screen.
3. Pipe codex events and hook events to stdout as they arrive, formatting inline.
4. After each event or response, re-render only the prompt widget at bottom, using ratatui or manual cursor movement.
5. Manage a queue buffer for user inputs during command execution; display queued items inline above prompt.
6. Wire approval requests to render a minimal panel above the prompt.
7. Handle `/inspect-env` and `/edit` commands inline, emitting output via stdout.
8. Add unit tests for history rendering: feed example event sequences (denied exec, auto-approved hook, tool failure, reasoning, text, shell-command) into the renderer and assert ANSI-formatted output contains the expected icons, summaries, and order.
9. On shell-command inputs, spawn the command in a pseudo-tty (pty) so stdin/stdout are live-interactive and displayed inline; ensure sensitive prompts (e.g. sudo password) are read directly and not captured or sent to Codex.

## Notes

This spec combines requirements from tasks 6, 10, 13, 20, 30, 31, 34, 48, 50, 51, 58, 62.
