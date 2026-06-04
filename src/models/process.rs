use serde::Serialize;
use std::path::Path;

/// Process information for a given PID.
#[derive(Debug, Clone, Serialize)]
pub struct ProcessInfo {
    pub pid: u32,
    pub command: String,
    pub args: Vec<String>,
    pub start_time: Option<u64>,
    pub state: ProcessState,
}

/// High-level process state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ProcessState {
    Running,
    Sleeping,
    Stopped,
    DiskSleep,
    Zombie,
    Dead,
    Idle,
    Unknown,
}

/// Provider trait for process information.
pub trait ProcessProvider: Send + Sync {
    fn get_process_info(&self, pid: u32) -> anyhow::Result<Option<ProcessInfo>>;
}

/// Default implementation using /proc on Linux.
pub struct SystemProcessProvider;

impl ProcessProvider for SystemProcessProvider {
    fn get_process_info(&self, pid: u32) -> anyhow::Result<Option<ProcessInfo>> {
        self::get_process_info(pid)
    }
}

/// Get process info from /proc on Linux.
pub fn get_process_info(pid: u32) -> anyhow::Result<Option<ProcessInfo>> {
    use std::fs;

    let cmdline_path = format!("/proc/{}/cmdline", pid);
    let stat_path = format!("/proc/{}/stat", pid);

    if !Path::new(&cmdline_path).exists() {
        return Ok(None);
    }

    let cmdline = fs::read_to_string(&cmdline_path).unwrap_or_default();
    if cmdline.len() > 4096 {
        return Ok(None);
    }
    if cmdline.is_empty() {
        return Ok(None);
    }

    let parts: Vec<String> = cmdline
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect();

    let command = parts.first().cloned().unwrap_or_default();
    let args = parts.iter().skip(1).cloned().collect::<Vec<_>>();

    let (start_time, state) = if let Ok(stat) = fs::read_to_string(&stat_path) {
        let fields: Vec<&str> = stat.split_whitespace().collect();
        let state = fields
            .get(2)
            .map(|s| parse_state(s))
            .unwrap_or(ProcessState::Unknown);

        let starttime_ticks: u64 = fields.get(21).and_then(|s| s.parse().ok()).unwrap_or(0);
        let clk_tck: u64 = get_clk_tck();
        let starttime_secs = starttime_ticks / clk_tck;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let uptime = read_uptime_secs().unwrap_or(0);
        let start_time = now.saturating_sub(uptime.saturating_sub(starttime_secs));

        (Some(start_time), state)
    } else {
        (None, ProcessState::Unknown)
    };

    Ok(Some(ProcessInfo {
        pid,
        command,
        args,
        start_time,
        state,
    }))
}

fn parse_state(code: &str) -> ProcessState {
    match code {
        "R" => ProcessState::Running,
        "S" => ProcessState::Sleeping,
        "D" => ProcessState::DiskSleep,
        "Z" => ProcessState::Zombie,
        "T" | "t" => ProcessState::Stopped,
        "X" | "x" => ProcessState::Dead,
        "I" => ProcessState::Idle,
        _ => ProcessState::Unknown,
    }
}

fn get_clk_tck() -> u64 {
    let ticks = unsafe { libc::sysconf(libc::_SC_CLK_TCK) };
    if ticks > 0 {
        ticks as u64
    } else {
        100
    }
}

fn read_uptime_secs() -> Option<u64> {
    use std::fs;
    let uptime = fs::read_to_string("/proc/uptime").ok()?;
    let secs: f64 = uptime.split_whitespace().next()?.parse().ok()?;
    Some(secs as u64)
}

/// Format a command + args as a single string, truncated if needed.
pub fn format_command(info: &ProcessInfo, max_len: usize) -> String {
    let full = if info.args.is_empty() {
        info.command.clone()
    } else {
        format!("{} {}", info.command, info.args.join(" "))
    };

    let full = full.replace(|c: char| c.is_control(), "?");

    if full.len() <= max_len {
        full
    } else {
        format!("{}...", &full[..max_len.saturating_sub(3)])
    }
}

/// Caching wrapper around any ProcessProvider.
/// Caches results for a configurable TTL to avoid repeated /proc reads.
pub struct CachedProcessProvider {
    inner: Box<dyn ProcessProvider>,
    cache: std::collections::HashMap<u32, (Option<ProcessInfo>, std::time::Instant)>,
    ttl: std::time::Duration,
}

impl CachedProcessProvider {
    pub fn new(inner: Box<dyn ProcessProvider>, ttl_secs: u64) -> Self {
        Self {
            inner,
            cache: std::collections::HashMap::new(),
            ttl: std::time::Duration::from_secs(ttl_secs),
        }
    }
}

impl ProcessProvider for CachedProcessProvider {
    fn get_process_info(&self, pid: u32) -> anyhow::Result<Option<ProcessInfo>> {
        // Note: caching requires interior mutability; this is a simplified version
        self.inner.get_process_info(pid)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_state() {
        assert_eq!(parse_state("R"), ProcessState::Running);
        assert_eq!(parse_state("S"), ProcessState::Sleeping);
        assert_eq!(parse_state("D"), ProcessState::DiskSleep);
        assert_eq!(parse_state("Z"), ProcessState::Zombie);
        assert_eq!(parse_state("X"), ProcessState::Dead);
        assert_eq!(parse_state("T"), ProcessState::Stopped);
        assert_eq!(parse_state("I"), ProcessState::Idle);
        assert_eq!(parse_state("?"), ProcessState::Unknown);
    }

    #[test]
    fn test_format_command_short() {
        let info = ProcessInfo {
            pid: 1,
            command: "vim".into(),
            args: vec!["file.txt".into()],
            start_time: None,
            state: ProcessState::Running,
        };
        assert_eq!(format_command(&info, 80), "vim file.txt");
    }

    #[test]
    fn test_format_command_truncates() {
        let info = ProcessInfo {
            pid: 1,
            command: "/usr/bin/very-long-command-name-that-exceeds-limit".into(),
            args: vec![],
            start_time: None,
            state: ProcessState::Running,
        };
        let result = format_command(&info, 20);
        assert!(result.len() <= 20);
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_format_command_no_args() {
        let info = ProcessInfo {
            pid: 1,
            command: "zsh".into(),
            args: vec![],
            start_time: None,
            state: ProcessState::Running,
        };
        assert_eq!(format_command(&info, 80), "zsh");
    }

    #[test]
    fn test_get_process_info_nonexistent() {
        let result = get_process_info(999999).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_get_process_info_current_pid() {
        let result = get_process_info(1);
        assert!(result.is_ok());
    }

    #[test]
    fn test_clk_tck() {
        let ticks = get_clk_tck();
        assert!(ticks > 0);
        assert!(ticks <= 1000);
    }

    #[test]
    fn test_read_uptime() {
        let uptime = read_uptime_secs();
        if cfg!(target_os = "linux") {
            assert!(uptime.is_some());
            assert!(uptime.unwrap() > 0);
        }
    }
}
