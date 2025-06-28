import subprocess
import agentydragon_tasks.task as task_cli
from agentydragon_tasks.common import INTEGRATION_BRANCH
from click.testing import CliRunner


def test_full_workflow(tmp_path, monkeypatch):
    # initialize a new Git repo and set integration branch
    subprocess.run(["git", "init"], cwd=tmp_path, check=True)
    # create initial commit so branches can point to it
    subprocess.run(
        ["git", "commit", "--allow-empty", "-m", "init"], cwd=tmp_path, check=True
    )
    subprocess.run(
        ["git", "checkout", "-b", INTEGRATION_BRANCH], cwd=tmp_path, check=True
    )

    # scaffold tasks directory with one task and commit it
    tasks_dir = tmp_path / "tasks"
    tasks_dir.mkdir()
    slug = "01-hello-world"
    (tasks_dir / f"{slug}.md").write_text(
        """+++
id: "01"
title: "Hello World"
status: "Not Started"
dependencies: []
+++
Write a program test.py in repo root that prints 'Hello, world!'
"""
    )
    subprocess.run(["git", "add", "tasks"], cwd=tmp_path, check=True)
    subprocess.run(["git", "commit", "-m", "add task file"], cwd=tmp_path, check=True)

    # run real agent and hydration (errors reported by caller)
    monkeypatch.chdir(tmp_path)
    monkeypatch.setenv("XDG_STATE_HOME", str(tmp_path))

    runner = CliRunner()
    # Invoke without catching exceptions so we see full traceback on error
    result = runner.invoke(
        task_cli.cli,
        ["start-agent", "--skip-presubmit", "develop", slug],
        obj={},
        catch_exceptions=False,
    )
    # show agent output for debugging
    print(f"EXIT CODE: {result.exit_code}")
    print(f"OUTPUT:\n{result.output}")
    # ignore exit code (agent may error), but test.py should be created
    from agentydragon_tasks.common import worktrees_dir

    assert "Hello, world!" in (worktrees_dir() / slug / "test.py").read_text()
