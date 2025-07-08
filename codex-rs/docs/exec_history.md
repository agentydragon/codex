# Execution History

This document describes the implementation of command execution history logging in codex-rs, enabling an audit trail for commands that required approval.

## Core Module: `exec_history.rs`

- Location: `core/src/exec_history.rs`
- Provides structured logging to `~/.codex/exec_history.jsonl`
- Tracks:
  - Command and working directory
  - Approval status (requested, auto-approved, user decision)
  - Execution results (exit code, duration, errors)
  - Session ID and timestamps

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

## Design Decisions

1. **Append-only log**: Each stage creates a new entry rather than updating.
2. **Session tracking**: Links commands to their session for context.
3. **No data loss**: Records all approval requests, even if denied.
4. **SystemTime**: Uses Rust's SystemTime for portability.
