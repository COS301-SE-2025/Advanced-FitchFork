//manager/queue.rs
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Notify;

pub struct Queue {
    max_concurrent: usize,
    running: usize,
    waiting: VecDeque<Arc<Notify>>,
}

impl Queue {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            max_concurrent,
            running: 0,
            waiting: VecDeque::new(),
        }
    }

    /// This methods is called when a job begins
    /// It tries to aquire a slot, if it cannot it waits
    pub fn try_acquire_slot(&mut self) -> Option<Arc<Notify>> {
        if self.running < self.max_concurrent {
            self.running += 1;
            None // Run instantly
        } else {
            let notify = Arc::new(Notify::new());
            self.waiting.push_back(notify.clone());
            Some(notify)
        }
    }

    /// This method is called when a job completes
    pub fn release_slot(&mut self) {
        self.running = self.running.saturating_sub(1);

        if let Some(waiting_task) = self.waiting.pop_front() {
            self.running += 1;
            waiting_task.notify_one();
        }
    }

    /// Returns current queue statistics
    pub fn stats(&self) -> (usize, usize, usize) {
        (self.running, self.waiting.len(), self.max_concurrent)
    }

    /// Updates the maximum concurrent slots allowed. If the new limit is higher
    /// than current running, this will immediately notify up to the difference
    /// worth of waiting tasks.
    pub fn set_max_concurrent(&mut self, new_max: usize) {
        self.max_concurrent = new_max.max(1);
        // Wake up waiting tasks if we have spare capacity now
        while self.running < self.max_concurrent {
            if let Some(waiting_task) = self.waiting.pop_front() {
                self.running += 1;
                waiting_task.notify_one();
            } else {
                break;
            }
        }
    }
}
