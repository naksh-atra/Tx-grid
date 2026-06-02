use thiserror::Error;

#[derive(Error, Debug)]
pub enum TaskGridError {
    #[error("tmux not found in PATH")]
    TmuxNotFound,

    #[error("tmux server not running")]
    TmuxNotRunning,

    #[error("tmux version too old: {0} (need 3.2+)")]
    TmuxVersionTooOld(String),

    #[error("popup not supported in this tmux version")]
    PopupUnsupported,

    #[error("process inspection failed for PID {0}: {1}")]
    ProcessInspectionFailed(u32, String),

    #[error("action failed: {0}")]
    ActionFailed(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("unknown error: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, TaskGridError>;
