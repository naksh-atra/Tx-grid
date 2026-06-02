use crate::models::process::{ProcessProvider, SystemProcessProvider};
use crate::models::task::{build_tasks, Task};
use log::info;

pub struct TaskService {
    process_provider: Box<dyn ProcessProvider>,
}

impl TaskService {
    pub fn new() -> Self {
        Self {
            process_provider: Box::new(SystemProcessProvider),
        }
    }

    #[cfg(test)]
    pub fn with_provider(provider: Box<dyn ProcessProvider>) -> Self {
        Self {
            process_provider: provider,
        }
    }

    pub fn discover_tasks(&self, current_pane_id: Option<&str>) -> anyhow::Result<Vec<Task>> {
        let panes = crate::services::tmux_service::list_panes()?;
        info!("Discovered {} panes", panes.len());

        let tasks = build_tasks(&panes, self.process_provider.as_ref(), current_pane_id);
        info!("Built {} tasks", tasks.len());

        Ok(tasks)
    }
}

impl Default for TaskService {
    fn default() -> Self {
        Self::new()
    }
}
