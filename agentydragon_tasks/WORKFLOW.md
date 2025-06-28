# Agent Handoff Workflow

This document explains the multi-agent handoff pattern used for task development and commits
in the `agentydragon` workspace. It consolidates shared guidance so individual agent prompts
do not need to repeat these details.

This document does not include tasks intended to be run by the human controlling the system,
which go into `README.md`.

## 1. Developer Agent
- **Scope**: Runs inside a sandboxed git worktree for a single task branch (`agentydragon-<ID>-<slug>`).
- **Actions**:
  1. If the task’s **Status** is `Needs input`, stop immediately and await further instructions; do **not** implement code changes or run pre-commit hooks.
  2. Update the task Markdown file’s **Status** to `Done` when implementation is complete.
  3. Implement the code changes for the task.
  4. Run `pre-commit run --files $(git diff --name-only)` to apply and stage any autofix changes.
  5. **Do not** run `git commit`.

## 2. Commit Agent
- **Scope**: Runs in the sandbox (read-only `.git`) or equivalent environment.
- **Actions**:
  1. Emit exactly one line to stdout: the commit message prefixed `agentydragon(tasks): `
     summarizing the task’s **Implementation** section.
  2. Stop immediately.

## 3. Orchestrator
- **Scope**: Outside the sandbox with full Git permissions.
  **Actions**:
  1. Stage all changes: `git add -u`.
  2. Run `pre-commit run --files $(git diff --name-only --cached)`.
  3. Read the commit message and run `git commit -m "$MSG"`.
  4. For each task branch marked Done with commits ahead:
     a. Dry-run merge using `git merge-tree` to detect conflicts without modifying the worktree.
     b. If clean, merge the branch into `agentydragon` using `git merge --no-ff`.
     c. If conflicts are detected, launch the Merge Conflict Resolution Agent (see below), then re-run the dry-run; if still conflicted, skip merging.
  5. Optionally remove merged branches and worktrees for disposed tasks.

## 4. Rebase Branch Agent

- **Scope**: Runs inside the task’s worktree with explicit Git-write permission on `.git`.
- **Actions**:
  1. Launch the Rebase Branch agent (via `tasks start-agent rebase <task-slug|NN>`).
2. Agent runs these commands to preserve and reapply uncommitted changes:
   ```bash
   git diff > changes.patch
   git rebase agentydragon
   git apply changes.patch
   ```
   If the rebase or patch application fails, abort and restore branch state:
   ```bash
   git rebase --abort
   git reset --hard
   exit 1
   ```
3. Agent edits files to resolve remaining conflicts; remove conflict markers (`<<<<<<<`, `=======`, `>>>>>>>`).
4. Agent runs pre-commit checks and applies autofixes so that all hooks pass. If any hooks still fail after autofix, fix the issues manually until pre-commit succeeds:
   ```bash
   pre-commit run --all-files
   ```
5. Agent leaves the worktree dirty (conflicts resolved, hooks passed)—do **not** stage or commit; the orchestrator will finish the rebase.

## 5. Merge Conflict Resolution Agent
- **Scope**: Runs inside the task’s worktree when preparing a completed task branch for integration.
- **Actions**:
  1. If merging `agentydragon-<ID>-<slug>` into `agentydragon` fails due to conflicts, launch the Merge Conflict Resolution agent in that task’s worktree.
  2. Agent reads the `prompts/merge-conflict-fix.md` template and the additional branch-specific instructions.
  3. Agent resolves all merge conflicts so that the branch can be cleanly merged into `agentydragon`, stages the resolutions, and commits the merge-resolution commit.
  4. Stop after creating the clean merge-resolution commit; do not modify unrelated files or push changes.

This guide centralizes the handoff workflow for all agents.
