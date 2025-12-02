use crate::plugins::validation;
use anyhow::{Context, Result};
use std::path::Path;
use std::process::{Command, ExitStatus, Output};

/// Spawn a process on behalf of a plugin, but enforce capability checks from its manifest.
/// `plugin_dir` should be the directory containing `plugin.toml`.
#[allow(dead_code)]
pub fn spawn_guarded(plugin_dir: &Path, cmd: &str, args: &[&str]) -> Result<ExitStatus> {
    let owned_args = args.iter().map(|a| a.to_string()).collect::<Vec<_>>();
    let output = exec_with_output(plugin_dir, cmd, &owned_args)?;
    Ok(output.status)
}

pub fn exec_with_output(plugin_dir: &Path, cmd: &str, args: &[String]) -> Result<Output> {
    let manifest_path = plugin_dir.join("plugin.toml");
    if !manifest_path.exists() {
        anyhow::bail!("plugin manifest not found: {:?}", manifest_path);
    }

    let (metadata, caps) = validation::validate_manifest(&manifest_path)
        .with_context(|| format!("validating manifest {:?}", manifest_path))?;

    if !caps.contains("process_exec") {
        anyhow::bail!(
            "plugin '{}' does not declare 'process_exec' capability; refusing to spawn process",
            metadata.name.unwrap_or_else(|| "<unknown>".to_string())
        );
    }

    let mut command = Command::new(cmd);
    for arg in args {
        command.arg(arg);
    }
    command.current_dir(plugin_dir);

    let output = command
        .output()
        .with_context(|| format!("spawning process {} for plugin", cmd))?;

    if output.status.success() {
        Ok(output)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "plugin '{}' command failed (status {}): {}",
            metadata.name.unwrap_or_else(|| cmd.to_string()),
            output.status,
            stderr.trim()
        );
    }
}
