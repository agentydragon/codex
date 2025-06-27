#!/usr/bin/env python3
"""
TEMPORARY ONE-OFF to query exec history from codex-rs

Can delete after: Built-in query functionality is sufficient
"""

import json
import os
import sys
from pathlib import Path
from datetime import datetime

# Path to exec history
EXEC_HISTORY_FILE = Path.home() / ".codex" / "exec_history.jsonl"

def format_timestamp(timestamp_dict):
    """Format SystemTime to readable string"""
    if 'secs_since_epoch' in timestamp_dict:
        secs = timestamp_dict['secs_since_epoch']
        return datetime.fromtimestamp(secs).strftime('%Y-%m-%d %H:%M:%S')
    return "Unknown"

def main():
    if not EXEC_HISTORY_FILE.exists():
        print(f"No exec history found at {EXEC_HISTORY_FILE}")
        return
    
    entries = []
    with open(EXEC_HISTORY_FILE, 'r') as f:
        for line in f:
            if line.strip():
                try:
                    entry = json.loads(line)
                    entries.append(entry)
                except json.JSONDecodeError as e:
                    print(f"Error parsing line: {e}")
    
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
    
    # Display entries
    for entry in entries:
        print(f"{'='*60}")
        print(f"Time:     {format_timestamp(entry.get('timestamp', {}))}")
        print(f"Command:  {' '.join(entry.get('command', []))}")
        print(f"Dir:      {entry.get('working_dir', 'Unknown')}")
        
        if entry.get('approval_requested'):
            decision = entry.get('approval_decision', 'Pending')
            print(f"Approval: {decision}")
        elif entry.get('auto_approved'):
            print(f"Approval: Auto-approved")
        
        if entry.get('execution_started'):
            result = entry.get('execution_result', {})
            if result:
                exit_code = result.get('exit_code', 'N/A')
                duration = result.get('duration_ms', 0) / 1000
                print(f"Result:   Exit code {exit_code} ({duration:.2f}s)")
                if result.get('error'):
                    print(f"Error:    {result['error']}")
        else:
            print(f"Result:   Not executed")
    
    print(f"\n{'='*60}")
    print(f"Total: {len(entries)} commands")
    
    # Summary stats
    approved = sum(1 for e in entries if e.get('auto_approved') or 
                   e.get('approval_decision') in ['Approved', 'ApprovedForSession'])
    denied = sum(1 for e in entries if e.get('approval_decision') in ['Denied', 'Abort'])
    executed = sum(1 for e in entries if e.get('execution_started'))
    
    print(f"Approved: {approved}, Denied: {denied}, Executed: {executed}")

if __name__ == "__main__":
    main()