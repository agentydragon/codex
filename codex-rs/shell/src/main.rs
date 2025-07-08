use clap::Parser;
use codex_common::CliConfigOverrides;
use codex_shell::Cli;
use codex_shell::run_main;

/// Top‑level CLI to merge overrides and invoke run_main
#[derive(Parser, Debug)]
struct TopCli {
    #[clap(flatten)]
    config_overrides: CliConfigOverrides,

    #[clap(flatten)]
    inner: Cli,
}

fn main() -> anyhow::Result<()> {
    codex_linux_sandbox::run_with_sandbox(|sandbox_exe| async move {
        let top = TopCli::parse();
        let mut cli = top.inner;
        cli.config_overrides
            .raw_overrides
            .splice(0..0, top.config_overrides.raw_overrides);
        run_main(cli, sandbox_exe).await?;
        Ok(())
    })
}
