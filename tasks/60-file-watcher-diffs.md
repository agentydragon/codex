+++
id = "60"
title = "Show diffs for external file modifications"
status = "open"
freeform_status = "Depends on file watcher daemon implementation"
dependencies = [59] # Manager rationale: Requires the base file watcher daemon to be implemented first
last_updated = "2025-01-28T00:00:00Z"
+++

# Task 60: Show diffs for external file modifications

> *This task is specific to codex-rs.*

## Acceptance Criteria

1. **Diff Generation**: When external changes are detected on text files, generate concise diffs
2. **Size Limits**: Only show diffs for reasonably sized changes (configurable threshold)
3. **Format**: Present diffs in a readable format that helps model understand what changed
4. **Performance**: Diff generation doesn't significantly slow down notification delivery
5. **Configuration**: Users can enable/disable diff generation and configure size thresholds

## Implementation

**How it will be implemented**

1. **Diff Generation**:
   - Store file snapshots when model last accessed them
   - Use a diff library (e.g., `similar` crate) to generate unified diffs
   - Apply smart truncation for large diffs

2. **Notification Enhancement**:
   - Extend file change notifications to include diff information
   - Format: "External change: src/main.rs modified (+10/-5 lines)" followed by diff
   - Include context lines for better understanding

3. **Performance Optimization**:
   - Only generate diffs for files under size threshold (default: 100KB)
   - Cache file contents efficiently
   - Generate diffs lazily only when notification is about to be sent

**How it will work**

1. **Baseline Capture**: When model reads/modifies a file, capture its content as baseline
2. **Change Detection**: File watcher detects external modification
3. **Diff Generation**: Compare current content with baseline, generate unified diff
4. **Smart Formatting**: Truncate large diffs, highlight key changes
5. **Enhanced Notification**: Include diff in the change notification to model

## Notes

### Example Notification Format

```
External modifications detected:

src/config.rs - Modified by rustfmt
  +5/-3 lines changed
  
  @@ -45,7 +45,9 @@
  -    let config = Config::new().unwrap();
  +    let config = Config::new()
  +        .expect("Failed to load config");
   
  -    process_data(&config,data);
  +    process_data(&config, data);
```

### Configuration Options

```toml
[file_watcher.diffs]
enabled = true
max_file_size = 102400  # 100KB
max_diff_lines = 50
context_lines = 3
```

### Challenges

- Memory usage for storing file baselines
- Handling binary files that were misidentified as text
- Dealing with very large diffs (e.g., formatted JSON files)