use crate::models::pane::{PaneId, PaneInfo};
use anyhow::Context;
use log::{debug, warn};
use std::process::Command;

/// Execute a tmux command and return stdout.
pub fn tmux_command(args: &[&str]) -> anyhow::Result<String> {
    let output = Command::new("tmux")
        .args(args)
        .output()
        .context("failed to execute tmux command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("tmux command failed: {}", stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// List all panes across all sessions.
pub fn list_panes() -> anyhow::Result<Vec<PaneInfo>> {
    let format = "#{session_name}\t#{session_id}\t#{window_index}\t#{window_name}\t#{pane_index}\t#{pane_id}\t#{pane_pid}\t#{pane_active}";
    let output = tmux_command(&["list-panes", "-a", "-F", format])?;

    let mut panes = Vec::new();
    for line in output.lines() {
        if line.trim().is_empty() {
            continue;
        }
        match parse_pane_line(line) {
            Some(pane) => {
                debug!("Discovered pane: {}", pane.locator());
                panes.push(pane);
            }
            None => {
                warn!("Failed to parse pane line: {}", line);
            }
        }
    }

    Ok(panes)
}

/// Get the current pane ID (the pane running this binary).
pub fn current_pane_id() -> Option<String> {
    std::env::var("TMUX_PANE").ok()
}

/// Get a tmux option value with a default fallback.
pub fn get_tmux_option(name: &str, default: &str) -> String {
    let output = Command::new("tmux")
        .args(["show-options", "-gqv", name])
        .output();

    match output {
        Ok(out) if out.status.success() => {
            let val = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if val.is_empty() {
                default.to_string()
            } else {
                val
            }
        }
        _ => default.to_string(),
    }
}

/// Select (jump to) a specific pane.
pub fn select_pane(pane_id: &PaneId) -> anyhow::Result<()> {
    tmux_command(&["select-pane", "-t", &pane_id.0])?;
    Ok(())
}

/// Kill a specific pane.
pub fn kill_pane(pane_id: &PaneId) -> anyhow::Result<()> {
    tmux_command(&["kill-pane", "-t", &pane_id.0])?;
    Ok(())
}

/// Send keys to a specific pane.
pub fn send_keys(pane_id: &PaneId, keys: &str) -> anyhow::Result<()> {
    tmux_command(&["send-keys", "-t", &pane_id.0, keys])?;
    Ok(())
}

/// Get the current command running in a pane.
pub fn pane_current_command(pane_id: &PaneId) -> Option<String> {
    let output = tmux_command(&["display-message", "-p", "-t", &pane_id.0, "#{pane_current_command}"]);
    output.ok().map(|s| s.trim().to_string())
}

/// Check if tmux is available and meets minimum version.
pub fn check_tmux() -> anyhow::Result<(String, bool)> {
    let output = Command::new("tmux")
        .arg("-V")
        .output()
        .context("tmux not found in PATH")?;

    let version_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
    debug!("tmux version: {}", version_str);

    // Parse version number
    let version_supported = version_str
        .split_whitespace()
        .nth(1)
        .and_then(|v| {
            let parts: Vec<&str> = v.split('.').collect();
            let major = parts.first()?.parse::<u32>().ok()?;
            let minor = parts.get(1)?.parse::<u32>().ok()?;
            Some(major > 3 || (major == 3 && minor >= 2))
        })
        .unwrap_or(false);

    Ok((version_str, version_supported))
}

fn parse_pane_line(line: &str) -> Option<PaneInfo> {
    let parts: Vec<&str> = line.split('\t').collect();
    if parts.len() < 8 {
        return None;
    }

    Some(PaneInfo {
        session_name: parts[0].to_string(),
        session_id: parts[1].to_string(),
        window_index: parts[2].parse().ok()?,
        window_name: parts[3].to_string(),
        pane_index: parts[4].parse().ok()?,
        pane_id: PaneId::new(parts[5]),
        pane_pid: parts[6].parse().ok()?,
        pane_active: parts[7] == "1",
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_pane_line_valid() {
        let line = "main\t$0\t1\teditor\t0\t%5\t12345\t1";
        let pane = parse_pane_line(line).unwrap();
        assert_eq!(pane.session_name, "main");
        assert_eq!(pane.window_index, 1);
        assert_eq!(pane.window_name, "editor");
        assert_eq!(pane.pane_index, 0);
        assert_eq!(pane.pane_id.0, "%5");
        assert_eq!(pane.pane_pid, 12345);
        assert!(pane.pane_active);
    }

    #[test]
    fn test_parse_pane_line_inactive() {
        let line = "main\t$0\t1\teditor\t0\t%5\t12345\t0";
        let pane = parse_pane_line(line).unwrap();
        assert!(!pane.pane_active);
    }

    #[test]
    fn test_parse_pane_line_too_few_fields() {
        let line = "main\t$0\t1";
        assert!(parse_pane_line(line).is_none());
    }

    #[test]
    fn test_parse_pane_line_empty() {
        assert!(parse_pane_line("").is_none());
    }
}
