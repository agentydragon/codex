//! codex-shell: lightweight inline shell mode for codex-rs

use anyhow::Result;
use std::io::{self, Write};
use tokio::io::AsyncBufReadExt;
use clap::Parser;
use codex_core::config::Config;
use codex_core::config::ConfigOverrides;

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
    // print initial session event
    println!("{session_event:?}");

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
    Ok(())
}
