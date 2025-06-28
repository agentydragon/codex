import subprocess
from pathlib import Path

import pytest
from click.testing import CliRunner

from agentydragon.tools.create_task_worktree import main as cli_main


@pytest.fixture
def tmp_repo(tmp_path, monkeypatch):
    # Create a temporary git repo with a dummy task file and .worktrees dir
    repo = tmp_path / "repo"
    repo.mkdir()
    subprocess.check_call(["git", "init", "-q"], cwd=repo)
    tasks = repo / "agentydragon" / "tasks"
    (tasks / ".worktrees").mkdir(parents=True)
    task_file = tasks / "01-foo-task.md"
    task_file.write_text(
        '+++' + "\n"
        'id = "01"\n'
        'title = "Foo Task"\n'
        'status = "open"\n'
        'dependencies = []\n'
        'last_updated = "2025-01-01T00:00:00Z"\n'
        '+++\n'
    )
    # Monkey-patch common path functions to point to tmp_repo
    import agentydragon.tools.create_task_worktree as mod
    monkeypatch.setattr(mod, 'repo_root', lambda: repo)
    monkeypatch.setattr(mod, 'tasks_dir', lambda: tasks)
    monkeypatch.setattr(mod, 'worktrees_dir', lambda: tasks / '.worktrees')
    return repo

def test_worktree_creation(tmp_repo):
    runner = CliRunner()
    result = runner.invoke(cli_main, ['--skip-presubmit', '01'])
    assert result.exit_code == 0, result.output
    wt = tmp_repo / 'agentydragon/tasks/.worktrees/01-foo-task'
    assert wt.is_dir()
    gitfile = wt / '.git'
    assert gitfile.exists() and gitfile.is_file()
    assert (wt / 'agentydragon/tasks/01-foo-task.md').exists()
