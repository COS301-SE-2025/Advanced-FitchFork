// manager/manager.rs
use crate::container::container::run_container;
use crate::manager::queue::Queue;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};
use util::execution_config::ExecutionConfig;

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
    pub async fn run(
        &self,
        config: &ExecutionConfig,
        commands: Vec<String>,
        files: Vec<(String, Vec<u8>)>,
        interpreter: bool,
    ) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync + 'static>> {
        let maybe_notify = {
            let mut queue = self.queue.lock().await;
            queue.try_acquire_slot()
        };

        if let Some(notify) = maybe_notify {
            notify.notified().await;
        }

        tracing::info!("Running container with commands: {:?}", commands);

        // Actually run the container
        let result = run_container(config, commands, files, interpreter).await;

        // Release slot after run finishes
        {
            let mut queue = self.queue.lock().await;
            queue.release_slot();
        }

        result
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

    pub async fn get_stats(&self) -> (usize, usize, usize) {
        let q = self.queue.lock().await;
        q.stats()
    }

    pub async fn set_max_concurrent(&self, new_max: usize) {
        let mut q = self.queue.lock().await;
        q.set_max_concurrent(new_max);
    }
}
