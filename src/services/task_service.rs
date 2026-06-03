use crate::models::process::{ProcessProvider, SystemProcessProvider};
use crate::models::task::{build_tasks, Task};
use log::info;
use std::collections::HashMap;
use std::time::{Duration, Instant};

struct CacheEntry {
    tasks: Vec<Task>,
    timestamp: Instant,
}

pub struct TaskService {
    process_provider: Box<dyn ProcessProvider>,
    cache: HashMap<String, CacheEntry>,
    ttl: Duration,
}

impl TaskService {
    pub fn new() -> Self {
        Self {
            process_provider: Box::new(SystemProcessProvider),
            cache: HashMap::new(),
            ttl: Duration::from_secs(2), // Cache for 2 seconds
        }
    }

    #[cfg(test)]
    pub fn with_provider(provider: Box<dyn ProcessProvider>) -> Self {
        Self {
            process_provider: provider,
            cache: HashMap::new(),
            ttl: Duration::from_secs(0), // No caching in tests
        }
    }

    pub fn discover_tasks(&mut self, current_pane_id: Option<&str>) -> anyhow::Result<Vec<Task>> {
        let cache_key = format!("{:?}", current_pane_id);

        // Check cache
        if let Some(entry) = self.cache.get(&cache_key) {
            if entry.timestamp.elapsed() < self.ttl {
                return Ok(entry.tasks.clone());
            }
        }

        // Cache miss — fetch fresh
        let panes = crate::services::tmux_service::list_panes()?;
        info!("Discovered {} panes", panes.len());

        let tasks = build_tasks(&panes, self.process_provider.as_ref(), current_pane_id);
        info!("Built {} tasks", tasks.len());

        // Update cache
        self.cache.insert(
            cache_key,
            CacheEntry {
                tasks: tasks.clone(),
                timestamp: Instant::now(),
            },
        );

        Ok(tasks)
    }
}

impl Default for TaskService {
    fn default() -> Self {
        Self::new()
    }
}
