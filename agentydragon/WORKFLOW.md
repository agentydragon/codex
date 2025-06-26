 # Agent Handoff Workflow

 This document explains the multi-agent handoff pattern used for task development and commits
 in the `agentydragon` workspace. It consolidates shared guidance so individual agent prompts
 do not need to repeat these details.

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
 - **Actions**:
   1. Stage all changes: `git add -u`.
   2. Run `pre-commit run --files $(git diff --name-only --cached)`.
   3. Read the commit message and run `git commit -m "$MSG"`.

## 4. Merge Conflict Resolution Agent
- **Scope**: Runs inside the task’s worktree when preparing a completed task branch for integration.
- **Actions**:
  1. If merging `agentydragon-<ID>-<slug>` into `agentydragon` fails due to conflicts, launch the Merge Conflict Resolution agent in that task’s worktree.
  2. Agent reads the `prompts/merge-conflict-fix.md` template and the additional branch-specific instructions.
  3. Agent resolves all merge conflicts so that the branch can be cleanly merged into `agentydragon`, stages the resolutions, and commits the merge-resolution commit.
  4. Stop after creating the clean merge-resolution commit; do not modify unrelated files or push changes.

## 5. Status & Launch
 - Use `agentydragon_task.py status` to view tasks (including those in `.done/`).
 - Summaries:
   - **Merged:** tasks with no branch/worktree.
   - **Ready to merge:** tasks marked Done with branch commits ahead.
   - **Unblocked:** tasks with no outstanding dependencies.
- The script also prints a `agentydragon/tools/create_task_worktree.py --agent --tmux <IDs>` command for all unblocked tasks.

This guide centralizes the handoff workflow for all agents.
