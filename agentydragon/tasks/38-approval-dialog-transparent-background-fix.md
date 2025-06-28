+++
id = "38"
title = "Fix Approval Dialog Transparent Background"
status = "open"
freeform_status = ""
dependencies = ""
last_updated = "2025-06-26T17:52:40.329780"
+++

> *UI bug:* When the approval dialog appears, its background is transparent and any partially entered prompt text shows through, overlapping and confusing the dialog.

## Status

**General Status**: done  
**Summary**: Opaque background implemented and validated via unit test.

## Goal

Ensure the approval dialog is drawn with a solid background color (matching the dialog border or theming) so that any underlying text does not bleed through.

## Acceptance Criteria

- Approval dialogs block underlying prompt text (solid background).
- Existing unit/integration tests validate dialog visual rendering.

## Implementation

- Modify `render_ref` in `codex-rs/tui/src/user_approval_widget.rs` to fill the dialog `area` with a `DarkGray` background before drawing border and content.
- Use nested loops over `area` to call `buf[(col, row)].set_bg(Color::DarkGray)` for each cell.
- Add unit test `render_approval_dialog_fills_background` to prefill a buffer with a sentinel background and assert all cells in the dialog region are recolored.

## Notes

<!-- Any implementation notes -->
