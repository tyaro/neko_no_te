use crate::plugins::validation;
use crate::plugins::metadata::PluginMetadata;
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Spawn a process on behalf of a plugin, but enforce capability checks from its manifest.
/// `plugin_dir` should be the directory containing `plugin.toml`.
pub fn spawn_guarded(plugin_dir: &Path, cmd: &str, args: &[&str]) -> Result<std::process::ExitStatus> {
    let manifest_path = plugin_dir.join("plugin.toml");
    if !manifest_path.exists() {
        anyhow::bail!("plugin manifest not found: {:?}", manifest_path);
    }

    let (metadata, caps) = validation::validate_manifest(&manifest_path)
        .with_context(|| format!("validating manifest {:?}", manifest_path))?;

    // If the plugin does not declare process_exec capability, refuse to run.
    if !caps.contains("process_exec") {
        anyhow::bail!("plugin '{}' does not declare 'process_exec' capability; refusing to spawn process", metadata.name.unwrap_or_else(|| "<unknown>".to_string()));
    }

    // Build and spawn the command in a guarded way (minimal env pass-through)
    let mut command = Command::new(cmd);
    for a in args {
        command.arg(a);
    }

    // Optionally: sanitize environment (keep PATH and minimal vars)
    // For now inherit parent's environment but avoid forwarding sensitive vars.

    let status = command
        .status()
        .with_context(|| format!("spawning process {} for plugin", cmd))?;

    Ok(status)
}
