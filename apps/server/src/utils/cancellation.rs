use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

/// A shared registry that maps task IDs to cancellation flags.
///
/// - `register(task_id)` creates a flag and returns it (for the background thread to poll).
/// - `cancel(task_id)` sets the flag so the background thread stops at the next check point.
/// - `remove(task_id)` cleans up after the task finishes.
#[derive(Default)]
pub struct CancellationRegistry {
    flags: RwLock<HashMap<i32, Arc<AtomicBool>>>,
}

impl CancellationRegistry {
    pub fn new() -> Self {
        Self {
            flags: RwLock::new(HashMap::new()),
        }
    }

    /// Register a new cancellation flag for the given task. Returns the flag for the worker.
    pub fn register(&self, task_id: i32) -> Arc<AtomicBool> {
        let flag = Arc::new(AtomicBool::new(false));
        self.flags
            .write()
            .expect("cancellation registry lock poisoned")
            .insert(task_id, flag.clone());
        flag
    }

    /// Signal cancellation for the given task. Returns `true` if the task was found.
    pub fn cancel(&self, task_id: i32) -> bool {
        let guard = self
            .flags
            .read()
            .expect("cancellation registry lock poisoned");
        if let Some(flag) = guard.get(&task_id) {
            flag.store(true, Ordering::Relaxed);
            true
        } else {
            false
        }
    }

    /// Remove the flag entry once the task is done (completed, failed, or cancelled).
    pub fn remove(&self, task_id: i32) {
        self.flags
            .write()
            .expect("cancellation registry lock poisoned")
            .remove(&task_id);
    }

    /// Check whether cancellation was requested for the given task.
    pub fn is_cancelled(&self, task_id: i32) -> bool {
        let guard = self
            .flags
            .read()
            .expect("cancellation registry lock poisoned");
        guard
            .get(&task_id)
            .map(|f| f.load(Ordering::Relaxed))
            .unwrap_or(false)
    }
}
