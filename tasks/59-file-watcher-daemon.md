+++
id = "59"
title = "File watcher daemon for external changes"
status = "open"
freeform_status = "Design phase - comprehensive technical design completed"
dependencies = [] # Manager rationale: This is a standalone feature that doesn't depend on other tasks
last_updated = "2025-01-28T00:00:00Z"
+++

# Task 59: File watcher daemon for external changes

> *This task is specific to codex-rs.*

## Acceptance Criteria

1. **Change Detection**: System detects when files are modified by external tools (linters, formatters, editors, git operations)
2. **Model Notification**: Model receives clear notifications about external changes without disrupting ongoing operations
3. **Performance**: No noticeable performance degradation with typical project sizes (10k files)
4. **Filtering**: Smart filtering prevents noise from temporary files, build artifacts, and model's own changes
5. **Cross-platform**: Works on Linux, macOS, and Windows
6. **Configuration**: Users can enable/disable and configure watched paths and ignore patterns

## Implementation

**How it will be implemented**

1. **File Watching Layer**:
   - Use `notify = "6.1"` crate for cross-platform file system monitoring
   - Create `FileWatcherDaemon` struct managing watch paths and event handling
   - Track model-modified files to distinguish external vs internal changes

2. **Event Integration**:
   - Leverage existing `BackgroundEventEvent` system for non-disruptive notifications
   - Add event handling in main Codex event loop
   - Queue changes and only notify when model is about to sample (no real-time notifications needed)

3. **Smart Filtering**:
   - Configurable ignore patterns (`.git/`, `target/`, `*.swp`, etc.)
   - Distinguish binary vs text files for appropriate reporting
   - Rate limiting to prevent notification spam

4. **Configuration**:
   - Add `[file_watcher]` section to config with enable/disable, paths, patterns
   - Default to watching current directory with sensible ignores

**How it will work**

1. **Startup**: File watcher daemon initializes with configured watch paths
2. **Detection**: When external tool modifies a file, `notify` detects the change
3. **Filtering**: Daemon checks if change is external (not in model-modified set) and passes filters
4. **Queueing**: Changes are accumulated in a queue (no immediate notification)
5. **Notification**: When model is about to sample next, inject summary of all queued changes
6. **Model Response**: Model acknowledges changes and adjusts approach if needed

## Notes

### Key Design Decisions

1. **BackgroundEvent vs New Event Type**: Using existing BackgroundEvent system minimizes changes and maintains compatibility

2. **Lazy Notification Strategy**: Changes are queued and only presented when model samples, avoiding unnecessary real-time notifications

3. **Model-Modified Tracking**: Essential for distinguishing external changes from model's own edits

### Testing Scenarios

- Run `cargo fmt` while model is editing Rust files
- Git checkout/merge operations during active session
- IDE auto-save while model has file context
- Build system generating files
- Bulk refactoring tools modifying multiple files

### Future Enhancements

- Intelligent source detection (identify which tool made changes)
- Integration with LSP for smarter notifications
- Selective watching based on current model focus