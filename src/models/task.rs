use serde::Serialize;

use super::pane::PaneInfo;
use super::process::ProcessInfo;

/// Classification of a task based on process state and command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum TaskState {
    Running,
    Exited,
    Idle,
    Unknown,
}

impl TaskState {
    pub fn as_str(&self) -> &'static str {
        match self {
            TaskState::Running => "running",
            TaskState::Exited => "exited",
            TaskState::Idle => "idle",
            TaskState::Unknown => "unknown",
        }
    }
}

/// A task represents a pane annotated with process info and classification.
#[derive(Debug, Clone, Serialize)]
pub struct Task {
    pub pane: PaneInfo,
    pub process: Option<ProcessInfo>,
    pub state: TaskState,
    pub runtime: Option<u64>, // seconds
    pub command_display: String,
}

/// Known idle/shell commands that indicate no meaningful work.
const IDLE_COMMANDS: &[&str] = &[
    "bash", "zsh", "fish", "sh", "dash", "csh", "tcsh",
];

/// Check if a command is a known shell (idle when no child processes).
fn is_idle_command(cmd: &str) -> bool {
    let base = cmd.rsplit('/').next().unwrap_or(cmd);
    IDLE_COMMANDS.contains(&base)
}

/// Build a Task from a PaneInfo and optional ProcessInfo.
pub fn classify(pane: &PaneInfo, process: Option<&ProcessInfo>) -> Task {
    let state = match process {
        None => TaskState::Exited,
        Some(p) => match p.state {
            super::process::ProcessState::Dead | super::process::ProcessState::Zombie => TaskState::Exited,
            super::process::ProcessState::Running | super::process::ProcessState::Sleeping => {
                if is_idle_command(&p.command) {
                    TaskState::Idle
                } else {
                    TaskState::Running
                }
            }
            _ => TaskState::Unknown,
        },
    };

    let command_display = process
        .map(|p| super::process::format_command(p, 60))
        .unwrap_or_else(|| String::from("(none)"));

    let runtime = process.and_then(|p| p.start_time).map(|start| {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        now.saturating_sub(start)
    });

    Task {
        pane: pane.clone(),
        process: process.cloned(),
        state,
        runtime,
        command_display,
    }
}

/// Build tasks for all panes, filtering out the current pane (the taskgrid popup itself)
/// and optionally filtering by a set of session names to exclude.
pub fn build_tasks(
    panes: &[PaneInfo],
    process_provider: &dyn super::process::ProcessProvider,
    current_pane_id: Option<&str>,
) -> Vec<Task> {
    panes
        .iter()
        .filter(|p| {
            // Exclude the current taskgrid popup pane
            current_pane_id.map_or(true, |id| p.pane_id.as_str() != id)
        })
        .map(|pane| {
            let process = process_provider.get_process_info(pane.pane_pid).ok().flatten();
            classify(pane, process.as_ref())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::pane::PaneId;
    use crate::models::process::ProcessState;

    fn make_pane(pid: u32, active: bool) -> PaneInfo {
        PaneInfo {
            session_name: "main".into(),
            session_id: "$0".into(),
            window_index: 0,
            window_name: "main".into(),
            pane_index: pid,
            pane_id: PaneId::new(format!("%{}", pid)),
            pane_pid: pid,
            pane_active: active,
        }
    }

    fn make_running_proc(cmd: &str) -> ProcessInfo {
        ProcessInfo {
            pid: 123,
            command: cmd.to_string(),
            args: vec![],
            start_time: Some(0),
            state: ProcessState::Running,
        }
    }

    #[test]
    fn test_classify_running() {
        let pane = make_pane(123, true);
        let proc = make_running_proc("nvim");
        let task = classify(&pane, Some(&proc));
        assert_eq!(task.state, TaskState::Running);
    }

    #[test]
    fn test_classify_idle_shell() {
        let pane = make_pane(123, true);
        let proc = make_running_proc("zsh");
        let task = classify(&pane, Some(&proc));
        assert_eq!(task.state, TaskState::Idle);
    }

    #[test]
    fn test_classify_exited() {
        let pane = make_pane(123, false);
        let task = classify(&pane, None);
        assert_eq!(task.state, TaskState::Exited);
    }

    #[test]
    fn test_classify_dead_process() {
        let pane = make_pane(123, false);
        let proc = ProcessInfo {
            pid: 123,
            command: "vim".into(),
            args: vec![],
            start_time: None,
            state: ProcessState::Dead,
        };
        let task = classify(&pane, Some(&proc));
        assert_eq!(task.state, TaskState::Exited);
    }

    #[test]
    fn test_build_tasks_filters_current() {
        use crate::models::process::ProcessProvider;

        struct MockProvider;
        impl ProcessProvider for MockProvider {
            fn get_process_info(&self, _: u32) -> anyhow::Result<Option<ProcessInfo>> {
                Ok(Some(make_running_proc("nvim")))
            }
        }

        let panes = vec![
            make_pane(0, true),
            make_pane(1, false),
        ];

        let tasks = build_tasks(&panes, &MockProvider, Some("%0"));
        assert_eq!(tasks.len(), 1);
    }
}
