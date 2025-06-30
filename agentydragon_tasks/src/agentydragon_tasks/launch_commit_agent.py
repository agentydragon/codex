#!/usr/bin/env python3
"""
launch_commit_agent.py: Run the non-interactive Commit agent for completed tasks.
"""
import subprocess
import sys
from pathlib import Path

import click
from importlib import resources

import agentydragon_tasks.prompts

from agentydragon_tasks.common import (
    repo_root,
    tasks_dir,
    worktrees_dir,
    resolve_slug,
    sandbox_flags_for_worktree,
)
from agentydragon_tasks.tasklib import find_task_file


@click.command()
@click.argument("task_inputs", nargs=-1)
def main(task_inputs):
    """Run Commit agent for completed tasks; with no arguments, select interactively."""
    # Interactive selection when no specific tasks are given
    if not task_inputs:
        candidates = []  # list of (slug, title)
        import re

        for wt in worktrees_dir().iterdir():
            if not wt.is_dir():
                continue
            slug = wt.name
            try:
                md = find_task_file(slug)
            except FileNotFoundError:
                continue
            text = md.read_text(encoding="utf-8")
            m_stat = re.search(r'status\s*=\s*"([^"]+)"', text)
            if not m_stat or m_stat.group(1) != "Done":
                continue
            m_title = re.search(r'title\s*=\s*"([^"]+)"', text)
            title = m_title.group(1) if m_title else slug
            st = subprocess.check_output(
                ["git", "status", "--porcelain"], cwd=wt, text=True
            ).strip()
            if not st:
                continue
            candidates.append((slug, title))
        if not candidates:
            click.echo("No tasks ready to commit.")
            sys.exit(0)
        click.echo("Tasks ready to commit:")
        for i, (slug, title) in enumerate(candidates, start=1):
            click.echo(f"[{i}] {slug}: {title}")
        sel = click.prompt(
            "Enter numbers to commit (e.g. '1 2'), or 'a' for all", default="a"
        )
        if sel.strip().lower() == "a":
            task_inputs = [s for s, _ in candidates]
        else:
            try:
                idxs = [int(x) for x in sel.split()]
                task_inputs = [
                    candidates[i - 1][0] for i in idxs if 1 <= i <= len(candidates)
                ]
            except Exception:
                click.echo("Invalid selection", err=True)
                sys.exit(1)
        # Launch commit agents in parallel
        procs = []
        script = Path(__file__).resolve()
        for slug in task_inputs:
            p = subprocess.Popen(
                [sys.executable, str(script), slug], cwd=str(repo_root())
            )
            procs.append((slug, p))
        for slug, p in procs:
            ret = p.wait()
            if ret != 0:
                click.echo(
                    f"Commit agent for {slug} exited with status {ret}", err=True
                )
        return

    slug = resolve_slug(task_inputs[0])
    wt = worktrees_dir() / slug
    if not wt.exists():
        click.echo(
            f"Error: worktree for '{slug}' not found; run 'tasks start-agent develop <task>' first",
            err=True,
        )
        sys.exit(1)

    # load commit prompt from package resources
    from importlib import resources

    base = resources.read_text(agentydragon_tasks.prompts, "commit.md")
    try:
        task_file = find_task_file(slug)
    except FileNotFoundError:
        click.echo(f"Error: task file not found for {slug}", err=True)
        sys.exit(1)

    # prepare temp file for commit message
    msg_file = Path(subprocess.check_output(["mktemp"]).decode().strip())
    try:
        # Abort early if no pending changes in this worktree
        status_out = subprocess.check_output(
            ["git", "status", "--porcelain"], cwd=wt, text=True
        ).strip()
        if not status_out:
            click.echo(
                f"No changes detected in worktree for '{slug}'; nothing to commit.",
                err=True,
            )
            sys.exit(0)
        # Use codex's built-in --cd flag and grant sandbox access to the worktree
        cd_arg = ["--cd", str(wt)]
        cmd = (
            ["codex"]
            + cd_arg
            + ["--full-auto", "exec", "--output-last-message", str(msg_file)]
        )
        cmd += sandbox_flags_for_worktree(wt)
        click.echo(f"Running commit agent: {' '.join(cmd)}")
        prompt_content = prompt_file.read_text(encoding="utf-8")
        task_content = task_file.read_text(encoding="utf-8")
        subprocess.check_call(
            cmd + [prompt_content + "\n\n" + task_content], stdout=subprocess.DEVNULL
        )
        # Stage all changes in this worktree, including new files (not just modifications)
        subprocess.check_call(["git", "add", "-A"], cwd=str(wt))
        try:
            subprocess.check_call(["git", "commit", "-F", str(msg_file)], cwd=str(wt))
        except subprocess.CalledProcessError as e:
            click.echo(
                f"Error running command: {' '.join(e.cmd)} returned {e.returncode}",
                err=True,
            )
            msg = msg_file.read_text(encoding="utf-8")
            click.echo("Commit message was:\n" + msg)
            sys.exit(e.returncode)
        # Print the commit message for visibility
        msg = msg_file.read_text(encoding="utf-8").strip()
        click.echo("Commit message:\n" + msg)
    finally:
        # cleanup temp file
        msg_file.unlink()


if __name__ == "__main__":
    main()
