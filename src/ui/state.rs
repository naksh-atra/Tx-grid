use crate::models::task::Task;
use crate::models::task::TaskState;
use std::time::{Duration, Instant};

/// Application modes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    Filter,
    Confirm,
}

/// Sort order for the task list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SortOrder {
    Runtime,
    Session,
    State,
}

impl SortOrder {
    pub fn next(&self) -> Self {
        match self {
            SortOrder::Runtime => SortOrder::Session,
            SortOrder::Session => SortOrder::State,
            SortOrder::State => SortOrder::Runtime,
        }
    }
}

pub struct App {
    pub tasks: Vec<Task>,
    pub filtered_indices: Vec<usize>,
    pub selection: usize,
    pub mode: AppMode,
    pub filter_text: String,
    pub sort_order: SortOrder,
    pub status_message: Option<String>,
    pub scroll_offset: usize,
    pub confirm_action: Option<ConfirmAction>,
    pub last_refresh: Instant,
    pub refresh_interval: Duration,
}

pub enum ConfirmAction {
    Kill(String), // pane_id
}

impl App {
    pub fn new(tasks: Vec<Task>) -> Self {
        let filtered_indices: Vec<usize> = (0..tasks.len()).collect();
        Self {
            tasks,
            filtered_indices,
            selection: 0,
            mode: AppMode::Normal,
            filter_text: String::new(),
            sort_order: SortOrder::Runtime,
            status_message: None,
            scroll_offset: 0,
            confirm_action: None,
            last_refresh: Instant::now(),
            refresh_interval: Duration::from_secs(5),
        }
    }

    /// Rebuild the filtered and sorted index list.
    pub fn apply_filter_and_sort(&mut self) {
        let filter_lower = self.filter_text.to_lowercase();

        let mut indices: Vec<usize> = (0..self.tasks.len())
            .filter(|i| {
                if filter_lower.is_empty() {
                    return true;
                }
                let task = &self.tasks[*i];
                task.command_display.to_lowercase().contains(&filter_lower)
                    || task.pane.session_name.to_lowercase().contains(&filter_lower)
                    || task.pane.window_name.to_lowercase().contains(&filter_lower)
            })
            .collect();

        // Sort
        match self.sort_order {
            SortOrder::Runtime => {
                indices.sort_by_key(|i| {
                    std::cmp::Reverse(self.tasks[*i].runtime.unwrap_or(0))
                });
            }
            SortOrder::Session => {
                indices.sort_by_key(|i| {
                    (
                        self.tasks[*i].pane.session_name.clone(),
                        self.tasks[*i].pane.window_index,
                        self.tasks[*i].pane.pane_index,
                    )
                });
            }
            SortOrder::State => {
                indices.sort_by_key(|i| self.tasks[*i].state.as_str().to_string());
            }
        }

        self.filtered_indices = indices;

        // Clamp selection
        if self.filtered_indices.is_empty() {
            self.selection = 0;
        } else if self.selection >= self.filtered_indices.len() {
            self.selection = self.filtered_indices.len() - 1;
        }
    }

    pub fn move_up(&mut self) {
        if self.selection > 0 {
            self.selection -= 1;
        }
    }

    pub fn move_down(&mut self, max_visible: usize) {
        let max = self.filtered_indices.len().saturating_sub(1);
        if self.selection < max {
            self.selection += 1;
            if self.selection >= self.scroll_offset + max_visible {
                self.scroll_offset = self.selection + 1 - max_visible;
            }
        }
    }

    pub fn move_to_first(&mut self) {
        self.selection = 0;
        self.scroll_offset = 0;
    }

    pub fn move_to_last(&mut self, max_visible: usize) {
        let max = self.filtered_indices.len().saturating_sub(1);
        self.selection = max;
        self.scroll_offset = max.saturating_sub(max_visible.saturating_sub(1));
    }

    pub fn page_up(&mut self, page_size: usize) {
        self.selection = self.selection.saturating_sub(page_size);
        self.scroll_offset = self.scroll_offset.saturating_sub(page_size);
    }

    pub fn page_down(&mut self, page_size: usize, max_visible: usize) {
        let max = self.filtered_indices.len().saturating_sub(1);
        self.selection = (self.selection + page_size).min(max);
        if self.selection >= self.scroll_offset + max_visible {
            self.scroll_offset = self.selection + 1 - max_visible;
        }
    }

    pub fn selected_task(&self) -> Option<&Task> {
        self.filtered_indices
            .get(self.selection)
            .and_then(|i| self.tasks.get(*i))
    }

    pub fn enter_filter_mode(&mut self) {
        self.mode = AppMode::Filter;
    }

    pub fn exit_filter_mode(&mut self) {
        if self.mode == AppMode::Filter {
            self.mode = AppMode::Normal;
        }
    }

    pub fn append_filter(&mut self, c: char) {
        self.filter_text.push(c);
        self.apply_filter_and_sort();
    }

    pub fn backspace_filter(&mut self) {
        self.filter_text.pop();
        self.apply_filter_and_sort();
    }

    pub fn clear_filter(&mut self) {
        self.filter_text.clear();
        self.apply_filter_and_sort();
    }

    pub fn cycle_sort(&mut self) {
        self.sort_order = self.sort_order.next();
        self.apply_filter_and_sort();
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
    }

    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    pub fn should_refresh(&self) -> bool {
        self.last_refresh.elapsed() >= self.refresh_interval
    }

    pub fn mark_refreshed(&mut self) {
        self.last_refresh = Instant::now();
    }

    pub fn prompt_kill(&mut self) {
        if let Some(task) = self.selected_task() {
            self.confirm_action = Some(ConfirmAction::Kill(task.pane.pane_id.as_str().to_string()));
            self.mode = AppMode::Confirm;
        }
    }

    pub fn cancel_confirm(&mut self) {
        self.confirm_action = None;
        self.mode = AppMode::Normal;
    }

    pub fn confirm(&mut self) -> Option<ConfirmAction> {
        self.mode = AppMode::Normal;
        self.confirm_action.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::pane::{PaneId, PaneInfo};
    use crate::models::process::{ProcessInfo, ProcessState};
    use crate::models::task::Task;

    fn make_task(name: &str, session: &str, runtime: u64, state: TaskState) -> Task {
        Task {
            pane: PaneInfo {
                session_name: session.to_string(),
                session_id: "$0".to_string(),
                window_index: 0,
                window_name: "main".to_string(),
                pane_index: 0,
                pane_id: PaneId::new(name),
                pane_pid: 123,
                pane_active: false,
            },
            process: Some(ProcessInfo {
                pid: 123,
                command: "nvim".to_string(),
                args: vec![],
                start_time: Some(0),
                state: if state == TaskState::Running {
                    ProcessState::Running
                } else {
                    ProcessState::Dead
                },
            }),
            state,
            runtime: Some(runtime),
            command_display: "nvim".to_string(),
        }
    }

    #[test]
    fn test_navigation() {
        let tasks = vec![
            make_task("%0", "main", 100, TaskState::Running),
            make_task("%1", "main", 200, TaskState::Running),
            make_task("%2", "dev", 50, TaskState::Idle),
        ];

        let mut app = App::new(tasks);
        app.apply_filter_and_sort();

        assert_eq!(app.selection, 0);

        app.move_down(10);
        assert_eq!(app.selection, 1);

        app.move_down(10);
        assert_eq!(app.selection, 2);

        app.move_down(10);
        assert_eq!(app.selection, 2); // clamped

        app.move_up();
        assert_eq!(app.selection, 1);
    }

    #[test]
    fn test_filter() {
        let tasks = vec![
            make_task("%0", "main", 100, TaskState::Running),
            make_task("%1", "dev", 200, TaskState::Running),
        ];

        let mut app = App::new(tasks);
        app.apply_filter_and_sort();
        assert_eq!(app.filtered_indices.len(), 2);

        app.enter_filter_mode();
        app.append_filter('d'); // "d" matches "dev"
        assert_eq!(app.filter_text, "d");
        assert_eq!(app.filtered_indices.len(), 1);

        app.clear_filter();
        assert_eq!(app.filtered_indices.len(), 2);
    }

    #[test]
    fn test_sort_cycle() {
        assert_eq!(SortOrder::Runtime.next(), SortOrder::Session);
        assert_eq!(SortOrder::Session.next(), SortOrder::State);
        assert_eq!(SortOrder::State.next(), SortOrder::Runtime);
    }
}
