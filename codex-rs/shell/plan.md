# Implementation Plan for `codex-shell`

Below is the step-by-step TODO list to implement the inline append-mode shell.

- [x] **CLI & crate setup**: wire `TopCli` → parse overrides → call `run_main(Cli, sandbox_exe)` in a new `codex-shell` binary.
- [x] **Config loading**: load configuration (including shell-specific styles/spinners/collapse rules) via `Config::load_with_cli_overrides`, under a `[shell]` config section.
- [x] **Startup banner & title**: print banner lines with `println!` (no alt-screen/raw-mode) and set the terminal title (OSC) to include session ID.
- [ ] **Event loop**: initialize Codex client and hooks (`init_codex`), spawn tasks for model events and pre/post-command hooks, unify events into a single channel.
- [ ] **Inline history + prompt redraw**: for each event/response, `println!` the formatted block to stdout (append-only), then re-render only the bottom prompt (via ANSI cursor sequences or minimal ratatui).
- [ ] **Shell vs Message mode**: capture <Ctrl‑X> to toggle shell-command vs text-message mode, updating prompt icon/color.
- [ ] **Spinners & icons**: display a spinner glyph for in-progress commands/model sampling, replace with success/failure icons on completion; all styles configurable.
- [ ] **Queued-message buffer**: while a command is running, accept user input into a queued buffer, show queued draft lines above the prompt, without injecting into history.
- [ ] **Queued-message insertion**: on send or auto-approval, insert queued messages into history at the actual send position and remove the queued placeholder.
- [ ] **Approval panel**: render approval requests in a panel immediately above the prompt without displacing the prompt.
- [ ] **Auto-approved collapse**: collapse output of commands auto-approved by the model into one summary line; collapse patch diffs into `+<added> -<deleted>` counts; apply syntax coloring only to patches/markdown.
- [ ] **Unit tests for rendering**: write unit tests feeding example event sequences (denied exec, auto-approved hook, tool failure, reasoning, text, shell-command) into the renderer and assert ANSI output correctness.
- [ ] **Interactive shell commands via PTY**: spawn shell-command inputs in a pseudo-tty so stdin/stdout are interactive; ensure sensitive prompts (e.g. sudo password) are read directly and not captured or sent to the model.
- [ ] **Slash-commands**: implement `/inspect-env` and `/edit` inline handlers printing output to stdout and returning focus to prompt.
