id = "56"
title = "Support DEC color-preference autodetection"
status = "open"
freeform_status = ""
dependencies = [""]
last_updated = "2024-06-11T00:00:00Z"

# Task: Support DEC color-preference autodetection

> _This task is specific to codex-rs._

## Acceptance Criteria

1. The TUI must listen for the DEC color-preference OSC sequence (`ESC [ ? 1002 l` / `ESC [ ? 1002 h`).
2. On startup (or when switching panes), if the terminal reports its preferred color mode (e.g. 8‑bit vs 24‑bit), the TUI respects that preference automatically.
3. Users can still override color mode manually via `tui.colors` config.
4. All existing color-themed widgets render correctly under both auto-detected and manually overridden modes.

## Implementation

**How it was implemented**

(To be completed by the developer.)

**How it works**

(To be completed by the developer.)

## Notes

- The DEC private mode for color-preference is defined in the VT520 manual.
- This may require enabling/reading terminal responses via the input loop.
