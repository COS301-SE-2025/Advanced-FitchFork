use code_manager::manager::manager::ContainerManager;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::time::{Duration, Instant};

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_queue_respects_max_concurrency() {
    let max_concurrent = 2;
    let manager = ContainerManager::new(max_concurrent);
    let total_runs = 5;
    let expected_min_duration = Duration::from_secs(6); // ceil(5/2) * 2 = 6 seconds
    let tolerance = Duration::from_millis(500); // Allow 500ms variance

    let start = Instant::now();

    // Track actual concurrency during execution
    let running_count = Arc::new(AtomicUsize::new(0));
    let max_observed_concurrent = Arc::new(AtomicUsize::new(0));

    // Spawn multiple concurrent run requests
    let handles: Vec<_> = (0..total_runs)
        .map(|i| {
            let mgr = manager.clone();
            let running_count_clone = Arc::clone(&running_count);
            let max_observed_clone = Arc::clone(&max_observed_concurrent);

            tokio::spawn(async move {
                let language = "rust";
                let files = vec![format!("file_{}.rs", i)];

                // Call the new mock method that handles tracking internally
                mgr.run_mock(language, &files, running_count_clone, max_observed_clone)
                    .await
            })
        })
        .collect();

    // Wait for all to finish
    let results = futures::future::join_all(handles).await;
    let elapsed = start.elapsed();

    println!(
        "All {} runs finished in {:.2} seconds",
        total_runs,
        elapsed.as_secs_f64()
    );
    println!(
        "Max observed concurrent: {}",
        max_observed_concurrent.load(Ordering::SeqCst)
    );

    // Verify all tasks completed successfully
    for (i, result) in results.into_iter().enumerate() {
        let output = result.expect("Task should not panic");
        assert!(
            output.contains(&format!("file_{}.rs", i)),
            "Output should contain correct filename: {}",
            output
        );
        assert!(
            output.contains("Ran container for language 'rust'"),
            "Output should contain expected format: {}",
            output
        );
    }

    // Verify concurrency was respected
    assert!(
        max_observed_concurrent.load(Ordering::SeqCst) <= max_concurrent,
        "Observed {} concurrent runs, but max should be {}",
        max_observed_concurrent.load(Ordering::SeqCst),
        max_concurrent
    );

    // Verify timing constraints (with tolerance for test environment variance)
    assert!(
        elapsed >= expected_min_duration - tolerance,
        "Elapsed time {:.2}s is too short, expected at least {:.2}s (with {:.2}s tolerance)",
        elapsed.as_secs_f64(),
        expected_min_duration.as_secs_f64(),
        tolerance.as_secs_f64()
    );

    // Reasonable upper bound to catch infinite waits
    let max_expected = Duration::from_secs(10);
    assert!(
        elapsed <= max_expected,
        "Elapsed time {:.2}s is too long, expected at most {:.2}s",
        elapsed.as_secs_f64(),
        max_expected.as_secs_f64()
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_queue_handles_different_concurrency_limits() {
    // Test with max_concurrent = 1 (serial execution)
    let manager = ContainerManager::new(1);
    let start = Instant::now();

    let handles: Vec<_> = (0..3)
        .map(|i| {
            let mgr = manager.clone();
            tokio::spawn(async move { mgr.run("python", &[format!("script_{}.py", i)]).await })
        })
        .collect();

    futures::future::join_all(handles).await;
    let elapsed = start.elapsed();

    // With max_concurrent=1, 3 runs should take ~6 seconds (3 * 2s each)
    assert!(
        elapsed >= Duration::from_millis(5500),
        "Serial execution should take at least 5.5s, got {:.2}s",
        elapsed.as_secs_f64()
    );
    assert!(
        elapsed <= Duration::from_secs(8),
        "Serial execution took too long: {:.2}s",
        elapsed.as_secs_f64()
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_queue_with_no_waiting() {
    // Test case where all requests can run immediately
    let max_concurrent = 5;
    let total_runs = 3; // Less than max_concurrent
    let manager = ContainerManager::new(max_concurrent);

    let start = Instant::now();

    let handles: Vec<_> = (0..total_runs)
        .map(|i| {
            let mgr = manager.clone();
            tokio::spawn(async move { mgr.run("javascript", &[format!("app_{}.js", i)]).await })
        })
        .collect();

    let results = futures::future::join_all(handles).await;
    let elapsed = start.elapsed();

    // All should run concurrently, so total time should be ~2 seconds
    assert!(
        elapsed >= Duration::from_millis(1800),
        "Concurrent execution should take at least 1.8s"
    );
    assert!(
        elapsed <= Duration::from_millis(3000),
        "Concurrent execution took too long: {:.2}s",
        elapsed.as_secs_f64()
    );

    // Verify all completed successfully
    assert_eq!(results.len(), total_runs);
    for result in results {
        assert!(result.is_ok());
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_queue_fairness() {
    // Test that tasks are processed in FIFO order
    let manager = ContainerManager::new(1); // Force serial execution

    let completion_order = Arc::new(tokio::sync::Mutex::new(Vec::new()));

    let handles: Vec<_> = (0..5)
        .map(|i| {
            let mgr = manager.clone();
            let order = Arc::clone(&completion_order);
            tokio::spawn(async move {
                let result = mgr.run("rust", &[format!("task_{}.rs", i)]).await;
                order.lock().await.push(i);
                result
            })
        })
        .collect();

    futures::future::join_all(handles).await;

    let final_order = completion_order.lock().await;
    println!("Completion order: {:?}", *final_order);

    // Should complete in order 0, 1, 2, 3, 4
    assert_eq!(
        *final_order,
        vec![0, 1, 2, 3, 4],
        "Tasks should complete in FIFO order"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_managers() {
    // Test that multiple manager instances work independently
    let manager1 = ContainerManager::new(2);
    let manager2 = ContainerManager::new(1);

    let start = Instant::now();

    // Manager 1 runs 3 tasks (should take ~4 seconds: 2 parallel + 1 sequential)
    let handles1: Vec<_> = (0..3)
        .map(|i| {
            let mgr = manager1.clone();
            tokio::spawn(async move { mgr.run("rust", &[format!("mgr1_task_{}.rs", i)]).await })
        })
        .collect();

    // Manager 2 runs 2 tasks serially (should take ~4 seconds)
    let handles2: Vec<_> = (0..2)
        .map(|i| {
            let mgr = manager2.clone();
            tokio::spawn(async move { mgr.run("python", &[format!("mgr2_task_{}.py", i)]).await })
        })
        .collect();

    // Both should complete around the same time since they run independently
    let (results1, results2) = tokio::join!(
        futures::future::join_all(handles1),
        futures::future::join_all(handles2)
    );

    let elapsed = start.elapsed();

    // Should take around 4 seconds (not 6-8 if they were blocking each other)
    assert!(elapsed >= Duration::from_millis(3500));
    assert!(
        elapsed <= Duration::from_millis(5000),
        "Independent managers took too long: {:.2}s",
        elapsed.as_secs_f64()
    );

    assert_eq!(results1.len(), 3);
    assert_eq!(results2.len(), 2);
}
