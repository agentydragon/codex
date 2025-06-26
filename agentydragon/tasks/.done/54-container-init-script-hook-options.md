+++
id = "54"
title = "Container init script hook options"
status = "merged"
freeform_status = ""
dependencies = "43"
last_updated = "2025-06-26T18:55:28.815413"
+++

# Task 54: Container init script hook options

# Add support for custom hooks or scripts to run during container initialization, allowing users to specify commands (e.g. workarounds for pyenv shim rehash errors) that execute before in-container command execution.

## Implementation

**How it was implemented**  
- Extended `run_in_container.sh` to accept a repeatable `--init-hook` flag, collecting each provided hook command in an `INIT_HOOKS` array.  
- Updated the usage header to document `--init-hook`.  
- After firewall initialization and permissions setup, the script iterates over `INIT_HOOKS` and executes each hook inside the container as root via `bash -lc`.  
- Added argument validation to error if `--init-hook` is provided without a command.

**How it works**  
- Users can supply one or more `--init-hook "<command>"` flags before the main command.  
- Each hook command runs in the container context before the main user command executes.
