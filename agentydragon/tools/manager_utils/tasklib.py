"""
Simple library for loading and saving task metadata embedded as TOML front-matter
in task Markdown files.
"""

import re
import subprocess
from datetime import datetime
from pathlib import Path

import toml
from enum import Enum
from pydantic import BaseModel, Field

FRONTMATTER_RE = re.compile(r"^\+\+\+\s*(.*?)\s*\+\+\+", re.S | re.M)


def repo_root():
    return Path(
        subprocess.check_output(["git", "rev-parse", "--show-toplevel"])
        .decode()
        .strip()
    )


def task_dir():
    return repo_root() / "agentydragon/tasks"


def worktree_dir():
    return task_dir() / ".worktrees"


def find_task_file(task_id: str) -> Path:
    """Locate the task Markdown file for TASK_ID in tasks/ or tasks/.done."""
    base = task_dir()
    files = list(base.glob(f"{task_id}-*.md"))
    done_dir = base / ".done"
    if done_dir.exists():
        files += list(done_dir.glob(f"{task_id}-*.md"))
    if not files:
        raise FileNotFoundError(f"Task {task_id} not found")
    return files[0]


def list_task_files() -> list[Path]:
    """List all task Markdown files in tasks/ and tasks/.done, without recursing into worktrees."""
    base = task_dir()
    files = sorted(base.glob("[0-9][0-9]-*.md"))
    done_dir = base / ".done"
    if done_dir.exists():
        files += sorted(done_dir.glob("[0-9][0-9]-*.md"))
    return files


class TaskStatus(str, Enum):
    NOT_STARTED = "open"
    IN_PROGRESS = "wip"
    NEEDS_INPUT = "needsinput"
    DONE = "done"
    CANCELLED = "cancelled"
    MERGED = "merged"


class TaskMeta(BaseModel):
    id: str
    title: str
    status: TaskStatus
    freeform_status: str = Field(default="")
    dependencies: list[int] = Field(default_factory=list)
    last_updated: datetime = Field(default_factory=datetime.utcnow)


def load_task(path: Path) -> (TaskMeta, str):
    text = path.read_text(encoding="utf-8")
    m = FRONTMATTER_RE.match(text)
    if not m:
        raise ValueError(f"No TOML frontmatter in {path}")
    meta = toml.loads(m.group(1))
    tm = TaskMeta(**meta)
    body = text[m.end() :].lstrip("\n")
    return tm, body


def save_task(path: Path, meta: TaskMeta, body: str) -> None:
    tm = meta.dict()
    # Serialize enum to its string value for front-matter
    if isinstance(tm.get("status"), Enum):
        tm["status"] = tm["status"].value
    tm["last_updated"] = meta.last_updated.isoformat()
    fm = toml.dumps(tm).strip()
    content = f"+++\n{fm}\n+++\n\n{body.lstrip()}"
    path.write_text(content, encoding="utf-8")
