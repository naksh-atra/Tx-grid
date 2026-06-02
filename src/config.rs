use std::time::Duration;

/// Application configuration read from tmux options.
#[derive(Debug, Clone)]
pub struct Config {
    pub popup_width: String,
    pub popup_height: String,
    pub runtime_threshold: u64,
    pub confirm_kill: bool,
    pub refresh_interval: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            popup_width: "80%".to_string(),
            popup_height: "60%".to_string(),
            runtime_threshold: 5,
            confirm_kill: true,
            refresh_interval: Duration::from_secs(5),
        }
    }
}

impl Config {
    /// Load config from tmux options with defaults.
    pub fn from_tmux() -> Self {
        let default = Self::default();
        Self {
            popup_width: crate::services::tmux_service::get_tmux_option(
                "@taskgrid-popup-width",
                &default.popup_width,
            ),
            popup_height: crate::services::tmux_service::get_tmux_option(
                "@taskgrid-popup-height",
                &default.popup_height,
            ),
            runtime_threshold: crate::services::tmux_service::get_tmux_option(
                "@taskgrid-runtime-threshold",
                &default.runtime_threshold.to_string(),
            )
            .parse()
            .unwrap_or(default.runtime_threshold),
            confirm_kill: crate::services::tmux_service::get_tmux_option(
                "@taskgrid-confirm-kill",
                "1",
            ) == "1",
            refresh_interval: Duration::from_secs(
                crate::services::tmux_service::get_tmux_option(
                    "@taskgrid-refresh-interval",
                    "5",
                )
                .parse()
                .unwrap_or(5),
            ),
        }
    }
}
