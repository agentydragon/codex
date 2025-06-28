#!/usr/bin/env python3
"""Queries exec history from codex-rs"""

import json
import sys
from datetime import datetime
from pathlib import Path

# Path to exec history
EXEC_HISTORY_FILE = Path.home() / ".codex" / "exec_history.jsonl"

def format_timestamp(timestamp_dict):
    """Format SystemTime to readable string"""
    if 'secs_since_epoch' in timestamp_dict:
        secs = timestamp_dict['secs_since_epoch']
        return datetime.fromtimestamp(secs).strftime('%Y-%m-%d %H:%M:%S')
    return "?"

def main():
    if not EXEC_HISTORY_FILE.exists():
        print(f"No exec history found at {EXEC_HISTORY_FILE}")
        return

    entries = []
    decoder = json.JSONDecoder()
    with open(EXEC_HISTORY_FILE, 'r') as f:
        for line in f:
            if not line.strip():
                continue
            try:
                obj, _ = decoder.raw_decode(line)
            except json.JSONDecodeError as e:
                print(f"Invalid JSON line: {line.rstrip()}", file=sys.stderr)
                try:
                    obj, _ = decoder.raw_decode(line[:e.pos])
                except json.JSONDecodeError:
                    continue
            entries.append(obj)

    if not entries:
        print("No command execution entries found")
        return

    print(f"Found {len(entries)} command execution(s):\n")

    # Filter options
    if "--approved" in sys.argv:
        entries = [e for e in entries if e.get('auto_approved') or
                   e.get('approval_decision') in ['Approved', 'ApprovedForSession']]
    elif "--denied" in sys.argv:
        entries = [e for e in entries if e.get('approval_decision') in ['Denied', 'Abort']]

    # Group by working directory and command, then count approval/denial types and execution
    stats: dict[tuple[str, str], dict[str, int]] = {}
    for e in entries:
        cwd = e.get('working_dir', 'Unknown')
        # Build the command string and escape embedded newlines
        cmd = ' '.join(e.get('command', [])).replace('\n', '\\n')
        key = (cwd, cmd)
        if key not in stats:
            stats[key] = {
                'total': 0,
                'autoapproved': 0,
                'autodenied': 0,
                'user_approved': 0,
                'user_denied': 0,
                'executed': 0,
                'sandbox_failures': 0,
                'retries': 0,
            }
        stats[key]['total'] += 1
        if e.get('auto_approved'):
            stats[key]['autoapproved'] += 1
        # user-requested approval/denial
        if e.get('approval_requested'):
            if e.get('approval_decision') in ['Approved', 'ApprovedForSession']:
                stats[key]['user_approved'] += 1
            elif e.get('approval_decision') in ['Denied', 'Abort']:
                stats[key]['user_denied'] += 1
        # auto-denied: no request, not auto-approved, but denied decision
        if not e.get('approval_requested') and not e.get('auto_approved') and e.get('approval_decision') in ['Denied', 'Abort']:
            stats[key]['autodenied'] += 1
        if e.get('execution_started'):
            stats[key]['executed'] += 1
        # Count sandbox failures by matching the error message
        if (res := e.get('execution_result')) and (err := res.get('error')):
            if err.startswith('Sandbox error'):
                stats[key]['sandbox_failures'] += 1
        # Count retries identified by retry call_id suffix
        if e.get('id', '').endswith('-retry'):
            stats[key]['retries'] += 1

    # Print grouped summary
    header = (
        f"{'Tot':>3} {'Auto✓':>6} {'Auto✗':>6} {'User✓':>6} {'User✗':>6} "
        f"{'Exec':>5} {'SBX✗':>6} {'Retry':>6} {'CWD':<30} Command"
    )
    print(header)
    for (cwd, cmd), s in sorted(stats.items(), key=lambda item: item[1]['total'], reverse=True):
        print(
            f"{s['total']:>3} {s['autoapproved']:>6} {s['autodenied']:>6} "
            f"{s['user_approved']:>6} {s['user_denied']:>6} {s['executed']:>5} "
            f"{s['sandbox_failures']:>6} {s['retries']:>6} "
            f"{cwd:<30} {cmd}"
        )

if __name__ == "__main__":
    main()
