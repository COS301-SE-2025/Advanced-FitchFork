// manager/manager.rs
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration}; // Add this import

use crate::manager::queue::Queue;

pub struct ContainerManager {
    queue: Arc<Mutex<Queue>>,
}

impl ContainerManager {
    pub fn new(max_concurrent: usize) -> Self {
        Self {
            queue: Arc::new(Mutex::new(Queue::new(max_concurrent))),
        }
    }

    #[allow(dead_code)]
    pub fn clone(&self) -> Self {
        Self {
            queue: Arc::clone(&self.queue),
        }
    }

    /// Runs code, either immediately or after waiting in queue.
    /// Mock container run with a 2-second delay.
    pub async fn run(&self, language: &str, files: &[String]) -> String {
        let maybe_notify = {
            let mut queue = self.queue.lock().await;
            queue.try_acquire_slot()
        };

        // If we got a notify, wait for it outside the mutex
        if let Some(notify) = maybe_notify {
            notify.notified().await;
        }

        // Runmock container (just wait 2 seconds)
        tracing::info!("Running container for language: {}", language);
        sleep(Duration::from_secs(2)).await;

        // Release slot after run finishes
        {
            let mut queue = self.queue.lock().await;
            queue.release_slot();
        }

        format!(
            "Ran container for language '{}', files: {:?}",
            language, files
        )
    }

    /// A mock run method specifically for testing concurrency.
    #[allow(dead_code)]
    pub async fn run_mock(
        &self,
        language: &str,
        files: &[String],
        running_count: Arc<AtomicUsize>,
        max_observed_concurrent: Arc<AtomicUsize>,
    ) -> String {
        let maybe_notify = {
            let mut queue = self.queue.lock().await;
            queue.try_acquire_slot()
        };

        if let Some(notify) = maybe_notify {
            notify.notified().await;
        }

        let current = running_count.fetch_add(1, Ordering::SeqCst) + 1;
        max_observed_concurrent.fetch_max(current, Ordering::SeqCst);

        tracing::info!("Running container for language: {}", language);
        sleep(Duration::from_secs(2)).await;

        running_count.fetch_sub(1, Ordering::SeqCst);
        {
            let mut queue = self.queue.lock().await;
            queue.release_slot();
        }

        format!(
            "Ran container for language '{}', files: {:?}",
            language, files
        )
    }
}
