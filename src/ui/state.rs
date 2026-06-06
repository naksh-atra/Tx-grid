use crate::models::task::{Task, TaskState};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppMode {
    Normal,
    Filter,
    Confirm,
    Rename,
    Notes,
}

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
    pub rename_text: String,
    pub notes_text: String,
    pub notes_pane_id: Option<String>,
    pub last_refresh: Instant,
    pub refresh_interval: Duration,
}

pub enum ConfirmAction {
    Kill(String),
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
            rename_text: String::new(),
            notes_text: String::new(),
            notes_pane_id: None,
            last_refresh: Instant::now(),
            refresh_interval: Duration::from_secs(5),
        }
    }

    pub fn apply_filter_and_sort(&mut self) {
        let filter_lower = self.filter_text.to_lowercase();
        let mut indices: Vec<usize> = (0..self.tasks.len())
            .filter(|i| {
                if filter_lower.is_empty() {
                    return true;
                }
                let task = &self.tasks[*i];
                task.command_display.to_lowercase().contains(&filter_lower)
                    || task
                        .pane
                        .session_name
                        .to_lowercase()
                        .contains(&filter_lower)
                    || task.pane.window_name.to_lowercase().contains(&filter_lower)
            })
            .collect();

        match self.sort_order {
            SortOrder::Runtime => {
                indices.sort_by_key(|i| std::cmp::Reverse(self.tasks[*i].runtime.unwrap_or(0)));
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
        if self.filtered_indices.is_empty() {
            self.selection = 0;
        } else if self.selection >= self.filtered_indices.len() {
            self.selection = self.filtered_indices.len() - 1;
        }
    }

    pub fn move_up(&mut self, max_visible: usize) {
        if self.selection > 0 {
            self.selection -= 1;
            if self.selection < self.scroll_offset {
                self.scroll_offset = self.selection;
            }
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
        if self.filter_text.len() < 256 {
            self.filter_text.push(c);
            self.apply_filter_and_sort();
        }
    }

    pub fn backspace_filter(&mut self) {
        self.filter_text.pop();
        self.apply_filter_and_sort();
    }

    pub fn clear_filter(&mut self) {
        self.filter_text.clear();
        self.apply_filter_and_sort();
    }

    pub fn enter_rename_mode(&mut self) {
        if let Some(task) = self.selected_task() {
            // Use existing pane title if set, otherwise start with empty string
            // (user sees a clean input, not the internal pane_id like %0)
            let current = &task.pane.pane_title;
            self.rename_text = if current.is_empty() {
                // Fallback: use session:window as a hint
                format!("{}:{}", task.pane.session_name, task.pane.window_name)
            } else {
                current.clone()
            };
            self.mode = AppMode::Rename;
        }
    }

    pub fn exit_rename_mode(&mut self) {
        self.rename_text.clear();
        self.mode = AppMode::Normal;
    }

    pub fn append_rename(&mut self, c: char) {
        if self.rename_text.len() < 64 {
            self.rename_text.push(c);
        }
    }

    pub fn backspace_rename(&mut self) {
        self.rename_text.pop();
    }

    pub fn get_rename_text(&self) -> &str {
        &self.rename_text
    }

    pub fn enter_notes_mode(&mut self) {
        if let Some(task) = self.selected_task() {
            let pane_id = task.pane.pane_id.as_str().to_string();
            self.notes_pane_id = Some(pane_id.clone());
            self.notes_text = load_notes(&pane_id);
            self.mode = AppMode::Notes;
        }
    }

    pub fn exit_notes_mode(&mut self) {
        if let Some(ref pane_id) = self.notes_pane_id {
            save_notes(pane_id, &self.notes_text);
        }
        self.notes_text.clear();
        self.notes_pane_id = None;
        self.mode = AppMode::Normal;
    }

    pub fn append_notes(&mut self, c: char) {
        if self.notes_text.len() < 1024 {
            self.notes_text.push(c);
        }
    }

    pub fn backspace_notes(&mut self) {
        self.notes_text.pop();
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

fn notes_file_path() -> std::path::PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    std::path::PathBuf::from(format!("{}/.tmux-taskgrid-notes", home))
}

fn load_notes(pane_id: &str) -> String {
    let path = notes_file_path();
    if !path.exists() {
        return String::new();
    }
    if let Ok(content) = std::fs::read_to_string(&path) {
        for line in content.lines() {
            if let Some((id, note)) = line.split_once('\t') {
                if id == pane_id {
                    return note.to_string();
                }
            }
        }
    }
    String::new()
}

fn save_notes(pane_id: &str, note: &str) {
    let path = notes_file_path();
    let mut lines: Vec<String> = Vec::new();

    if path.exists() {
        if let Ok(existing) = std::fs::read_to_string(&path) {
            for line in existing.lines() {
                if let Some((id, _)) = line.split_once('\t') {
                    if id != pane_id {
                        lines.push(line.to_string());
                    }
                }
            }
        }
    }

    if !note.trim().is_empty() {
        lines.push(format!("{}\t{}", pane_id, note.trim()));
    }

    let _ = std::fs::write(&path, lines.join("\n") + "\n");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::pane::{PaneId, PaneInfo};
    use crate::models::process::{ProcessInfo, ProcessState};
    use crate::models::task::{Task, TaskState};

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
                activity_at: None,
                pane_title: String::new(),
            },
            process: Some(ProcessInfo {
                pid: 123,
                command: "nvim".into(),
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
        assert_eq!(app.selection, 2);
        app.move_up(10);
        assert_eq!(app.selection, 1);
    }

    #[test]
    fn test_scroll_down_and_up() {
        // Simulate many tasks with small visible area
        let mut tasks = Vec::new();
        for i in 0..30 {
            tasks.push(make_task(
                &format!("%{}", i),
                "main",
                (i * 100) as u64,
                TaskState::Running,
            ));
        }
        let mut app = App::new(tasks);
        app.apply_filter_and_sort();
        assert_eq!(app.selection, 0);
        assert_eq!(app.scroll_offset, 0);

        // Scroll down past visible area (max_visible=5)
        for _ in 0..6 {
            app.move_down(5);
        }
        // Selection should be >= 5, scroll_offset should have moved
        assert!(app.selection >= 5);
        assert!(app.scroll_offset > 0);

        // Now scroll back up
        for _ in 0..6 {
            app.move_up(5);
        }
        // Should be back at top
        assert_eq!(app.selection, 0);
        assert_eq!(app.scroll_offset, 0);
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
        app.append_filter('d');
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

    #[test]
    fn test_rename_mode() {
        let tasks = vec![make_task("%0", "main", 100, TaskState::Running)];
        let mut app = App::new(tasks);
        assert_eq!(app.mode, AppMode::Normal);
        app.enter_rename_mode();
        assert_eq!(app.mode, AppMode::Rename);
        assert_eq!(app.get_rename_text(), "main:main");
        app.append_rename('x');
        assert_eq!(app.get_rename_text(), "main:mainx");
        app.backspace_rename();
        assert_eq!(app.get_rename_text(), "main:main");
        app.exit_rename_mode();
        assert_eq!(app.mode, AppMode::Normal);
    }

    #[test]
    fn test_notes_mode() {
        let path = notes_file_path();
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
        let tasks = vec![make_task("%0", "main", 100, TaskState::Running)];
        let mut app = App::new(tasks);
        assert_eq!(app.mode, AppMode::Normal);
        app.enter_notes_mode();
        assert_eq!(app.mode, AppMode::Notes);
        app.append_notes('h');
        app.append_notes('i');
        assert_eq!(app.notes_text, "hi");
        app.backspace_notes();
        assert_eq!(app.notes_text, "h");
        app.exit_notes_mode();
        assert_eq!(app.mode, AppMode::Normal);
        assert!(app.notes_text.is_empty());
        if path.exists() {
            let _ = std::fs::remove_file(&path);
        }
    }
}
