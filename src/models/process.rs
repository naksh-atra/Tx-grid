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
    Zombie,
    Dead,
    Unknown,
}

/// Provider trait for process information — allows mocking in tests.
pub trait ProcessProvider: Send + Sync {
    fn get_process_info(&self, pid: u32) -> anyhow::Result<Option<ProcessInfo>>;
}

/// Default implementation using sysinfo + procfs on Linux.
pub struct SystemProcessProvider;

impl ProcessProvider for SystemProcessProvider {
    fn get_process_info(&self, pid: u32) -> anyhow::Result<Option<ProcessInfo>> {
        self::get_process_info(pid)
    }
}

/// Get process info using sysinfo with procfs fallback on Linux.
pub fn get_process_info(pid: u32) -> anyhow::Result<Option<ProcessInfo>> {
    use std::fs;

    // Try /proc/<pid>/stat and /proc/<pid>/cmdline first (Linux)
    let cmdline_path = format!("/proc/{}/cmdline", pid);
    let stat_path = format!("/proc/{}/stat", pid);

    if !Path::new(&cmdline_path).exists() {
        return Ok(None);
    }

    let cmdline = fs::read_to_string(&cmdline_path).unwrap_or_default();
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

    // Parse stat for start time (field 22) and state (field 3)
    let (start_time, state) = if let Ok(stat) = fs::read_to_string(&stat_path) {
        let fields: Vec<&str> = stat.split_whitespace().collect();
        let state = fields.get(2).map(|s| parse_state(*s)).unwrap_or(ProcessState::Unknown);
        let start_time = fields.get(21).and_then(|s| s.parse::<u64>().ok());
        (start_time, state)
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
        "Z" => ProcessState::Zombie,
        "X" | "x" => ProcessState::Dead,
        _ => ProcessState::Unknown,
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_state() {
        assert_eq!(parse_state("R"), ProcessState::Running);
        assert_eq!(parse_state("S"), ProcessState::Sleeping);
        assert_eq!(parse_state("Z"), ProcessState::Zombie);
        assert_eq!(parse_state("X"), ProcessState::Dead);
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
}
