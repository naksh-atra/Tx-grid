use serde::Serialize;

/// Unique identifier for a tmux pane.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
pub struct PaneId(String);

impl PaneId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for PaneId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Information about a single tmux pane.
#[derive(Debug, Clone, Serialize)]
pub struct PaneInfo {
    pub session_name: String,
    pub session_id: String,
    pub window_index: u32,
    pub window_name: String,
    pub pane_index: u32,
    pub pane_id: PaneId,
    pub pane_pid: u32,
    pub pane_active: bool,
}

impl PaneInfo {
    /// Returns a short locator string like "main:2.1"
    pub fn locator(&self) -> String {
        format!(
            "{}:{}.{}",
            self.session_name, self.window_index, self.pane_index
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pane_locator() {
        let pane = PaneInfo {
            session_name: "main".into(),
            session_id: "$1".into(),
            window_index: 2,
            window_name: "editor".into(),
            pane_index: 1,
            pane_id: PaneId::new("%5"),
            pane_pid: 12345,
            pane_active: true,
        };
        assert_eq!(pane.locator(), "main:2.1");
    }

    #[test]
    fn test_pane_id_as_str() {
        let id = PaneId::new("%5");
        assert_eq!(id.as_str(), "%5");
        assert_eq!(format!("{}", id), "%5");
    }
}
