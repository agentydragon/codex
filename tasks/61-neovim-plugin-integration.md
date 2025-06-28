+++
id = "61"
title = "Neovim Plugin Integration"
status = "open"
freeform_status = ""
dependencies = []
last_updated = "2025-01-28"
+++

# Task 61: Neovim Plugin Integration

> *This task is specific to codex-rs.*

## Status

**General Status**: open  
**Summary**: Design phase complete. Implementation not started.

## Goal

Create a Neovim plugin that provides seamless integration with codex-rs, allowing developers to interact with AI coding assistance without leaving their editor. The plugin will communicate with codex via the existing `codex proto` protocol mode.

## Acceptance Criteria

- [ ] Plugin works with modern Neovim (0.9+) using Lua API
- [ ] Chat interface available as split window or floating window
- [ ] Context-aware prompts include current file, selection, and LSP diagnostics
- [ ] Streaming responses with real-time syntax highlighting
- [ ] Diff preview before applying code changes
- [ ] Core commands: `:CodexChat`, `:CodexSend`, `:CodexPrompt`
- [ ] Configurable keybindings (default `<leader>cc` for chat)
- [ ] Installation via standard plugin managers (lazy.nvim, packer, etc.)

## Implementation

**Architecture:**
- Neovim plugin spawns `codex proto` as subprocess using job API
- Communication via stdin/stdout using JSON protocol
- Pure Lua implementation for modern Neovim

**Plugin Structure:**
```
codex.nvim/
├── lua/
│   ├── codex/
│   │   ├── init.lua          # Main entry point
│   │   ├── protocol.lua      # Protocol handler
│   │   ├── ui.lua           # UI components
│   │   ├── chat.lua         # Chat window management
│   │   ├── commands.lua     # Vim commands
│   │   ├── config.lua       # Configuration
│   │   └── utils.lua        # Utilities
│   └── codex.lua            # Plugin loader
├── plugin/
│   └── codex.vim            # Vim plugin entry
├── syntax/
│   └── codex.vim            # Syntax highlighting
└── doc/
    └── codex.txt            # Help documentation
```

**Key Features to Implement:**
1. Chat buffer with custom filetype and keybindings
2. Context gathering (file, selection, git root, LSP info)
3. Streaming response handling with incremental display
4. Code block extraction and application
5. Diff preview for code changes
6. Session management and history

**Communication Protocol:**
- Use existing codex protocol (`Submission` and `EventMsg`)
- Handle async responses via Neovim's event loop
- Support interruption and session management

## Notes

- See `docs/neovim-integration-design.md` for detailed design
- Leverages existing `codex proto` mode - no server changes needed
- Consider future enhancements: Telescope integration, nvim-cmp support, multi-file refactoring

## Related

- Depends on working `codex proto` implementation
- Complements Task 32 (Embedded Neovim) which goes the opposite direction
- Could share some UI concepts with the TUI implementation