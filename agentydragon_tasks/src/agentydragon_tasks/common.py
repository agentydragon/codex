#!/usr/bin/env python3
"""
common.py: Shared utilities for agentydragon tooling scripts.
"""
import subprocess
from pathlib import Path

import click


def repo_root() -> Path:
    """Return the Git repository root directory."""
    out = subprocess.check_output(["git", "rev-parse", "--show-toplevel"])
    return Path(out.decode().strip())


def tasks_dir() -> Path:
    """Path to the tasks directory in the repo root."""
    return repo_root() / "tasks"


def worktrees_dir() -> Path:
    """Path to the .worktrees directory under tasks."""
    return tasks_dir() / ".worktrees"


def resolve_slug(input_id: str) -> str:
    """Resolve a two-digit task ID into its full slug, or return slug unchanged."""
    if input_id.isdigit() and len(input_id) == 2:
        matches = list(tasks_dir().glob(f"{input_id}-*.md"))
        if len(matches) == 1:
            return matches[0].stem
        raise ValueError(
            f"Expected one task file for ID {input_id}, found {len(matches)}"
        )
    return input_id


def sandbox_flags_for_worktree(worktree: Path) -> list[str]:
    """
    Return sandbox-exec flags granting read/write access to a worktree and its Git data.

    git worktrees store metadata outside the worktree directory (via a .git "gitdir" file),
    so we must also explicitly whitelist the main repo's .git.
    """
    gitdir = repo_root() / ".git"
    return [
        "-s",
        f"disk-read-folder={worktree}",
        "-s",
        f"disk-read-folder={gitdir}",
        "-s",
        f"disk-write-folder={worktree}",
        "-s",
        f"disk-write-folder={gitdir}",
    ]


def task_branch(slug: str) -> str:
    """Return the Git branch name for the given task slug."""
    return f"agentydragon-{slug}"


def list_task_branches() -> list[str]:
    """Return a list of all Git branches for tasks (agentydragon-*)"""
    out = subprocess.check_output(
        [
            "git",
            "for-each-ref",
            "--format=%(refname:short)",
            "refs/heads/agentydragon-*",
        ],
        cwd=repo_root(),
        text=True,
    )
    return [b for b in out.strip().splitlines() if b]


# Name of the integration branch into which tasks are merged
INTEGRATION_BRANCH = "agentydragon"


def run_codex_exec(
    worktree: Path, prompt: str, full_auto: bool = True, exec_mode: bool = True
) -> None:
    """Run a Codex session in the given worktree with the specified mode and prompt."""
    cmd = ["codex"]
    if full_auto:
        cmd.append("--full-auto")
    if exec_mode:
        cmd.append("exec")
    cmd += ["--cd", str(worktree)]
    cmd += sandbox_flags_for_worktree(worktree)
    click.echo(f"Running Codex: {' '.join(cmd)}")
    subprocess.check_call(cmd + [prompt])


def run_git(args: list[str], cwd: Path | None = None) -> None:
    """Run a git command at the repo root or specified directory."""
    cwd_path = cwd or repo_root()
    subprocess.check_call(["git", *args], cwd=str(cwd_path))
