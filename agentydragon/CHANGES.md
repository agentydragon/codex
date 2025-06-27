# codex-rs Changelog

This document summarizes changes between `main` and the current HEAD for the `codex-rs` workspace.

## 🚀 New Features

### Build & Installation

- **Rust-native binary installation**: you can now build and install the Rust CLI from source:

```shell
cargo install --path cli --locked
```

For system-wide installation (e.g., `/usr/local`):

```shell
sudo cargo install --path cli --locked --root /usr/local
```

### Custom Approval Predicates (`auto_allow`)

- Added an `auto_allow` section in `config.toml` to define user-provided scripts that vote on each shell command (`allow`, `deny`, or `no-opinion`).

```toml
[[auto_allow]]
script = "/path/to/approve_predicate.py"
```

##### Example predicate script (`approve_predicate.py`)

```python
#!/usr/bin/env python3
"""
Custom auto-approval predicate for codex-rs.

Reads the candidate shell command as its sole argument and prints exactly
one of: "allow", "deny", or "no-opinion" to stdout.
"""
import sys

def main(cmd: str) -> None:
    # Deny destructive patterns
    if "rm -rf /" in cmd:
        print("deny")
        return

    # Auto-allow git commands
    if cmd.strip().startswith("git "):
        print("allow")
        return

    # Otherwise, defer to manual approval
    print("no-opinion")

if __name__ == "__main__":
    if len(sys.argv) != 2:
        sys.exit("Usage: approve_predicate.py '<full command line>'")
    main(sys.argv[1])
```

### Model Context Protocol Enhancements

- Expanded MCP server configuration examples under `mcp_servers` in `config.toml`:

```toml
[mcp_servers.server-name]
command = "npx"
args    = ["-y", "mcp-server"]
env     = { "API_KEY" = "value" }
```

- Added JSON-RPC examples for `tools/list` and `tools/call` messages:

```jsonc
// ListTools request
{ "jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {} }

// CallTool request
{ "jsonrpc": "2.0", "id": 2, "method": "tools/call",
  "params": { "name": "codex", "arguments": { "prompt": "Hello!" } }
}
```

### Built-in System Prompt Override

- Introduced `CODEX_BASE_INSTRUCTIONS_FILE` environment variable to override or disable the built-in system prompt (`prompt.md`).

## ⚙️ CLI Enhancements

### `codex config` Subcommands

- Added `config` subcommand to manage `~/.codex/config.toml`:

```shell
codex config edit               # open config in $EDITOR (or vi)
codex config set KEY VALUE      # set a TOML key (e.g., tui.auto_mount_repo true)
```

### Session Resume

- Added `codex session <UUID>` to resume an existing TUI session.

### Sandbox Inspection (`inspect-env`)

- New `codex inspect-env` command to display sandbox mounts, permissions, and network:

```shell
codex inspect-env --full-auto -s disk-full-read-access
```

### Sandbox Permission Flags

- Introduced `--sandbox-permission/-s` flags to grant granular sandbox permissions via CLI:

```shell
codex -s disk-full-read-access \
      -s disk-write-cwd \
      -s disk-write-platform-user-temp-folder
```

## 🛠 Configuration Schema Changes

### Approval Policy Rename

- Changed default `approval_policy` value from `untrusted` to `unless-allow-listed`.

### Sandbox Permissions

- Replaced the old `[sandbox]` table with a `sandbox_permissions` array. For example:

```toml
sandbox_permissions = [
  "disk-full-read-access",
  "disk-write-cwd",
  "disk-write-platform-user-temp-folder",
]
```

- Use `disk-write-folder=/path/to/folder` or `disk-read-folder=/path/to/folder` to grant access to custom paths.

### TUI Configuration Options

- Added new TUI settings in `config.toml`:

```toml
[tui]
disable_mouse_capture = true     # let your terminal handle mouse selection
composer_max_rows = 10           # max input lines before internal scrolling
editor = "${VISUAL:-${EDITOR:-nvim}}"  # external editor for prompt composition
message_spacing = false          # insert blank lines between messages
sender_break_line = false        # render sender label above content
```

## 📚 Documentation Updates

- **codex-rs/README.md**: Added build-from-source instructions, `auto_allow` guide, MCP JSON-RPC examples, and `codex config` walkthrough.
- **config.md**: Updated `approval_policy`, added `sandbox_permissions`, `auto_allow`, and `base_instructions_override` sections; streamlined TUI docs.
- **core/README.md**: Documented system prompt composition steps and `CODEX_BASE_INSTRUCTIONS_FILE` behavior.
- **core/init.md**: Introduced a template for generating `AGENTS.md` for coding agents.

## 🔄 Code & API Changes

- **cli crate**: integrated `SandboxPermissionOption`, extended `create_sandbox_policy`, introduced TOML override parsing and `apply_override` logic for `codex config set`.
- **common crate**: added `SandboxPermissionOption`, removed deprecated `sandbox_summary` module, and updated `ApprovalModeCliArg` variant to `UnlessAllowListed`.

## 🛠 Dependency Updates

- `.gitignore`: now ignores `debug.log` and `debug-sequencing.log`.
- **cli Cargo.toml**: added `toml`, `serde`, `uuid` (with serde/v4), and `tempfile` as a dev-dependency.
- **common Cargo.toml**: removed `sandbox_summary` feature.
- **Cargo.lock**: updated and added crates: `filetime`, `fsevent-sys`, `inotify`/`inotify-sys`, `kqueue`/`kqueue-sys`, `notify`, `mio v0.8.11`, `windows-sys v0.48.0`, `windows-targets v0.48.5`, and others.
