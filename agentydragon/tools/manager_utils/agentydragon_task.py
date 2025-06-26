"""
CLI for managing agentydragon tasks: status, set-status, set-deps, dispose, launch.
"""
import subprocess
import re
import sys
from datetime import datetime
from pathlib import Path

import click
import time
from tasklib import load_task, repo_root, save_task, task_dir, TaskMeta, worktree_dir, TaskStatus
import pygit2
import shutil

# Styling configuration for task statuses
STATUS_COLORS: dict[str, dict[str, str]] = {
    TaskStatus.NOT_STARTED.value:         {'fg': 'reset'},
    TaskStatus.IN_PROGRESS.value:         {'fg': 'yellow'},
    TaskStatus.NEEDS_INPUT.value:         {'fg': 'red'},
    TaskStatus.NEEDS_MANUAL_REVIEW.value: {'fg': 'red'},
    TaskStatus.DONE.value:                {'fg': 'green'},
    TaskStatus.CANCELLED.value:           {'fg': 'red'},
    TaskStatus.MERGED.value:              {'fg': 'blue'},
}

try:
    from tabulate import tabulate
except ImportError:
    tabulate = None


@click.group()
def cli():
    """Manage agentydragon tasks."""
    pass

@cli.command()
@click.option('--timings', is_flag=True, help='Print timing breakdown of status execution')
def status(timings: bool):
    """Show a table of task id, title, status, dependencies, last_updated.

    If tabulate is installed, render as GitHub-flavored Markdown table;
    otherwise fallback to fixed-width formatting.
    """
    start = time.monotonic() if timings else None
    # Load all task metadata, excluding worktrees for speed; include .done explicitly
    all_meta: dict[str, TaskMeta] = {}
    path_map: dict[str, Path] = {}
    task_root = task_dir()
    done_root = task_root / '.done'
    files: list[Path] = sorted(task_root.glob('[0-9][0-9]-*.md'))
    if done_root.exists():
        files += sorted(done_root.glob('[0-9][0-9]-*.md'))
    for md in files:
        if md.name in ('task-template.md',) or md.name.endswith('-plan.md'):
            continue
        try:
            meta, _ = load_task(md)
        except Exception as e:
            print(f"Error loading {md}: {e}")
            continue
        all_meta[meta.id] = meta
        path_map[meta.id] = md
    if timings:
        print(f"Loaded {len(path_map)} tasks in {time.monotonic() - start:.3f}s")

    # Reload from worktree copies if present (to reflect live Status in branch)
    if timings:
        t0 = time.monotonic()
    repo = repo_root()
    for tid, md in list(path_map.items()):
        wt_task = worktree_dir() / md.stem / md.relative_to(repo)
        if wt_task.exists():
            try:
                wt_meta, _ = load_task(wt_task)
                all_meta[tid] = wt_meta
                path_map[tid] = wt_task
            except Exception as e:
                print(f"Error loading {wt_task}: {e}")
    if timings:
        t1 = time.monotonic()
        print(f"Reloaded worktree tasks in {t1 - t0:.3f}s")

    # Build dependency graph, excluding already merged tasks
    merged_ids = {tid for tid, m in all_meta.items() if m.status == 'Merged'}
    deps_map: dict[str, list[str]] = {}
    for tid, meta in all_meta.items():
        deps_map[tid] = [d for d in re.findall(r"\d+", meta.dependencies)
                         if d in all_meta and d not in merged_ids]

    # Topologically sort tasks by dependencies, fall back on filename order on error
    try:
        sorted_ids: list[str] = []
        temp: set[str] = set()
        perm: set[str] = set()
        def visit(n: str) -> None:
            if n in perm:
                return
            if n in temp:
                raise RuntimeError(f"Circular dependency detected at task {n}")
            temp.add(n)
            for m in deps_map.get(n, []):
                visit(m)
            temp.remove(n)
            perm.add(n)
            sorted_ids.append(n)
        for n in all_meta:
            visit(n)
    except Exception as e:
        print(f"Warning: cannot topo-sort tasks ({e}); falling back to filename order")
        sorted_ids = [m.id for m in sorted(all_meta.values(), key=lambda m: path_map[m.id].name)]

    # Identify tasks that are merged with no branch and no worktree (bottom summary)
    bottom_merged_ids: set[str] = set()
    for tid in sorted_ids:
        meta = all_meta[tid]
        if meta.status != 'Merged':
            continue
        branches = subprocess.run(
            ['git', 'for-each-ref', '--format=%(refname:short)',
             f'refs/heads/agentydragon-{tid}-*'],
            capture_output=True, text=True, cwd=repo_root()
        ).stdout.strip().splitlines()
        wt_dir = task_dir() / '.worktrees' / path_map[tid].stem
        if not branches and not wt_dir.exists():
            bottom_merged_ids.add(tid)

    # time deps & topo sort
    if timings:
        t2 = time.monotonic()
        print(f"Deps & topo sort in {t2 - t1:.3f}s")
    # Initialize pygit2 for batch Git operations
    repo = pygit2.Repository(str(repo_root()))
    integration_ref = 'refs/heads/agentydragon'
    integration_oid = repo.references[integration_ref].target

    # Build rows (branch/worktree checks)
    worktree_time = 0.0
    branch_time = 0.0
    if timings:
        t3 = time.monotonic()
    rows: list[tuple] = []
    merged_tasks: list[tuple[str, str]] = []
    root = repo_root()

    for tid in sorted_ids:
        meta = all_meta[tid]
        md = path_map[tid]
        slug = md.stem
        # Worktree cleanliness and branch status via pygit2
        wt_info = 'none'
        wt_dir = worktree_dir() / slug
        if wt_dir.exists():
            wt_start = time.monotonic() if timings else None
            try:
                # Use gitstatusd-backed porcelain status for performance
                out = subprocess.run(
                    ['git', 'status', '--porcelain=2', '--branch', '--untracked-files=no'],
                    cwd=wt_dir, capture_output=True, text=True
                ).stdout
                wt_info = 'dirty' if out.strip() else 'clean'
            except Exception:
                wt_info = 'dirty'
            if timings and wt_start is not None:
                worktree_time += time.monotonic() - wt_start
        # Skip fully merged tasks (no branch, no worktree)
        pattern = f'refs/heads/agentydragon-{tid}-'
        branch_refs = [r for r in repo.references if r.startswith(pattern)]
        if meta.status == TaskStatus.MERGED and not branch_refs and wt_info == 'none':
            merged_tasks.append((tid, meta.title))
            continue
        # Dependencies (excluding bottom-merged)
        deps = [d for d in deps_map.get(tid, []) if d not in bottom_merged_ids]
        deps_str = ','.join(deps)
        # Determine branch_info
        # Branch status timing
        if timings:
            br_start = time.monotonic()
        if not branch_refs:
            branch_info = 'no branch'
        elif wt_info == 'dirty':
            # skip ahead/behind when worktree is dirty
            branch_info = 'dirty-wt'
        else:
            branch_ref = branch_refs[0]
            branch_oid = repo.references[branch_ref].target
            if repo.descendant_of(integration_oid, branch_oid):
                branch_info = 'merged'
            else:
                ahead, behind = repo.ahead_behind(branch_oid, integration_oid)
                if ahead == 0 and behind == 0:
                    branch_info = 'up-to-date'
                else:
                    arrows: list[str] = []
                    if behind:
                        arrows.append(f'{behind}↓')
                    if ahead:
                        arrows.append(f'{ahead}↑')
                    branch_info = ''.join(arrows)
        if timings:
            branch_time += time.monotonic() - br_start

        # Style status and worktree columns
        label = meta.status.value
        stat_disp = click.style(label, **STATUS_COLORS.get(label, {}))
        wt_disp = wt_info
        if wt_info == 'dirty':
            wt_disp = click.style(wt_info, fg='red')

        rows.append((
            tid, meta.title, stat_disp,
            deps_str, meta.last_updated.strftime('%Y-%m-%d %H:%M'),
            branch_info, wt_disp
        ))

    if timings:
        print(f"Worktree checks: {worktree_time:.3f}s, Branch checks: {branch_time:.3f}s")
    headers = ['ID', 'Title', 'Status', 'Depends on', 'Updated',
               'Branch Status', 'Worktree Status']

    if timings:
        t4 = time.monotonic()
        print(f"Built table rows in {t4 - t3:.3f}s")
    if tabulate:
        print(tabulate(rows, headers=headers, tablefmt='github'))
    else:
        fmt = '{:>2}  {:<30}  {:<12}  {:<20}  {:<16}  {:<40}  {:<10}'
        print(fmt.format(*headers))
        for r in rows:
            print(fmt.format(*r))

    # summary of fully merged tasks (no branch, no worktree)
    if merged_tasks:
        items = ' '.join(f"{tid} ({title})" for tid, title in merged_tasks)
        print(f"\n\033[32mMerged:\033[0m {items}")

    # summary of tasks Ready to merge (Done with branch commits)
    ready_tasks: list[tuple[str, str]] = []
    for tid in sorted_ids:
        meta = all_meta[tid]
        if meta.status != TaskStatus.DONE:
            continue
        pattern = f'refs/heads/agentydragon-{tid}-'
        branch_refs = [r for r in repo.references if r.startswith(pattern)]
        if not branch_refs:
            continue
        branch_oid = repo.references[branch_refs[0]].target
        # ahead count relative to integration branch
        ahead, _ = repo.ahead_behind(branch_oid, integration_oid)
        if ahead > 0:
            ready_tasks.append((tid, meta.title))
    if ready_tasks:
        items = ' '.join(f"{tid} ({title})" for tid, title in ready_tasks)
        print(f"\n\033[33mReady to merge:\033[0m {items}")

    # identify unblocked tasks (no remaining dependencies)
    unblocked = [tid for tid in sorted_ids if tid not in merged_ids and not deps_map.get(tid)]
    if unblocked:
        print(f"\n\033[1mUnblocked:\033[0m {' '.join(unblocked)}")
        print(f"\033[1mLaunch unblocked in tmux:\033[0m python agentydragon/tools/create_task_worktree.py --agent --tmux {' '.join(unblocked)}")
    if timings:
        print(f"Total status time: {time.monotonic() - start:.3f}s")

@cli.command()
@click.argument('task_id')
@click.argument('status')
def set_status(task_id, status):
    """Set status of TASK_ID to STATUS"""
    # search both in tasks/ and tasks/.done/ for the task file
    files = list(task_dir().rglob(f'{task_id}-*.md'))
    if not files:
        click.echo(f'Task {task_id} not found', err=True)
        sys.exit(1)
    path = files[0]
    meta, body = load_task(path)
    # Validate status string against TaskStatus enum
    try:
        meta.status = TaskStatus(status)
    except ValueError:
        valid = ', '.join([s.value for s in TaskStatus])
        click.echo(f"Invalid status '{status}'. Valid statuses: {valid}", err=True)
        sys.exit(1)
    meta.last_updated = datetime.utcnow()
    save_task(path, meta, body)
    # Move between tasks/ and tasks/.done according to status transitions
    done_dir = task_dir() / '.done'
    # Move to .done on Merged
    if meta.status == TaskStatus.MERGED and path.parent.name != '.done':
        done_dir.mkdir(exist_ok=True)
        dest = done_dir / path.name
        click.echo(f"Archiving task: moving {path.name} -> {done_dir.relative_to(repo_root())}")
        subprocess.run(['git', 'mv', str(path), str(dest)], cwd=repo_root())
    # Move back to main tasks/ when status changes away from Done/Merged
    elif path.parent.name == '.done' and meta.status not in (TaskStatus.DONE, TaskStatus.MERGED):
        dest = task_dir() / path.name
        click.echo(f"Reopening task: moving {path.name} -> {dest.parent.relative_to(repo_root())}")
        subprocess.run(['git', 'mv', str(path), str(dest)], cwd=repo_root())

@cli.command()
@click.argument('task_id')
@click.argument('deps', nargs=-1)
def set_deps(task_id, deps):
    """Set dependencies of TASK_ID"""
    files = list(task_dir().glob(f'{task_id}-*.md'))
    if not files:
        click.echo(f'Task {task_id} not found', err=True)
        sys.exit(1)
    path = files[0]
    meta, body = load_task(path)
    now = datetime.utcnow().isoformat()
    meta.dependencies = f'as of {now}: ' + ', '.join(deps)
    meta.last_updated = datetime.utcnow()
    save_task(path, meta, body)

@cli.command()
@click.argument('task_id', nargs=-1)
def dispose(task_id):
    """Dispose worktree and delete branch for TASK_ID(s)"""
    root = repo_root()
    wt_base = worktree_dir()
    for tid in task_id:
        # Remove any matching worktree directories
        g = f'{tid}-*'
        matching_wts = wt_base.glob(g)
        for wt_dir in matching_wts:
            click.echo(f"Disposing worktree {wt_dir}")
            # unregister worktree; then delete the directory if still present
            rel = wt_dir.relative_to(root)
            subprocess.run(['git', 'worktree', 'remove', str(rel), '--force'], cwd=root)
            if wt_dir.exists():
                shutil.rmtree(wt_dir)
        else:
            print(f"No worktrees matching {g} in {wt_base}")
        # prune any stale worktree entries
        subprocess.run(['git', 'worktree', 'prune'], cwd=root)
        # Delete any matching branches
        # delete any matching local branches cleanly via for-each-ref
        ref_pattern = f'refs/heads/agentydragon-{tid}-*'
        branches = subprocess.run(
            ['git', 'for-each-ref', '--format=%(refname:short)', ref_pattern],
            capture_output=True, text=True, cwd=root
        ).stdout.splitlines()
        branches = [br for br in branches if br]
        if branches:
            click.echo(f"Disposing branches: {branches}")
            subprocess.run(['git', 'branch', '-D', *branches], cwd=root)
        else:
            click.echo(f"No branches matching {ref_pattern}")
        click.echo(f'Disposed task {tid}')
        # If the task was marked Done, auto-move it into .done/
        files = list(task_dir().glob(f"{tid}-*.md"))
        if len(files) == 1:
            path = files[0]
            meta, _ = load_task(path)
            if meta.status == TaskStatus.DONE:
                done_dir = task_dir() / '.done'
                done_dir.mkdir(exist_ok=True)
                target = done_dir / path.name
                click.echo(f"Moving {path.name} -> .done/ (status Done)")
                subprocess.run(['git', 'mv', str(path), str(target)], cwd=repo_root())

@cli.command()
@click.argument('task_id', nargs=-1)
def launch(task_id):
    """Copy tmux launch one-liner for TASK_ID(s) to clipboard"""
    cmd = ['create-task-worktree.sh', '--agent', '--tmux'] + list(task_id)
    line = ' '.join(cmd)
    # system clipboard
    try:
        subprocess.run(['pbcopy'], input=line.encode(), check=True)
        click.echo('Copied to clipboard:')
    except FileNotFoundError:
        click.echo(line)
        return
    click.echo(line)

@cli.command()
def workflow():
    """Interactive workflow: commit dirty worktrees, merge ready branches, dispose, and report task statuses."""
    root = repo_root()
    # gather tasks and worktree status
    dirty = []
    ready = []
    done = []
    need_input = []
    unblocked = []
    # dependencies map and task metas
    all_meta: dict[str, TaskMeta] = {}
    path_map: dict[str, Path] = {}
    deps_map: dict[str, list[str]] = {}
    # load task files
    for md in sorted(task_dir().rglob('[0-9][0-9]-*.md')):
        if md.name in ('task-template.md',) or md.name.endswith('-plan.md'):
            continue
        try:
            meta, _ = load_task(md)
        except Exception:
            continue
        all_meta[meta.id] = meta
        path_map[meta.id] = md
    # compute worktree and branch info
    for tid, md in list(path_map.items()):
        meta = all_meta[tid]
        slug = md.stem
        wt = worktree_dir() / slug
        # worktree status
        if wt.exists():
            st = subprocess.run(['git', 'status', '--porcelain'], cwd=wt,
                                capture_output=True, text=True).stdout.strip()
            if st:
                dirty.append(tid)
        # ready to merge: Done with commits ahead
        if meta.status == TaskStatus.DONE:
            branches = subprocess.run(
                ['git', 'for-each-ref', '--format=%(refname:short)',
                 f'refs/heads/agentydragon-{tid}-*'], capture_output=True, text=True, cwd=root
            ).stdout.splitlines()
            if branches and branches[0].strip():
                bname = branches[0].lstrip('*+ ').strip()
                a_cnt, _ = subprocess.check_output(
                    ['git', 'rev-list', '--left-right', '--count', f'{bname}...agentydragon'], cwd=root
                ).decode().split()
                if int(a_cnt) > 0:
                    ready.append((tid, bname))
        # tasks needing input
        if meta.status == TaskStatus.NEEDS_INPUT:
            need_input.append(tid)
    # dependencies for unblocked
    merged_ids = {tid for tid, m in all_meta.items() if m.status == TaskStatus.MERGED}
    for tid, meta in all_meta.items():
        deps = [d for d in re.findall(r"\d+", meta.dependencies)
                if d in all_meta and d not in merged_ids]
        deps_map[tid] = deps
        if meta.status not in (TaskStatus.MERGED,) and not deps:
            unblocked.append(tid)
    # 1. Commit dirty worktrees
    for tid in dirty:
        if click.confirm(f"Run Commit agent for task {tid}?", default=True):
            subprocess.run([sys.executable,
                            str(repo_root()/'agentydragon'/'tools'/'launch_commit_agent.py'), tid], cwd=root)
    # 2. Merge ready branches
    for tid, bname in ready:
        if click.confirm(f"Merge branch {bname} into agentydragon?", default=True):
            click.echo(f"Merging {bname} into agentydragon")
            subprocess.check_call(['git', 'checkout', 'agentydragon'], cwd=root)
            subprocess.check_call(['git', 'merge', '--no-ff', bname], cwd=root)
    # 3. Dispose merged tasks
    for tid, _ in ready:
        if click.confirm(f"Dispose task worktree and branch for {tid}?", default=False):
            subprocess.run([sys.executable,
                            __file__, 'dispose', tid], cwd=root)
    # 4. Tasks needing input
    if need_input:
        click.echo(f"Tasks needing input: {' '.join(need_input)}")
    # 4.5 Running tmux sessions and codex processes
    try:
        sessions = subprocess.run(['tmux', 'ls'], capture_output=True, text=True).stdout.strip()
        if sessions:
            click.echo("\nActive tmux sessions:")
            click.echo(sessions)
    except FileNotFoundError:
        pass
    procs = subprocess.run(['pgrep', '-fl', 'codex'], capture_output=True, text=True).stdout.strip()
    if procs:
        click.echo("\nRunning codex processes:")
        click.echo(procs)
    # 5. Offer to launch unblocked tasks
    if unblocked:
        click.echo(f"Unblocked tasks: {' '.join(unblocked)}")
        if click.confirm('Launch unblocked tasks in tmux?'):
            click.echo('Launching unblocked tasks:')
            click.echo(f"python3 agentydragon/tools/create_task_worktree.py --agent --tmux {' '.join(unblocked)}")
    # Print timing for print/table phase and total
    if timings:
        t4 = time.monotonic()
        click.echo(f"Printed table & summaries in {t4 - t3:.3f}s")
        click.echo(f"Total status time: {t4 - start:.3f}s")

if __name__ == '__main__':
    cli()
