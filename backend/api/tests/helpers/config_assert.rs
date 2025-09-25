use serde_json::Value;

/// Assert that a config JSON equals the library defaults.
/// Pass the **object under "data"** from the /config response: `assert_default_config(&json["data"])`.
pub fn assert_default_config(d: &Value) {
    // ---------- execution ----------
    assert_eq!(d["execution"]["timeout_secs"], 30);
    assert_eq!(d["execution"]["max_memory"], 8_589_934_592u64);
    assert_eq!(d["execution"]["max_cpus"], 2);
    assert_eq!(d["execution"]["max_uncompressed_size"], 100_000_000u64);
    assert_eq!(d["execution"]["max_processes"], 256);

    // ---------- marking ----------
    assert_eq!(d["marking"]["marking_scheme"], "exact");
    assert_eq!(d["marking"]["feedback_scheme"], "auto");
    assert_eq!(d["marking"]["deliminator"], "###");
    assert_eq!(d["marking"]["grading_policy"], "last");
    assert_eq!(d["marking"]["max_attempts"], 10);
    assert_eq!(d["marking"]["limit_attempts"], false);
    assert_eq!(d["marking"]["pass_mark"], 50);
    assert_eq!(d["marking"]["allow_practice_submissions"], false);
    assert!(
        d["marking"]["dissalowed_code"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    // NEW: late policy defaults
    assert_eq!(d["marking"]["late"]["allow_late_submissions"], false);
    assert_eq!(d["marking"]["late"]["late_window_minutes"], 0);
    assert_eq!(d["marking"]["late"]["late_max_percent"], 100.0);

    // ---------- project ----------
    assert_eq!(d["project"]["language"], "cpp"); // serde rename_all = "lowercase"
    assert_eq!(d["project"]["submission_mode"], "manual");

    // ---------- output ----------
    assert_eq!(d["output"]["stdout"], true);
    assert_eq!(d["output"]["stderr"], false);
    assert_eq!(d["output"]["retcode"], false);

    // ---------- gatlam ----------
    assert_eq!(d["gatlam"]["population_size"], 100);
    assert_eq!(d["gatlam"]["number_of_generations"], 50);
    assert_eq!(d["gatlam"]["selection_size"], 20);

    approx(
        &d["gatlam"]["reproduction_probability"],
        0.8,
        "gatlam.reproduction_probability",
    );
    approx(
        &d["gatlam"]["crossover_probability"],
        0.9,
        "gatlam.crossover_probability",
    );
    approx(
        &d["gatlam"]["mutation_probability"],
        0.01,
        "gatlam.mutation_probability",
    );

    assert_eq!(d["gatlam"]["crossover_type"], "onepoint");
    assert_eq!(d["gatlam"]["mutation_type"], "bitflip");
    approx(&d["gatlam"]["omega1"], 0.5, "gatlam.omega1");
    approx(&d["gatlam"]["omega2"], 0.3, "gatlam.omega2");
    approx(&d["gatlam"]["omega3"], 0.2, "gatlam.omega3");

    // task_spec defaults
    assert_eq!(
        d["gatlam"]["task_spec"]["valid_return_codes"]
            .as_array()
            .unwrap(),
        &vec![Value::from(0u64)]
    );
    assert!(
        d["gatlam"]["task_spec"]["forbidden_outputs"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert!(d["gatlam"]["task_spec"]["max_runtime_ms"].is_null());

    // genes (2 entries)
    let genes = d["gatlam"]["genes"].as_array().unwrap();
    assert_eq!(genes.len(), 2);
    assert_eq!(genes[0]["min_value"], -5);
    assert_eq!(genes[0]["max_value"], 5);
    assert_eq!(genes[1]["min_value"], -4);
    assert_eq!(genes[1]["max_value"], 9);

    assert_eq!(d["gatlam"]["max_parallel_chromosomes"], 4);
    assert_eq!(d["gatlam"]["verbose"], false);

    // ---------- security ----------
    assert_eq!(d["security"]["password_enabled"], false);
    assert!(d["security"]["password_pin"].is_null());
    assert_eq!(d["security"]["cookie_ttl_minutes"], 480);
    assert_eq!(d["security"]["bind_cookie_to_user"], true);
    assert!(
        d["security"]["allowed_cidrs"]
            .as_array()
            .unwrap()
            .is_empty()
    );

    // ---------- code_coverage ----------
    assert_eq!(d["code_coverage"]["code_coverage_weight"], 10f32);
    assert!(
        d["code_coverage"]["whitelist"]
            .as_array()
            .unwrap()
            .is_empty(),
        "code_coverage.whitelist should default to an empty array"
    );
}
fn approx(v: &Value, expected: f64, path: &str) {
    let got = v.as_f64().unwrap();
    assert!(
        (got - expected).abs() < 1e-9,
        "float mismatch at {path}: got {got}, expected {expected}"
    );
}
