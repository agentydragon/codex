+++
id = "03"
title = "Live Config Reload and Prompt on Changes"
status = "open"
dependencies = "02,07,09,11,14,29"
last_updated = "2025-06-25T05:36:17.783726"
+++

# Task 03: Live Config Reload and Prompt on Changes

> *This task is specific to codex-rs.*

## Status

Implementation not started

## Goal
Detect changes to `config.toml` while a session is running and prompt the user to apply or ignore the updated settings.

## Acceptance Criteria
- A background file watcher watches `$CODEX_HOME/config.toml` (or active user config path).
- On any write event, compute a diff between the in-memory config and the on-disk file. This diff will be on the level of the *active configuration Rust structure*, **NOT** on the level of the text file.
- Pause the agent, display the diff in the TUI bottom pane, and offer two actions: `Apply new config` or `Continue with old config`.
- If the user applies, re-parse the config, merge overrides, and resume using the new settings. Otherwise, discard changes and resume.
- Make sure that if config update prompt happens it allows things to continue! In particular:
- If config reload pops up while an approval prompt is visible, and then config reload is dismissed, user should be returned back into approval prompt and must be able to confirm it and agent must continue operating.

## Implementation notes

## Notes
- Leverage a crate such as `notify` for FS events
