+++
id = "42"
title = "Inspect Env CLI command returns empty output on Linux and macOS"
status = "open"
freeform_status = ""
dependencies = [] # Manager rationale: fix inspect-env bug on Linux and macOS; independent task
last_updated = "2025-06-28T02:09:48Z"
> *This task is specific to codex-rs.*

# Task 42: Inspect Env CLI command returns empty output on Linux and macOS

The `inspect-env` command currently produces no output when run on Linux or macOS systems. It should report environment variables as well as details of mounted volumes, including mount paths, permissions, and allowed actions.
