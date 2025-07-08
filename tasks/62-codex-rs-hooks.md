+++
id = "62"
title = "Support hooks API in codex-rs"
status = "open"
freeform_status = ""
dependencies = [] # Manager rationale: independent codex-rs hooks feature; no prerequisite tasks
last_updated = "2025-07-13T00:00:00Z"
+++

> *This task is specific to codex-rs.*

# Task 62: Support hooks API in codex-rs

## Acceptance Criteria

- Provide a registration mechanism for external hooks at session start, supplying the session ID.
- Expose hook events:
  - session start
  - session end
  - user message sent
  - model turn ended (user's turn begins)
  - command executed (with exit code and details)
  - patch applied or failed (with outcome)
  - approval dialog and pre-approval predicate evaluation (auto-approve/auto-deny) for commands and patches
- Allow hooks to register and unregister slash commands at runtime.
  - Slash command invocations are sent only to the hook that registered that command.
  - Slash invocation events include the full user-entered prompt (including slash prefix) and omit separate command IDs or tokenized args.
  - Slash commands are a distinct hook event type, separate from shell `command_executed` events.
- Enable hook responses to slash commands to inject messages back into the LLM.
- Allow hooks to augment the approval dialog with custom menu items, receiving approval request IDs.
- Provide access to session state: conversation history, current working directory, last executed command, and any context fields needed.
- Allow hooks to provide feedback to the model on both approved and unapproved command executions and patch applications.
- Allow hooks to insert arbitrary user-visible messages into the session.
  - Hooks may only send user-visible messages at points when user messages would be valid in the conversation flow (e.g., not in response to an approval-question event where a tool response is expected).


- Hooks must respond to each app event with zero or more actions/responses bundled in a single envelope (e.g., JSON object or array).
- Each response envelope MUST include the triggering event's ID in the `event_id` field to correlate it with its originating event.

- Hooks must respond with one of:
  - `no_opinion`
  - `approve`
  - `deny`
  - `user_options`: an array of option records `{ action: "approve"|"deny", text: String, id: String }`
- If the user selects an option, Codex-rs applies the approve/deny and emits an event back to the hook with that option’s `id`.

**Motivating example:** A Git-auto-approval hook can inspect branch, remote, or commit message to auto-approve or deny a `git commit`. If undecided, it can return `user_options` like:
```json
{
  "user_options": [
    { "action": "approve", "text": "Allow this commit and all commits in this repo", "id": "opt1" },
    { "action": "deny",    "text": "Disallow this commit and all commits on this branch", "id": "opt2" }
  ]
}
```

## Implementation

**How it was implemented**  

- Extend `HookManager::new` to register external hook processes at session start, passing the session ID.
- Add `HookManager::handle_event` to broadcast additional lifecycle events (user message sent, model turn ended, command executed, patch applied/failed, approval requests, slash invocations).
- Introduce a runtime `SlashCommandRegistry` to track slash commands hooks register/unregister, dispatching invocations to the appropriate hook.
- Update core session loop (`CodexSession` or equivalent) to invoke hook manager at each acceptance criterion point and supply event IDs.
- Extend `HookResponse` to support new response types: custom approval options (`user_options`), menu augmentation, insert user-visible or conversation messages.
- Process hook responses asynchronously in the event loop, correlating by `event_id` and performing actions: auto-approve/deny, inject messages, update approval UI, register/unregister slash commands.

**How it works**  

When a session starts, `HookManager::new` spawns each configured hook process and emits a `session_start` event with the session ID.

During the session, for each key lifecycle event (user message sent, model turn ended, command executed with exit code, patch applied or failed, approval dialog invoked, slash-command invocation), the core serializes a `HookEvent` (with a unique `event_id`) and writes it to each hook’s stdin.

Hooks respond by writing `HookResponse` JSON objects to stdout, containing the triggering `event_id`. The core reads these responses and:
- Applies approval decisions (`approve`/`deny`) or presents custom `user_options` in the approval dialog.
- Registers or unregisters slash commands dynamically.
- Inserts user-visible messages or injects conversation messages back into the LLM prompt.
- Augments the approval UI with custom menu items linked to hook-provided IDs.

On session end, `HookManager.shutdown` emits `session_end` and waits for hook processes to exit.

## Notes

<!-- Any additional notes or references. -->
