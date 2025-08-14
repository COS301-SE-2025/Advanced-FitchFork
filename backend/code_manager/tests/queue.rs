//tests/queue.rs
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

    let running_count = Arc::new(AtomicUsize::new(0));
    let max_observed_concurrent = Arc::new(AtomicUsize::new(0));

    let handles: Vec<_> = (0..total_runs)
        .map(|i| {
            let mgr = manager.clone();
            let running_count_clone = Arc::clone(&running_count);
            let max_observed_clone = Arc::clone(&max_observed_concurrent);

            tokio::spawn(async move {
                let language = "rust";
                let files = vec![format!("file_{}.rs", i)];

                mgr.run_mock(language, &files, running_count_clone, max_observed_clone)
                    .await
            })
        })
        .collect();

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

    assert!(
        max_observed_concurrent.load(Ordering::SeqCst) <= max_concurrent,
        "Observed {} concurrent runs, but max should be {}",
        max_observed_concurrent.load(Ordering::SeqCst),
        max_concurrent
    );

    assert!(
        elapsed >= expected_min_duration - tolerance,
        "Elapsed time {:.2}s is too short, expected at least {:.2}s (with {:.2}s tolerance)",
        elapsed.as_secs_f64(),
        expected_min_duration.as_secs_f64(),
        tolerance.as_secs_f64()
    );

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
    let manager = ContainerManager::new(1);
    let start = Instant::now();

    let running_count = Arc::new(AtomicUsize::new(0));
    let max_observed_concurrent = Arc::new(AtomicUsize::new(0));

    let handles: Vec<_> = (0..3)
        .map(|i| {
            let mgr = manager.clone();
            let running_count_clone = Arc::clone(&running_count);
            let max_observed_clone = Arc::clone(&max_observed_concurrent);
            tokio::spawn(async move {
                mgr.run_mock(
                    "python",
                    &[format!("script_{}.py", i)],
                    running_count_clone,
                    max_observed_clone,
                )
                .await
            })
        })
        .collect();

    let results = futures::future::join_all(handles).await;
    let elapsed = start.elapsed();

    for (i, result) in results.into_iter().enumerate() {
        let output = result.expect("Task should not panic");
        assert!(output.contains(&format!("script_{}.py", i)));
    }

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
    let max_concurrent = 5;
    let total_runs = 3;
    let manager = ContainerManager::new(max_concurrent);

    let start = Instant::now();

    let running_count = Arc::new(AtomicUsize::new(0));
    let max_observed_concurrent = Arc::new(AtomicUsize::new(0));

    let handles: Vec<_> = (0..total_runs)
        .map(|i| {
            let mgr = manager.clone();
            let running_count_clone = Arc::clone(&running_count);
            let max_observed_clone = Arc::clone(&max_observed_concurrent);
            tokio::spawn(async move {
                mgr.run_mock(
                    "javascript",
                    &[format!("app_{}.js", i)],
                    running_count_clone,
                    max_observed_clone,
                )
                .await
            })
        })
        .collect();

    let results = futures::future::join_all(handles).await;
    let elapsed = start.elapsed();

    assert!(
        elapsed >= Duration::from_millis(1800),
        "Concurrent execution should take at least 1.8s"
    );
    assert!(
        elapsed <= Duration::from_millis(3000),
        "Concurrent execution took too long: {:.2}s",
        elapsed.as_secs_f64()
    );

    assert_eq!(results.len(), total_runs);
    for (i, result) in results.into_iter().enumerate() {
        let output = result.expect("Task should not panic");
        assert!(output.contains(&format!("app_{}.js", i)));
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_managers() {
    let manager1 = ContainerManager::new(2);
    let manager2 = ContainerManager::new(1);

    let running_count_1 = Arc::new(AtomicUsize::new(0));
    let max_observed_1 = Arc::new(AtomicUsize::new(0));
    let running_count_2 = Arc::new(AtomicUsize::new(0));
    let max_observed_2 = Arc::new(AtomicUsize::new(0));

    let start = Instant::now();

    let handles1: Vec<_> = (0..3)
        .map(|i| {
            let mgr = manager1.clone();
            let rc = Arc::clone(&running_count_1);
            let mo = Arc::clone(&max_observed_1);
            tokio::spawn(async move {
                mgr.run_mock("rust", &[format!("mgr1_task_{}.rs", i)], rc, mo)
                    .await
            })
        })
        .collect();

    let handles2: Vec<_> = (0..2)
        .map(|i| {
            let mgr = manager2.clone();
            let rc = Arc::clone(&running_count_2);
            let mo = Arc::clone(&max_observed_2);
            tokio::spawn(async move {
                mgr.run_mock("python", &[format!("mgr2_task_{}.py", i)], rc, mo)
                    .await
            })
        })
        .collect();

    let (results1, results2) = tokio::join!(
        futures::future::join_all(handles1),
        futures::future::join_all(handles2)
    );

    let elapsed = start.elapsed();

    assert!(elapsed >= Duration::from_millis(3500));
    assert!(
        elapsed <= Duration::from_millis(5000),
        "Independent managers took too long: {:.2}s",
        elapsed.as_secs_f64()
    );

    assert_eq!(results1.len(), 3);
    assert_eq!(results2.len(), 2);

    for (i, result) in results1.into_iter().enumerate() {
        let output = result.expect("Task should not panic");
        assert!(output.contains(&format!("mgr1_task_{}.rs", i)));
    }
    for (i, result) in results2.into_iter().enumerate() {
        let output = result.expect("Task should not panic");
        assert!(output.contains(&format!("mgr2_task_{}.py", i)));
    }
}
