id = "57"
title = "Fix broken code-block rendering in assistant final messages"
status = "open"
dependencies = ["56"]
last_updated = "2024-06-11T00:00:00Z"

# Task: Fix broken code-block rendering in assistant final messages

> _This task is specific to codex-tui._

## Acceptance Criteria

1. Multi-line markdown code fences in the assistant's final message pane render correctly with proper styling and indentation.
2. The first line inside a code block is formatted the same as subsequent lines.
3. Existing markdown features (e.g., blockquotes, lists) continue to render correctly.

## Implementation

**How it was implemented**

(To be completed by the developer.)

**How it works**

(To be completed by the developer.)

## Notes

- The issue appears only for code fences in the final assistant message area (bottom pane) of the TUI.
- Other markdown elements and the thinking blocks render as expected.
- Likely related to the markdown-to-ANSI converter or the widget rendering logic skipping the first line formatting.
