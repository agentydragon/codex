//! codex-shell: lightweight inline shell mode for codex-rs

use anyhow::Result;
use std::io::{self, Write};
use tokio::io::AsyncBufReadExt;
use clap::Parser;
use codex_core::config::Config;
use codex_core::config::ConfigOverrides;
use codex_core::hooks::HookManager;
use codex_core::hooks::HookResponse;
use codex_core::protocol::Event;
use codex_core::protocol::InputItem;
use codex_core::protocol::Op;
use crossterm::cursor::MoveToColumn;
use crossterm::queue;
use crossterm::terminal::Clear;
use crossterm::terminal::ClearType;
use std::env;
use std::io::Write;
use std::io::{self};
use std::sync::Arc;
use tokio::io::AsyncBufReadExt;
use tokio::io::BufReader;
use tokio::sync::Mutex;

/// Unified event type for model and hook responses.
#[allow(dead_code)]
#[derive(Debug)]
enum ShellEvent {
    Codex(Event),
    Hook(HookResponse),
    /// user-typed line (without trailing newline)
    Input(String),
}

/// Print a single event inline (append-only).
fn render_event(evt: &ShellEvent) {
    // inline history entries with aligned labels
    const LABEL_WIDTH: usize = 6; // width of longest label, e.g. "codex:"
    match evt {
        ShellEvent::Input(text) => {
            let label = "user:";
            let spaces = " ".repeat(LABEL_WIDTH.saturating_sub(label.len()));
            println!("\x1b[32m{label}{spaces}{text}\x1b[0m");
        }
        ShellEvent::Codex(ev) => {
            use codex_core::protocol::EventMsg;
            let label = "codex:";
            let spaces = " ".repeat(LABEL_WIDTH - label.len());
            match &ev.msg {
                EventMsg::SessionConfigured(_) => {}
                EventMsg::AgentMessage(m) => println!(
                    "\x1b[34m{label}{spaces}{text}\x1b[0m",
                    text = m.message,
                    label = label,
                    spaces = spaces
                ),
                EventMsg::AgentReasoning(r) if !r.text.is_empty() => {
                    println!("\x1b[34m{}{}{}\x1b[0m", label, spaces, r.text)
                }
                other => println!("\x1b[34m{label}{spaces}{other:?}\x1b[0m"),
            }
        }
        ShellEvent::Hook(resp) => {
            let label = "hook::";
            let spaces = " ".repeat(LABEL_WIDTH.saturating_sub(label.len()));
            println!("\x1b[34m{label}{spaces}{resp:?}\x1b[0m");
        }
    }
}

/// Redraw the prompt showing cwd and '> '
fn redraw_prompt() {
    let cwd = env::current_dir().unwrap_or_default();
    let cwd_display = cwd.display();
    let mut out = io::stdout();
    let _ = queue!(out, MoveToColumn(0), Clear(ClearType::CurrentLine));
    let _ = write!(out, "{cwd_display} > ");
    let _ = out.flush();
}

/// CLI options for codex-shell
#[derive(Debug, Parser)]
pub struct Cli {
    /// Apply config overrides (key=value)
    #[clap(skip)]
    pub config_overrides: codex_common::CliConfigOverrides,

    /// Initial prompt to display
    #[clap(long)]
    pub prompt: Option<String>,
}

/// Run the codex-shell application
pub async fn run_main(cli: Cli, _sandbox_exe: Option<std::path::PathBuf>) -> Result<()> {
    // Load configuration with CLI overrides
    // Parse `-c` overrides from the CLI.
    let cli_kv_overrides = match cli.config_overrides.parse_overrides() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error parsing -c overrides: {e}");
            std::process::exit(1);
        }
    };
    let config = match Config::load_with_cli_overrides(cli_kv_overrides, ConfigOverrides::default())
    {
        Ok(cfg) => cfg,
        Err(err) => {
            eprintln!("Error loading configuration: {err}");
            std::process::exit(1);
        }
    };
    // Startup banner and terminal title
    let session_id = uuid::Uuid::new_v4();
    println!(
        "OpenAI Codex Shell v{} (session {})",
        env!("CARGO_PKG_VERSION"),
        session_id
    );
    // Set terminal title to session
    print!("\x1b]0;codex-shell {session_id}\x07");
    println!("/help = help, Enter = send, Ctrl-X = toggle mode, Ctrl-D = quit");

    // Initialize Codex client and display session start
    let (codex, session_event, ctrl_c) =
        codex_core::codex_wrapper::init_codex(config.clone()).await?;
    let codex = Arc::new(codex);
    // unify model and hook events into a single channel
    let (evt_tx, mut evt_rx) = tokio::sync::mpsc::unbounded_channel::<ShellEvent>();
    // spawn hook processes and read their responses
    let hook_mgr = Arc::new(Mutex::new(HookManager::new(&config.hooks, session_id)));
    {
        let hook_tx = evt_tx.clone();
        let hook_mgr = Arc::clone(&hook_mgr);
        tokio::spawn(async move {
            let mut mgr = hook_mgr.lock().await;
            mgr.read_responses(move |resp: HookResponse| {
                let _ = hook_tx.send(ShellEvent::Hook(resp));
            })
            .await;
        });
    }
    // broadcast initial session event
    let _ = evt_tx.send(ShellEvent::Codex(session_event.clone()));
    // spawn Op submission task
    let (op_tx, mut op_rx) = tokio::sync::mpsc::unbounded_channel::<Op>();
    {
        let codex = Arc::clone(&codex);
        tokio::spawn(async move {
            while let Some(op) = op_rx.recv().await {
                let _ = codex.submit(op).await;
            }
        });
    }
    // spawn user-input reader: echo & send
    {
        let evt_tx = evt_tx.clone();
        let op_tx = op_tx.clone();
        tokio::spawn(async move {
            let mut lines = BufReader::new(tokio::io::stdin()).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let _ = evt_tx.send(ShellEvent::Input(line.clone()));
                if !line.is_empty() {
                    let items = vec![InputItem::Text { text: line }];
                    let _ = op_tx.send(Op::UserInput { items });
                }
            }
        });
    }
    // spawn model event producer: forward to hooks then to channel
    {
        let hook_mgr = Arc::clone(&hook_mgr);
        let evt_tx = evt_tx.clone();
        let codex = Arc::clone(&codex);
        tokio::spawn(async move {
            while let Ok(e) = codex.next_event().await {
                let mut mgr = hook_mgr.lock().await;
                mgr.handle_event(&e);
                let _ = evt_tx.send(ShellEvent::Codex(e));
            }
        });
    }

    // spawn stdin reader: lines sent into input_rx
    let (input_tx, mut input_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
    {
        // read lines from stdin asynchronously
        let input_tx = input_tx.clone();
        tokio::spawn(async move {
            let stdin = tokio::io::stdin();
            let mut reader = tokio::io::BufReader::new(stdin).lines();
            while let Ok(Some(line)) = reader.next_line().await {
                if input_tx.send(line).is_err() {
                    break;
                }
            }
        });
    }
    // Event loop: handle Ctrl-C, Codex events, or user input
    loop {
        tokio::select! {
            // exit on Ctrl-C
            _ = ctrl_c.notified() => break,
            // next event from Codex
            res = codex.next_event() => match res {
                Ok(e) => {
                    // handle approval requests specially
                    let id = e.id.clone();
                    match e.msg {
                        codex_core::protocol::EventMsg::ExecApprovalRequest(req) => {
                            println!("Command approval request: {:?}", req.command);
                            print!("Approve? [1=approve, 2=approve-session, 3=abort, Enter=deny]: "); io::stdout().flush()?;
                            if let Some(ans) = input_rx.recv().await {
                                let dec = match ans.trim() {
                                    "1" => codex_core::protocol::ReviewDecision::Approved,
                                    "2" => codex_core::protocol::ReviewDecision::ApprovedForSession,
                                    "3" => codex_core::protocol::ReviewDecision::Abort,
                                    _ => codex_core::protocol::ReviewDecision::Denied,
                                };
                                let _ = codex.submit(
                                    codex_core::protocol::Op::ExecApproval { id, decision: dec }
                                ).await?;
                            }
                        }
                        codex_core::protocol::EventMsg::ApplyPatchApprovalRequest(req) => {
                            println!("Patch approval request");
                            print!("Approve patch? [1=approve, 2=approve-session, 3=abort, Enter=deny]: "); io::stdout().flush()?;
                            if let Some(ans) = input_rx.recv().await {
                                let dec = match ans.trim() {
                                    "1" => codex_core::protocol::ReviewDecision::Approved,
                                    "2" => codex_core::protocol::ReviewDecision::ApprovedForSession,
                                    "3" => codex_core::protocol::ReviewDecision::Abort,
                                    _ => codex_core::protocol::ReviewDecision::Denied,
                                };
                                let _ = codex.submit(
                                    codex_core::protocol::Op::PatchApproval { id, decision: dec }
                                ).await?;
                            }
                        }
                        other => {
                            // print other events
                            println!("{:?}", codex_core::protocol::Event { id, msg: other });
                        }
                    }
                }
                Err(_) => break,
            },
            // user input from stdin
            Some(line) = input_rx.recv() => {
                let items = vec![codex_core::protocol::InputItem::Text { text: line }];
                // submit user input to Codex
                let _ = codex.submit(codex_core::protocol::Op::UserInput { items }).await?;
            }
        }
    }
    // end
    Ok(())
}
