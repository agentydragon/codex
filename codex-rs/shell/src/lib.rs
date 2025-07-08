//! codex-shell: lightweight inline shell mode for codex-rs

use anyhow::Result;
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
    print!("\x1b]0;codex-shell {}\x07", session_id);
    println!("/help = help, Enter = send, Ctrl-X = toggle mode, Ctrl-D = quit");

    // Initialize Codex client and display session start
    let (codex, session_event, ctrl_c) =
        codex_core::codex_wrapper::init_codex(config.clone()).await?;
    // unify model and hook events into a single channel
    let (evt_tx, mut evt_rx) = tokio::sync::mpsc::unbounded_channel();
    // send initial session event
    let _ = evt_tx.send(session_event);
    // spawn model event producer
    tokio::spawn(async move {
        while let Ok(e) = codex.next_event().await {
            let _ = evt_tx.send(e);
        }
    });
    // TODO: spawn hook event producers and forward into evt_tx

    // Event loop: print each incoming unified event
    loop {
        tokio::select! {
            _ = ctrl_c.notified() => break,
            maybe = evt_rx.recv() => match maybe {
                Some(e) => println!("{:?}", e),
                None => break,
            }
        }
    }
    Ok(())
}
