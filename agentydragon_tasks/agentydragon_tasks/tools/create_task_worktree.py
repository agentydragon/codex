#!/usr/bin/env python3
"""
create_task_worktree.py: Create or reuse a git worktree for a specific task and optionally launch a Developer Codex agent.
"""
import re
import shlex
import shutil
import subprocess
import sys
from pathlib import Path

import click
from common import (
    repo_root,
    resolve_slug,
    tasks_dir,
    worktrees_dir,
    run_codex_exec,
)


def run(cmd, cwd=None):
    click.echo(f"Running: {' '.join(cmd)}")
    subprocess.check_call(cmd, cwd=cwd)


def resolve_slug(input_id: str) -> str:
    if input_id.isdigit() and len(input_id) == 2:
        matches = list(tasks_dir().glob(f"{input_id}-*.md"))
        if len(matches) == 1:
            return matches[0].stem
        click.echo(
            f"Error: expected one task file for ID {input_id}, found {len(matches)}",
            err=True,
        )
        sys.exit(1)
    return input_id


@click.command()
@click.option(
    "-a",
    "--agent",
    is_flag=True,
    help="Launch Developer Codex agent after setting up worktree.",
)
@click.option(
    "-t",
    "--tmux",
    "tmux_mode",
    is_flag=True,
    help="Open each task in its own tmux pane; implies --agent. "
    "Attaches to an existing session if already running.",
)
@click.option(
    "--hold",
    "hold_shell",
    is_flag=True,
    help="Keep a shell open in each tmux pane after the agent run finishes.",
)
@click.option(
    "-i",
    "--interactive",
    is_flag=True,
    help="Run agent in interactive mode (no exec); implies --agent.",
)
@click.option(
    "-s",
    "--shell",
    "shell_mode",
    is_flag=True,
    help="Launch an interactive Codex shell (skip exec and auto-commit); implies --agent and --interactive.",
)
@click.option(
    "--skip-presubmit",
    is_flag=True,
    help="Skip the initial presubmit pre-commit checks when creating a new worktree.",
)
@click.option(
    "--rebase",
    "rebase_mode",
    is_flag=True,
    help="Launch Rebase agent to update the branch onto the latest integration branch; implies --agent and --interactive.",
)
@click.argument("task_inputs", nargs=-1, required=True)
def main(
    agent,
    tmux_mode,
    hold_shell,
    interactive,
    shell_mode,
    skip_presubmit,
    rebase_mode,
    task_inputs,
):
    """Create/reuse a task worktree and optionally launch a Dev or Rebase agent or tmux session."""
    # shell mode implies interactive (skip exec within the worktree)
    if shell_mode:
        interactive = True
    if interactive or shell_mode or rebase_mode:
        agent = True

    if tmux_mode:
        agent = True
        session = "agentydragon_" + "_".join(task_inputs)
        # Attach if session already exists
        if subprocess.call(["tmux", "has-session", "-t", session]) == 0:
            click.echo(f"Session {session} already exists; attaching")
            run(["tmux", "attach", "-t", session])
            return
        # Launch panes for each task; optionally hold a shell after agent run
        for idx, inp in enumerate(task_inputs):
            slug = resolve_slug(inp)
            base_cmd = [sys.executable, "-u", __file__]
            if rebase_mode:
                base_cmd.append("--rebase")
            elif agent:
                base_cmd.append("--agent")
            base_cmd.append(slug)
            if hold_shell:
                pane_cmd = ["bash", "-lc", shlex.join(base_cmd) + "; exec $SHELL"]
            else:
                pane_cmd = base_cmd
            if idx == 0:
                run(["tmux", "new-session", "-d", "-s", session] + pane_cmd)
            else:
                run(["tmux", "new-window", "-t", session] + pane_cmd)
        run(["tmux", "attach", "-t", session])
        return

    # Single task
    slug = resolve_slug(task_inputs[0])
    branch = f"agentydragon-{slug}"
    wt_root = worktrees_dir()
    wt_path = wt_root / slug

    # Ensure branch exists
    if (
        subprocess.call(
            ["git", "show-ref", "--verify", "--quiet", f"refs/heads/{branch}"]
        )
        != 0
    ):
        run(["git", "branch", "--track", branch, "agentydragon"])

    wt_root.mkdir(parents=True, exist_ok=True)
    new_wt = False
    if not wt_path.exists():
        # --- COW hydration logic via rsync ---
        # Instead of checking out files normally, register the worktree empty and then
        # perform a filesystem-level hydration via rsync (with reflink if supported) for
        # near-instant setup while excluding VCS metadata and other worktrees.
        run(["git", "worktree", "add", "--no-checkout", str(wt_path), branch])
        src = str(repo_root())
        dst = str(wt_path)
        # Hydrate via CoW copy if possible, excluding the .worktrees directory; fallback to rsync
        cp_cmd = None
        if shutil.which("cp") and sys.platform != "darwin":
            # Copy all top-level entries except .worktrees and .git to avoid recursion and copying VCS metadata.
            # Note: in a git worktree, .git is a gitfile (not a directory) created by 'git worktree add'
            # that points to the main repo's worktrees metadata, so we skip it to preserve the link.
            base = Path(src)
            entries = [
                str(base / p.name)
                for p in base.iterdir()
                if p.name not in (".worktrees", ".git")
            ]
            cp_cmd = ["cp", "--archive", "--reflink=auto"] + entries + [dst]
            try:
                run(cp_cmd)
            except subprocess.CalledProcessError:
                cp_cmd = None
        if not cp_cmd:
            rsync_cmd = [
                "rsync",
                "-a",
                "--delete",
                # Do not include .git, but *do* include .gitignore, .github etc.
                "--exclude=.git/",
                "--exclude=.worktrees/",
                f"{src}/",
                f"{dst}/",
            ]
            run(rsync_cmd)
        # Guard against nested worktrees in the new worktree (avoid runaway recursion)
        nested = list(wt_path.rglob(".worktrees"))
        if nested:
            click.echo(
                "Error: nested .worktrees directory detected after hydration; aborting",
                err=True,
            )
            sys.exit(1)
        # Install pre-commit hooks in the new worktree
        if shutil.which("pre-commit"):
            run(["pre-commit", "install"], cwd=dst)
        else:
            click.echo("Warning: pre-commit not found; skipping hook install", err=True)
        new_wt = True
    else:
        click.echo(f"Worktree already exists at {wt_path}")

    git_path = wt_path / ".git"
    assert git_path.exists(), f"New worktree doesn't have .git? {git_path}"

    if not agent:
        return

    # Initial presubmit: only on new worktree & branch, unless skipped or in shell mode
    if new_wt and not skip_presubmit and not shell_mode:
        if shutil.which("pre-commit"):
            try:
                run(["pre-commit", "run", "--all-files"], cwd=str(wt_path))
            except subprocess.CalledProcessError:
                click.echo(
                    "Pre-commit checks failed. Please fix the issues in the worktree or "
                    + "re-run with --skip-presubmit to bypass these checks.",
                    err=True,
                )
                sys.exit(1)
        else:
            click.echo(
                "Warning: pre-commit not installed; skipping presubmit checks", err=True
            )

    # Launch Codex agent (exec or interactive) in the sandboxed worktree
    base_prompt = (
        repo_root()
        / "agentydragon"
        / "prompts"
        / ("rebase.md" if rebase_mode else "developer.md")
    ).read_text(encoding="utf-8")
    if rebase_mode:
        context = f"You are on branch '{branch}'. Below is context to perform a rebase onto 'agentydragon'."
    else:
        md_path = tasks_dir() / f"{slug}.md"
        text = md_path.read_text(encoding="utf-8")
        m = re.search(r'title\s*=\s*"([^"]+)"', text)
        title = m.group(1) if m else slug
        context = (
            f'You are working on task {slug}: "{title}". '
            f"See the full specification in {md_path}."
        )
    run_codex_exec(
        wt_path,
        base_prompt + "\n\n" + context,
        full_auto=not shell_mode,
        exec_mode=(not interactive and not shell_mode),
    )

    # Auto-commit when Developer agent finishes and task status is Done
    if not rebase_mode and not interactive and not shell_mode:
        # Launch commit agent when task status is Done
        md_path = tasks_dir() / f"{slug}.md"
        status_txt = md_path.read_text(encoding="utf-8")
        if re.search(r'status\s*=\s*"Done"', status_txt):
            click.echo(f"Status for {slug} is Done; running Commit agent...")
            script = Path(__file__).resolve().parent / "launch_commit_agent.py"
            subprocess.check_call(
                [sys.executable, str(script), slug], cwd=str(repo_root())
            )


if __name__ == "__main__":
    import shutil

    main()
