# Exec History Implementation for Codex-RS

## Summary

This implementation adds comprehensive command execution history logging to codex-rs, addressing the lack of audit trail for commands that required approval.

## What Was Added

### 1. Core Module: `exec_history.rs`
- Location: `core/src/exec_history.rs`
- Provides structured logging to `~/.codex/exec_history.jsonl`
- Tracks:
  - Command and working directory
  - Approval status (requested, auto-approved, user decision)
  - Execution results (exit code, duration, errors)
  - Session ID and timestamps

### 2. Integration Points
- **Session struct**: Added `exec_history` field
- **Command flow**: Logs at each stage:
  - When approval is requested
  - When approval decision is made
  - When execution starts
  - When execution completes

### 3. UI Component
- **Slash command**: `/exec-history` to view history
- **Bottom pane view**: Table with filtering options
  - Press 'a' to filter approved commands
  - Press 'd' to filter denied commands
  - Press 'r' to refresh
  - Press 'q' to close

### 4. Query Script
- `oneoff__query_exec_history.py`: Command-line tool to query history
- Supports `--approved` and `--denied` filters
- Shows summary statistics

## File Format

Each line in `exec_history.jsonl` is a JSON object:
```json
{
  "id": "call-123",
  "session_id": "uuid",
  "timestamp": {"secs_since_epoch": 1234567890, "nanos_since_epoch": 0},
  "command": ["ls", "-la"],
  "working_dir": "/home/user",
  "approval_requested": true,
  "approval_decision": "Approved",
  "auto_approved": false,
  "execution_started": true,
  "execution_result": {
    "exit_code": 0,
    "duration_ms": 123,
    "error": null
  }
}
```

## Key Design Decisions

1. **Append-only log**: Each stage creates a new entry rather than updating
2. **Session tracking**: Links commands to their session for context
3. **No data loss**: Records all approval requests, even if denied
4. **SystemTime**: Uses Rust's SystemTime for portability

## Future Improvements

1. Add query filters by date range
2. Export to different formats (CSV, etc.)
3. Integrate with existing audit systems
4. Add command pattern analysis
5. Implement log rotation