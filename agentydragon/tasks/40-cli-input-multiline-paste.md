+++
id = "40"
title = "Support Multiline Paste in codex-rs CLI Input Window"
status = "open"
freeform_status = ""
dependencies = []
last_updated = "2025-06-26T06:47:01.458158"
+++

# Task 40: Support Multiline Paste in codex-rs CLI Input Window

> *This task is specific to codex-rs.*

## Acceptance Criteria

- When pasting multiline text into the codex-rs CLI input (REPL), newlines in the pasted text are inserted into the input buffer rather than causing premature command execution.
- The pasted content preserves original end-of-line characters and spacing.
- The user can still press Enter to submit the complete command when desired.
- Behavior for single-line input and manual line breaks remains unchanged.

## Implementation
**How it was implemented**  
- Enabled bracketed paste mode in the line-editing library (`rustyline`) by setting `enable_bracketed_paste(true)` during CLI initialization.
- Updated input event handling in `cli/src/repl.rs` to detect bracketed-paste start/end sequences and insert newlines into the buffer rather than submitting the command prematurely.
- Added unit tests in `cli/tests/multiline_paste.rs` that simulate bracketed paste sequences and verify that the full multi-line content appears in the input buffer for editing.
- Updated documentation in `README.md` to mention bracketed paste support and any terminal prerequisites.

**How it works**  
When the user pastes multi-line text, the terminal emits bracketed paste control sequences. The CLI detects these sequences and temporarily suspends command submission until the paste ends, inserting all newline characters into the input buffer. The user can then review or edit the multi-line input and press Enter once to submit the complete text.

## Notes

- Investigate enabling bracketed paste support in the line-editing library used (e.g. rustyline, liner).
- Ensure that bracketed paste mode is enabled when initializing the CLI to distinguish between pasted content and typed input.
- Review how other REPLs implement multiline paste handling to inform the design.
