+++
id = "30"
title = "Non-Fullscreen Scrollback Mode with Native Terminal Scroll"
status = "done"
freeform_status = ""
dependencies = [] # Manager rationale: independent UI enhancement; no prerequisite tasks
last_updated = "2025-07-01T00:00:00.000000"
+++

## Summary
Offer a non-fullscreen TUI mode in codex-rs that appends conversation output and defers scrolling to the terminal scrollback.

## Detail

In this mode codex-rs should behave *analogously to a normal shell like `zsh` or similar* -- history growing at the bottom, mouse handled by terminal emulator, scroll wheel not captured.

Imagine user currently has a shell session open in their terminal:

 1 > /home/foo $ ls
 2 > Documents Downloads Pictures x.txt y.txt
 3 > /home/foo $ cd Documents
 4 > /home/foo/Documents $ _

Now they decide to fire up codex-rs and do some stuff here. codex-rs should
open, starting as a *short couple-lines-long output* letting the user enter
a prompt, and progrssively displaying converastin progress.

All > - blocks here show the *entire visible content* of the user's terminal
and '_' represents cursor position.
Precise UI here is a *mock* - it is *NOT* in scope for the implementation to implement it precisely.
It is just to illustrate the "codex-rs as a *shell*" UX idea.
In this mock I'm using \ as Codex's PS1 char for clarity. The 100% is the "context left" display.

User starts "Codex shell":

 1 > /home/foo $ ls
 2 > Documents Downloads Pictures x.txt y.txt
 3 > /home/foo $ cd Documents
 4 > /home/foo/Documents $ codex-rs
 5 > OpenAI Codex v0.0.0 (research preview)
 6 > /help = help, Enter = send, C-d = quit, C-j = newline
 7 > /home/foo/Documents 100% \ _

User can now enter a query:

 1 > /home/foo $ ls
 2 > Documents Downloads Pictures x.txt y.txt
 3 > /home/foo $ cd Documents
 4 > /home/foo/Documents $ codex-rs
 5 > OpenAI Codex v0.0.0 (research preview)
 6 > /help = help, Enter = send, C-d = quit, C-j = newline
 7 > /home/foo/Documents 100% \ count tokens to encode all *md except foo.md_

Then press Enter and send it to the model:

 1 > /home/foo $ ls
 2 > Documents Downloads Pictures x.txt y.txt
 3 > /home/foo $ cd Documents
 4 > /home/foo/Documents $ codex-rs
 5 > OpenAI Codex v0.0.0 (research preview)
 6 > /help = help, Enter = send, C-d = quit, C-j = newline
 7 >  user: count tokens to encode all *md except foo.md
 8 > codex: thinking...
 9 > /home/foo/Documents 99% \ _

While Codex is working, user might start typing another prompt for later:

 1 > /home/foo $ ls
 2 > Documents Downloads Pictures x.txt y.txt
 3 > /home/foo $ cd Documents
 4 > /home/foo/Documents $ codex-rs
 5 > OpenAI Codex v0.0.0 (research preview)
 6 > /help = help, Enter = send, C-d = quit, C-j = newline
 7 >  user: count tokens to encode all *md except foo.md
 8 > codex: ✓ 0.1s $ ls -1 
 9 > codex: ⣽ 0.3s $ pip list | grep tiktoken
10 > /home/foo/Documents 98% \ oh and also total kb in all images_

Note that user prompt and user's entered text remained *at the bottom*, while conversation
grew *just above it*.

Model may need something approved:

 1 > /home/foo $ ls
 2 > Documents Downloads Pictures x.txt y.txt
 3 > /home/foo $ cd Documents
 4 > /home/foo/Documents $ codex-rs
 5 > OpenAI Codex v0.0.0 (research preview)
 6 > /help = help, Enter = send, C-d = quit, C-j = newline
 7 >  user: count tokens to encode all *md except foo.md
 8 > codex: ✓ 0.1s $ ls -1 
 9 >        ✗ 0.5s $ pip list | grep tiktoken
10 >             ☞ $ pip install tiktoken ☜
11 >               Run this command?
12 >  > Allow once (C-y)
13 >    Allow `pip install *` for rest of session (C-a)
14 >    Deny/feedback (C-n)
15 >    Abort rollout (Esc)
16 > /home/foo/Documents 97% \ oh and also total kb in all images_

Note that shortcuts (C-y, C-a, C-n, Esc) are picked such that user is *unlikely* to accidentally
hit them while typing the prompt, and user *can* continue typing while an approve question is
popped up.

Then if user approves, their prompt might jump back up (here from line 16 to line 14):

 1 > /home/foo $ ls
 2 > Documents Downloads Pictures x.txt y.txt
 3 > /home/foo $ cd Documents
 4 > /home/foo/Documents $ codex-rs
 5 > OpenAI Codex v0.0.0 (research preview)
 6 > /help = help, Enter = send, C-d = quit, C-j = newline
 7 >  user: count tokens to encode all *md except foo.md
 8 > codex: ✓ 0.1s $ ls -1 
 9 >        ✗ 0.5s $ pip list | grep tiktoken
10 >  allow ⣻ 2.7s $ pip install tiktoken
11 > ... pip output ... pip output ...
12 > ... pip output ... pip output ...
13 > ... pip output ... pip output ...
14 > /home/foo/Documents 96% \ oh and also total kb in all images_

Users can alos enter their own commands to execute (though implementing that is NOT part of this
task; inputs & outputs of such commands will be shown to the model, too).
Users would denote that they want to execute a command (like in a shell) rather than send a text
message via a special mode they can toggle with a keyboard shortcut, say <C-x>.
Sampling from the model would be only triggered with text messages, not with shell commands.

 1 > /home/foo $ ls
 2 > Documents Downloads Pictures x.txt y.txt
 3 > /home/foo $ cd Documents
 4 > /home/foo/Documents $ codex-rs
 5 > OpenAI Codex v0.0.0 (research preview)
 6 > /help = help, Enter = send, C-d = quit, C-j = newline  
 7 >  user: count tokens to encode all *md except foo.md
 8 > codex: ✓ 0.1s $ ls -1 
 9 >        ✗ 0.5s $ pip list | grep tiktoken
10 >        ✓ 6.1s $ pip install tiktoken [once]
11 > ... pip output ... pip output ...
12 > ... pip output ... pip output ...
13 > ... pip output ... pip output ...
14 > ... pip output ... pip output ...
15 > ... pip output ... pip output ...
16 >               $ patch count_tokens.py [session]
17 >        ✓ 1.4s $ python count_tokens.py [session]
18 > Tokens: 17 308 in 17 .md files
19 > foo.md:  1 308
20 > codex: You'd need 16 000 tokens to encode all 16 *.md files (foo.md not included).
21 >  user: let's look in 'taxes' subfolder
22 > codex: ✓ 0.1s $ cd taxes
23 >        OK, we're in 'taxes' subfolder, what here?
24 >  user: delete all zip files
25 > codex: ✓ 0.1s $ rm *.zip
26 >        Done.
27 >  user: ✓ 0.1s $ ls
28 > 2024/ foo.txt bar.txt taxes-2025.pdf ...
29 >  user: ✓ 0.4s $ cd 2024
30 >  user: how many tokens in this folder's .md?
31 > codex: ✓ 0.8s $ python ../count_tokens.py --dir=. [allow session]
32 > Tokes: 9 300 in 4 .md files
33 > codex: There's 9 300 tokens in ~/Documents/taxes/2024/**/*.md.
34 > /home/foo/Documents/taxes/2024/ 90% \ <<user presses C-d here or /exit>>
35 > /home/foo/Documents $ _  <<user exited Codex, is now back in their original shell>>

Let's say user's terminal emulator is only 10 lines high and user can't see above line 23.
If they want to see the history of their interaction, they can user *their terminal emulator's
scrollback function* to scroll up to see the whole (console-pretty-formatted) transcript.
It's "concise" in the sense that we don't show detailed Rust data structures, we compress
approval prompt events and with their execution, etc.

## Goal
Provide an optional non-fullscreen mode for the chat UI where:
- The TUI does not capture the mouse scroll wheel.
- All conversation output is appended in place, allowing the terminal's native scrollback to navigate history.
- The user-entry window remains fixed at the bottom of the terminal.
- The entire UI runs in a standard terminal buffer (no alternate screen), so the user can use their terminal’s scrollbar or scrollback keys to review past messages.

## Acceptance Criteria

- Introduce a `tui.non_fullscreen_mode` config flag (default `false`).
- When enabled, the application:
  - Disables alternate screen buffering (i.e. does not switch to the TUI alt-screen).
  - Does not intercept mouse scroll events; scroll events are passed through to the terminal.
  - Renders new chat messages inline (appended) rather than redrawing the full viewport.
  - Keeps the user input prompt visible at the bottom after each message.
- Add integration tests or manual validation steps to confirm that: scrollback keys/mouse scroll work via terminal scrollback, and the prompt remains in view.

## Implementation

**How it was implemented**  
- Add `non_fullscreen_mode: bool` to the `tui` config section.
- In the TUI initialization, skip entering the alternate screen and disable pannable viewports.
- Remove mouse event capture for scroll wheel events when `non_fullscreen_mode` is true.
- Change rendering loop: after each new message, print the message directly to the stdout buffer (in append mode), then redraw only the input prompt line.
- Write integration tests that spawn the TUI in non-fullscreen mode, emit multiple messages, send scroll events (if possible), and assert that scrollback buffer contains the messages.

## Notes

- This mode trades advanced in-TUI scrolling features for simplicity and compatibility with users’ accustomed terminal scrollback.
- It may not support complex viewport resizing; documentation should note that.

## Current Implementation Limitations

- The existing `non_fullscreen_mode` implementation still switches to a cleared full-screen buffer on startup, wiping out the previous terminal content.
- Conversation history remains in a fixed-height TUI widget (with its own internal scrollbar); excess messages beyond the widget’s visible area do not enter the terminal emulator’s scrollback.
- Screen clearing and widget-based scrolling violate the spec’s requirement to append output inline and rely solely on the terminal’s native scrollback.
