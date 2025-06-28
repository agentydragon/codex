# TODO

- [x] Update README.md: replace `[tui.colors]` with `[tui.styles]`, document full style syntax (modifiers, fg=, bg=) and defaults.
- [x] Update docs/config.md: rename `tui.colors` section to `tui.styles` and update examples and supported syntax.
- [x] Revise `core/src/config_types.rs`: update comments and serde docs for style-based config keys.
- Refactor TUI code: replace `parse_color` usages with `parse_style` for style specs in all widgets (e.g., popup_bg, borders, labels).
- [ ] Clean up/remove `tui/src/color.rs`.
- Add or update tests for `parse_style` and style-based config loading.
- Update CLI exec flag/docs (`--color`) if migrating to full style support.
- Deprecate legacy color-only config fields and remove related code.
- Proposed patches still prompt to "run this command" and the "always allow writing" option does not appear.
- [ ] Config-changed popover destroys open approval dialog; session cannot continue afterwards.
- Add style config for standard (non-selected) text in approval dialog.
