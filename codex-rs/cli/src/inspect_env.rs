use std::path::PathBuf;

use crate::debug_sandbox::create_sandbox_policy;
use clap::Parser;
use codex_common::CliConfigOverrides;
use codex_common::SandboxPermissionOption;
use codex_core::config::Config;
use codex_core::config::ConfigOverrides;

/// Inspect the sandbox and container environment (mounts, permissions, network)
#[derive(Debug, Parser)]
pub struct InspectEnvArgs {
    /// Convenience alias for low-friction sandboxed automatic execution (network-disabled sandbox that can write to cwd and TMPDIR)
    #[arg(long = "full-auto", default_value_t = false)]
    pub full_auto: bool,

    /// Sandbox permission overrides (network, mounts)
    #[clap(flatten)]
    pub sandbox: SandboxPermissionOption,

    #[clap(skip)]
    pub config_overrides: CliConfigOverrides,
    /// Optional directory to write detailed logs (env: CODEX_INSPECT_LOG_DIR)
    /// Optional directory to write detailed logs
    #[arg(long = "log-dir")]
    pub log_dir: Option<PathBuf>,
}

/// Run the inspect-env command.
pub async fn run_inspect_env(
    args: InspectEnvArgs,
    codex_linux_sandbox_exe: Option<PathBuf>,
) -> anyhow::Result<()> {
    // Preserve sandbox executable path
    let exe_opt = codex_linux_sandbox_exe.clone();
    // Build sandbox policy from CLI flags.
    let sandbox_policy = create_sandbox_policy(args.full_auto, args.sandbox);
    // Load configuration to include any -c overrides and sandbox policy.
    let config = Config::load_with_cli_overrides(
        args.config_overrides
            .parse_overrides()
            .map_err(anyhow::Error::msg)?,
        ConfigOverrides {
            sandbox_policy: Some(sandbox_policy.clone()),
            codex_linux_sandbox_exe: exe_opt.clone(),
            ..Default::default()
        },
    )?;
    let policy = &config.sandbox_policy;
    // prepare log directory if requested via flag or env var
    let log_dir = args
        .log_dir
        .or_else(|| std::env::var_os("CODEX_INSPECT_LOG_DIR").map(PathBuf::from));
    if let Some(dir) = &log_dir {
        std::fs::create_dir_all(dir)?;
        // TODO: collect and write detailed container debug logs here
        println!("Logging inspector output to {}", dir.display());
    }
    let cwd = &config.cwd;

    // Compute mount entries: root and writable roots.
    let mut mounts = Vec::new();
    if policy.has_full_disk_write_access() {
        mounts.push(("/".to_string(), "rw".to_string()));
    } else if policy.has_full_disk_read_access() {
        mounts.push(("/".to_string(), "ro".to_string()));
    }
    let writable_roots = policy.get_writable_roots_with_cwd(cwd);
    for root in writable_roots.iter() {
        let path = root.display().to_string();
        if path != "/" {
            mounts.push((path, "rw".to_string()));
        }
    }

    // Determine column width for PATH.
    let width = mounts
        .iter()
        .map(|(p, _)| p.len())
        .max()
        .unwrap_or(0)
        .max(4);

    // Header.
    println!("Sandbox & Container Environment\n");
    // Container technology
    let tech = if cfg!(target_os = "linux") {
        "Landlock + seccomp"
    } else if cfg!(target_os = "macos") {
        "Seatbelt"
    } else {
        "None"
    };
    println!("Container technology: {tech}\n");
    // Environment variables (sorted)
    println!("Environment Variables:");
    let mut envs: Vec<_> = std::env::vars().collect();
    envs.sort_by(|a, b| a.0.cmp(&b.0));
    for (k, v) in envs {
        println!("  {k}={v}");
    }
    println!();

    // Mounts.
    println!("Mounts:");
    println!("  {:<width$}  MODE", "PATH", width = width);
    println!("  {:-<width$}  {:-<4}", "", "", width = width);
    for (path, mode) in &mounts {
        println!("  {path:<width$}  {mode}");
    }
    println!();
    // Working directory inside container
    println!("Container working dir: {}", cwd.display());
    println!();

    // Permissions.
    println!("Permissions:");
    for perm in policy.permissions() {
        println!("  - {perm:?}");
    }
    println!();

    // Network status.
    let net = if policy.has_full_network_access() {
        "enabled"
    } else {
        "disabled"
    };
    println!("Network: {net}");
    if net == "disabled" {
        println!(
            "  Outbound syscalls blocked: connect, accept, bind, listen, sendto, recvfrom, socket (non-AF_UNIX)"
        );
    }
    println!();

    // CLI in-container hint
    if let Some(exe) = exe_opt {
        println!(
            "To run commands inside sandbox: {} <cmd> [args]",
            exe.display()
        );
    }
    // Summary.
    println!("Summary:");
    println!("  Mount count: {}", mounts.len());
    println!("  Writable roots: {}", writable_roots.len());

    Ok(())
}
