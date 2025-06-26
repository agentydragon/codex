+++
id = "17"
title = "Sandbox Pre-commit Permission Error"
status = "open"
freeform_status = ""
dependencies = "15" # Rationale: depends on Task 15 for sandbox worktree configuration
last_updated = "2025-06-25T01:41:34.737190"
+++

> *This task addresses scaffolding/setup for Agent worktrees.*

## Acceptance Criteria

- Pre-commit hooks detect sandbox environment and skip or override gitconfig locking.
- Documentation in scaffold guides is updated to note pre-commit limitations and workarounds.
- Verification steps demonstrate pre-commit hooks succeeding in sandbox without modifying user gitconfig.

## Implementation

**How it was implemented**  
- Configured pre-commit invocation in `scaffold/setup.rs` and `tools/create_task_worktree.py` to set the `PRE_COMMIT_HOME` environment variable to a workspace-local directory.
- Updated pre-commit hook invocation commands to include `--config .pre-commit-config.yaml --repo .` flags, preventing writes to `~/.gitconfig`.
- Modified documentation in `WORKFLOW.md` and `scaffold/README.md` to describe the sandbox pre-commit workflow and troubleshooting steps.
- Added integration tests in `scaffold/tests/precommit_sandbox.rs` invoking `pre-commit run --all-files` inside a sandboxed worktree to assert no `PermissionError` occurs.

**How it works**  
When scaffolding a new worktree or rerunning pre-commit hooks in a sandbox, the environment is adjusted so that pre-commit uses a repo-local config directory instead of the user's home, avoiding permission errors. Documentation updates guide maintainers through the sandboxed pre-commit workflow.

## Notes

- The sandbox prevents locking ~/.gitconfig, leading to PermissionError.
- Consider configuring pre-commit to use a repo-local config or skip locking by passing `--config` or setting `PRE_COMMIT_HOME`.
