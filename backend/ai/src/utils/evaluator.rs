// ai/src/utils/evaluator.rs

#[derive(Debug, Clone, Copy)]
pub enum Language {
    Cpp,
    Java, 
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Property {
    Safety,            // G(¬unsafe)
    ProperTermination, // G(ter => VRC)
    SegmentationFault, // G(¬segfault)
    Exceptions,        // G(¬exception)
    ExecutionTime,     // G(ter => (r ≤ e))
    IllegalOutput,     // G(ter => ∀o∈Out ∀x∈X (x ≠ o))
    // ExpectedExact,   // handled elsewhere
    // ExpectedContains,// handled elsewhere
    // for these last two LTL properties we might need to handle them in this actual code, especially contains, but for now it should be fine :)

}

#[derive(Debug, Clone)]
pub struct TaskSpec {
    pub language: Language,
    /// Valid return codes for "proper termination" (default: [0])
    pub valid_return_codes: Option<Vec<i32>>,
    /// Execution-time bound in milliseconds (r ≤ e)
    pub max_runtime_ms: Option<u64>,
    /// For IllegalOutput: forbidden outputs X (exact line matches after trim)
    pub forbidden_outputs: Vec<String>,
}

impl Default for TaskSpec {
    fn default() -> Self {
        Self {
            language: Language::Cpp,
            valid_return_codes: Some(vec![0]),
            max_runtime_ms: None,
            forbidden_outputs: vec![],
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct TaskView {
    pub task_id: i64,
    pub exit_code: Option<i32>,
    pub runtime_ms: Option<u64>,
    pub stdout: String,
    pub stderr: String,
    pub terminated: bool,
}

#[derive(Debug, Clone, Default)]
pub struct TaskEvaluation {
    pub task_id: i64,
    /// properties that were violated for this task
    pub violated: Vec<Property>,
}

pub struct Evaluator;

impl Evaluator {
    pub fn new() -> Self { Self }

    /// Parse raw blob into a structured view.
    /// Expected markers (case-insensitive):
    ///   EXIT_CODE: <n>  or EXIT_CODE=<n>
    ///   RUNTIME_MS: <n> or RUNTIME_MS=<n>
    ///   STDERR:         (starts stderr section to EOF)
    pub fn parse(&self, task_id: i64, blob: &str) -> TaskView {
        let (exit_code, stdout, stderr) = split_exit_stdout_stderr(blob);
        let runtime_ms = extract_marker_int(blob, "runtime_ms").map(|v| v.max(0) as u64);
        let terminated = exit_code.is_some();
        TaskView { task_id, exit_code, runtime_ms, stdout, stderr, terminated }
    }

    /// Evaluate a single task against the selected property set (excluding expected-output checks).
    pub fn evaluate_task(&self, spec: &TaskSpec, view: &TaskView) -> TaskEvaluation {
        let mut violated = Vec::new();

        // Safety: G(¬unsafe)
        if violates_safety(spec.language, &view.stderr) {
            violated.push(Property::Safety);
        }

        // Proper Termination: G(ter => VRC)
        if view.terminated {
            let ok = is_valid_return_code(view.exit_code, spec.valid_return_codes.as_deref());
            if !ok {
                violated.push(Property::ProperTermination);
            }
        }

        // Segmentation Fault: G(¬segfault)
        if has_segfault(spec.language, &view.stderr) {
            violated.push(Property::SegmentationFault);
        }

        // Exceptions: G(¬exception)
        if has_exception(spec.language, &view.stderr) {
            violated.push(Property::Exceptions);
        }

        // Execution Time: G(ter => (r ≤ e))
        if view.terminated {
            if let Some(bound) = spec.max_runtime_ms {
                if let Some(r) = view.runtime_ms {
                    if r > bound {
                        violated.push(Property::ExecutionTime);
                    }
                }
            }
        }

        // Illegal Output: G(ter => ∀o∈Out ∀x∈X (x ≠ o))
        if view.terminated && !spec.forbidden_outputs.is_empty() {
            let outs = normalized_lines(&view.stdout);
            let forb = spec.forbidden_outputs.iter().map(|s| s.trim().to_string()).collect::<Vec<_>>();
            if outs.iter().any(|o| forb.iter().any(|x| x == o)) {
                violated.push(Property::IllegalOutput);
            }
        }

        TaskEvaluation { task_id: view.task_id, violated }
    }

    /// Helper for GA.
    /// Returns milli-fractions in 0..=1000:
    ///   (ltl_milli, fail_milli)
    /// - ltl_milli: fraction of violated LTL-ish properties across all tasks checked
    /// - fail_milli: fraction of tasks considered failed
    pub fn derive_props(&self, specs: &[TaskSpec], outs: &[(i64, String)]) -> (usize, usize) {
        let total_tasks = outs.len().max(1);

        let mut ltl_checks      = 0usize;
        let mut ltl_violations  = 0usize;
        let mut failed_tasks    = 0usize;

        for (i, (task_id, blob)) in outs.iter().enumerate() {
            let spec = specs.get(i).unwrap_or_else(|| specs.first().expect("non-empty specs"));
            let view = self.parse(*task_id, blob);
            let eval = self.evaluate_task(spec, &view);

            // Count how many checks we applied for this task (exclude the output-based ones handled elsewhere).
            // Here we’re using: Safety, ProperTermination, SegmentationFault, Exceptions, ExecutionTime, IllegalOutput
            // Note: ExecutionTime and IllegalOutput are only applicable if terminated and configured accordingly.
            let mut checks = 0usize;
            let mut viols  = 0usize;

            // Safety
            checks += 1;
            if eval.violated.contains(&Property::Safety) { viols += 1; }

            // ProperTermination
            checks += 1;
            if eval.violated.contains(&Property::ProperTermination) { viols += 1; }

            // Segfault
            checks += 1;
            if eval.violated.contains(&Property::SegmentationFault) { viols += 1; }

            // Exceptions
            checks += 1;
            if eval.violated.contains(&Property::Exceptions) { viols += 1; }

            // ExecutionTime (only counted if there was a bound and task terminated with measured runtime) TODO: we have to time how long the task took and parse it back to here
            if spec.max_runtime_ms.is_some() && view.terminated && view.runtime_ms.is_some() {
                checks += 1;
                if eval.violated.contains(&Property::ExecutionTime) { viols += 1; }
            }

            // IllegalOutput (only if there are forbidden outputs configured and task terminated)
            if !spec.forbidden_outputs.is_empty() && view.terminated {
                checks += 1;
                if eval.violated.contains(&Property::IllegalOutput) { viols += 1; }
            }

            ltl_checks     += checks;
            ltl_violations += viols;

            let ret_ok = is_valid_return_code(view.exit_code, spec.valid_return_codes.as_deref());
            let failed = !ret_ok
                || view.terminated && has_segfault(spec.language, &view.stderr)
                || view.terminated && has_exception(spec.language, &view.stderr)
                || self.contains_forbidden_output(&view.stdout, &spec.forbidden_outputs);

            if failed { failed_tasks += 1; }
        }

        let ltl_milli  = if ltl_checks == 0 { 0 } else { ((ltl_violations * 1000) / ltl_checks).min(1000) };
        let fail_milli = ((failed_tasks   * 1000) / total_tasks).min(1000);

        (ltl_milli, fail_milli)
    }

    fn contains_forbidden_output(&self, stdout: &str, forbidden: &[String]) -> bool {
        if forbidden.is_empty() { return false; }
        let hay = stdout.to_ascii_lowercase();
        forbidden.iter().any(|needle| hay.contains(&needle.to_ascii_lowercase()))
    }

}

fn split_exit_stdout_stderr(blob: &str) -> (Option<i32>, String, String) {
    let mut exit_code: Option<i32> = None;
    let mut stderr = String::new();
    let mut stdout = String::new();

    // EXIT_CODE
    if let Some(v) = extract_marker_int(blob, "exit_code") {
        exit_code = Some(v);
    } else if let Some(v) = extract_marker_int(blob, "exit code") {
        exit_code = Some(v);
    }

    // STDERR section
    if let Some(pos) = find_case_insensitive(blob, "stderr:") {
        let (head, tail) = blob.split_at(pos);
        stdout.push_str(head.trim_end());
        let label_len = tail.chars().take_while(|c| c.is_alphabetic()).count(); // "STDERR"
        let rest = tail[label_len..].trim_start_matches(':').trim_start();
        stderr.push_str(rest);
    } else {
       // Heuristic for errors if no STDERR section is found
        // This is a fallback for cases where the output does not follow the expected format.
        // We assume that if the blob contains error-like messages, they should go to stderr.
        let lower = blob.to_ascii_lowercase();
        let looks_error = [
            "error:", "exception", "segmentation fault", "sigsegv",
            "addresssanitizer", "asan", "double free", "invalid pointer",
            "use-after-free", "heap-use-after-free", "free(): invalid pointer",
            "munmap_chunk(): invalid pointer",
        ].iter().any(|needle| lower.contains(needle));
        if looks_error { stderr.push_str(blob); } else { stdout.push_str(blob); }
    }

    (exit_code, stdout.trim().to_string(), stderr.trim().to_string())
}

fn extract_marker_int(blob: &str, key: &str) -> Option<i32> {
    let key_lower = key.to_ascii_lowercase();
    for line in blob.lines() {
        let l = line.trim();
        let ll = l.to_ascii_lowercase();
        if let Some(idx) = ll.find(&key_lower) {
            let after = &ll[idx + key_lower.len()..];
            // strip separators
            let after = after.trim_start_matches(|c: char| c == ':' || c == '=' || c.is_whitespace());
            // take signed int prefix
            let mut end = 0;
            for ch in after.chars() {
                if ch.is_ascii_digit() || ch == '-' || ch == '+' { end += ch.len_utf8(); } else { break; }
            }
            if end > 0 {
                if let Ok(v) = after[..end].parse::<i32>() {
                    return Some(v);
                }
            }
        }
    }
    None
}

fn find_case_insensitive(haystack: &str, needle: &str) -> Option<usize> {
    let h = haystack.as_bytes();
    let n = needle.as_bytes();
    if n.is_empty() { return Some(0); }
    for i in 0..=h.len().saturating_sub(n.len()) {
        if h[i..i+n.len()].iter().zip(n.iter())
            .all(|(a,b)| a.to_ascii_lowercase() == b.to_ascii_lowercase()) {
            return Some(i);
        }
    }
    None
}


fn normalized_lines(s: &str) -> Vec<String> {
    s.lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .map(|l| l.to_string())
        .collect()
}


fn violates_safety(lang: Language, stderr: &str) -> bool {
    let s = stderr.to_ascii_lowercase();
    // IF YOU WANT TO ADD SUPPORT FOR OTHER LANGUAGES, ADD THEM HERE
    match lang {
        Language::Cpp => {
            s.contains("double free") ||
            s.contains("double-free") ||
            s.contains("invalid pointer") ||
            s.contains("use-after-free") ||
            s.contains("heap-use-after-free") ||
            s.contains("segmentation fault") ||
            s.contains("sigsegv") ||
            s.contains("addresssanitizer") ||
            s.contains("asan:")
        }
        Language::Java => {
            s.contains("hs_err_pid")                      // JVM fatal log header
                || s.contains("a fatal error has been detected by the java runtime environment")
                || s.contains("sigsegv")                  // native segv bubbled up by JVM
                || s.contains("exception_access_violation")
                || s.contains("problematic frame:")
                || s.contains("outofmemoryerror: direct buffer memory") // catastrophic OOM kind
                || s.contains("internal error (")         // hotspot internal error
        }
    }
}


fn has_segfault(lang: Language, stderr: &str) -> bool {
    let s = stderr.to_ascii_lowercase();
    match lang {
        Language::Cpp => s.contains("segmentation fault") || s.contains("sigsegv"),
        Language::Java => {
            s.contains("sigsegv")
                || s.contains("exception_access_violation")
                || s.contains("hs_err_pid")
                || s.contains("problematic frame:")

        }
    }
}

fn has_exception(lang: Language, stderr: &str) -> bool {
    let s = stderr.to_ascii_lowercase();
    match lang {
        Language::Cpp => {
            s.contains("exception")
                || s.contains("terminate called")
                || s.contains("std::terminate")
                || s.contains("std::bad_alloc")
        }
        Language::Java => {
            s.contains("exception in thread")
                || s.contains("java.lang.exception")
                || s.contains("java.lang.runtimeexception")
                || s.contains("java.lang.nullpointerexception")
                || s.contains("java.lang.illegalargumentexception")
                || s.contains("java.lang.indexoutofboundsexception")
                || s.contains("java.lang.arrayindexoutofboundsexception")
                || s.contains("java.lang.outofmemoryerror")
                || s.contains("java.lang.stackoverflowerror")
                || s.contains("exception:")
                || s.contains("error:")
        }
    }
}

fn is_valid_return_code(exit: Option<i32>, valid: Option<&[i32]>) -> bool {
    match (exit, valid) {
        (Some(code), Some(list)) => list.contains(&code),
        (Some(0), None)          => true,
        (Some(_), None)          => false,
        (None, _)                => false, 
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn spec_cpp() -> TaskSpec {
        TaskSpec {
            language: Language::Cpp,
            valid_return_codes: Some(vec![0]),
            max_runtime_ms: None,
            forbidden_outputs: vec![],
        }
    }

    fn spec_cpp_with_time(bound: u64) -> TaskSpec {
        TaskSpec { max_runtime_ms: Some(bound), ..spec_cpp() }
    }

    fn spec_cpp_with_forbidden(forb: &[&str]) -> TaskSpec {
        TaskSpec {
            forbidden_outputs: forb.iter().map(|s| s.to_string()).collect(),
            ..spec_cpp()
        }
    }

    #[test]
    fn extract_marker_int_handles_colon_and_equals() {
        let blob = "EXIT_CODE: 0\nRUNTIME_MS=123\n";
        assert_eq!(super::extract_marker_int(blob, "exit_code"), Some(0));
        assert_eq!(super::extract_marker_int(blob, "RUNTIME_MS"), Some(123));
    }

    #[test]
    fn split_exit_stdout_stderr_with_explicit_stderr() {
        let blob = "hello\nSTDERR: bad stuff\nline2";
        let (exit, out, err) = super::split_exit_stdout_stderr(blob);
        assert_eq!(exit, None);
        assert_eq!(out, "hello");
        assert_eq!(err, "bad stuff\nline2");
    }

    #[test]
    fn split_heuristic_sends_errors_to_stderr() {
        let blob = "Segmentation fault (core dumped)";
        let (exit, out, err) = super::split_exit_stdout_stderr(blob);
        assert_eq!(exit, None);
        assert!(out.is_empty());
        assert!(err.to_ascii_lowercase().contains("segmentation fault"));
    }

    #[test]
    fn parse_sets_terminated_if_exit_code_present() {
        let ev = Evaluator::new();
        let view = ev.parse(42, "EXIT_CODE: 0\nhello\n");
        assert_eq!(view.task_id, 42);
        assert_eq!(view.exit_code, Some(0));
        assert!(view.terminated);
        assert_eq!(view.stdout.trim(), "EXIT_CODE: 0\nhello"); // before any “STDERR:” marker
    }

    #[test]
    fn parse_runtime_ms_parsed_to_u64() {
        let ev = Evaluator::new();
        let view = ev.parse(1, "RUNTIME_MS=200\nEXIT_CODE=0\n");
        assert_eq!(view.runtime_ms, Some(200));
        assert!(view.terminated);
    }

    #[test]
    fn proper_termination_ok_when_zero_exit() {
        let ev = Evaluator::new();
        let spec = spec_cpp();
        let view = ev.parse(1, "EXIT_CODE: 0\n");
        let eval = ev.evaluate_task(&spec, &view);
        assert!(!eval.violated.contains(&Property::ProperTermination));
    }

    #[test]
    fn proper_termination_violates_on_nonzero_exit() {
        let ev = Evaluator::new();
        let spec = spec_cpp();
        let view = ev.parse(1, "EXIT_CODE=2\n");
        let eval = ev.evaluate_task(&spec, &view);
        assert!(eval.violated.contains(&Property::ProperTermination));
    }

    #[test]
    fn safety_detects_asan_and_uaf() {
        let ev = Evaluator::new();
        let spec = spec_cpp();
        let view = ev.parse(1, "STDERR: AddressSanitizer: heap-use-after-free\n");
        let eval = ev.evaluate_task(&spec, &view);
        assert!(eval.violated.contains(&Property::Safety));
    }

    #[test]
    fn segfault_detected() {
        let ev = Evaluator::new();
        let spec = spec_cpp();
        let view = ev.parse(1, "STDERR: Segmentation fault\n");
        let eval = ev.evaluate_task(&spec, &view);
        assert!(eval.violated.contains(&Property::SegmentationFault));
    }

    #[test]
    fn exception_detected() {
        let ev = Evaluator::new();
        let spec = spec_cpp();
        let view = ev.parse(1, "STDERR: terminate called after throwing an instance of 'std::exception'\n");
        let eval = ev.evaluate_task(&spec, &view);
        assert!(eval.violated.contains(&Property::Exceptions));
    }

    #[test]
    fn execution_time_violates_if_over_bound() {
        let ev = Evaluator::new();
        let spec = spec_cpp_with_time(100);
        let view = ev.parse(1, "RUNTIME_MS: 150\nEXIT_CODE: 0\n");
        let eval = ev.evaluate_task(&spec, &view);
        assert!(eval.violated.contains(&Property::ExecutionTime));
    }

    #[test]
    fn execution_time_not_checked_if_no_bound() {
        let ev = Evaluator::new();
        let spec = spec_cpp(); // max_runtime_ms: None
        let view = ev.parse(1, "RUNTIME_MS=1000\nEXIT_CODE=0\n");
        let eval = ev.evaluate_task(&spec, &view);
        assert!(!eval.violated.contains(&Property::ExecutionTime));
    }

    #[test]
    fn illegal_output_detected_on_exact_line_match_after_trim() {
        let ev = Evaluator::new();
        let spec = spec_cpp_with_forbidden(&["BAD", "forbidden"]);
        let view = ev.parse(1, "EXIT_CODE:0\n output \n BAD \n ok \n");
        let eval = ev.evaluate_task(&spec, &view);
        assert!(eval.violated.contains(&Property::IllegalOutput));
    }

    #[test]
    fn illegal_output_not_checked_if_forbidden_empty() {
        let ev = Evaluator::new();
        let spec = spec_cpp(); // no forbidden_outputs
        let view = ev.parse(1, "EXIT_CODE:0\nforbidden\n");
        let eval = ev.evaluate_task(&spec, &view);
        assert!(!eval.violated.contains(&Property::IllegalOutput));
    }

    // -------------------- derive_props (ltl_milli, fail_milli) --------------------

    #[test]
    fn derive_props_all_clean_tasks_yield_zero_milli() {
        let ev = Evaluator::new();
        let specs = vec![spec_cpp(), spec_cpp()];
        let outs = vec![
            (10, "EXIT_CODE: 0\nSTDERR:\n".to_string()),
            (11, "EXIT_CODE=0\n".to_string()),
        ];
        let (ltl_milli, fail_milli) = ev.derive_props(&specs, &outs);
        assert_eq!(ltl_milli, 0);
        assert_eq!(fail_milli, 0);
    }

    #[test]
    fn derive_props_counts_violations_and_failures() {
        // One task segfaults (violates Safety? maybe; definitely Segfault),
        // nonzero exit (ProperTermination), so expect both ltl and failure > 0.
        let ev = Evaluator::new();
        let specs = vec![spec_cpp()];
        let outs = vec![(
            99,
            "EXIT_CODE=139\nSTDERR: Segmentation fault\n".to_string()
        )];
        let (ltl_milli, fail_milli) = ev.derive_props(&specs, &outs);
        assert!(ltl_milli > 0);
        assert_eq!(fail_milli, 1000); // 1/1 tasks failed → 1000
    }

    #[test]
    fn derive_props_execution_time_included_only_if_bound_and_measured() {
        let ev = Evaluator::new();
        let specs = vec![spec_cpp_with_time(50)];
        // RUNTIME_MS present and exceeds bound -> counts as a check + violation.
        let outs = vec![(
            1,
            "EXIT_CODE: 0\nRUNTIME_MS: 100\n".to_string(),
        )];
        let (ltl_milli, fail_milli) = ev.derive_props(&specs, &outs);
        assert!(ltl_milli > 0);
        // Task not “failed” by our definition (return code OK, no segfault/exception/forbidden)
        assert_eq!(fail_milli, 0);
    }


    #[test]
    fn contains_forbidden_output_is_case_insensitive_substring() {
        let ev = Evaluator::new();
        assert!(ev.contains_forbidden_output("Hello FORB\n", &[String::from("forb")]));
        assert!(!ev.contains_forbidden_output("clean\n", &[String::from("bad")]));
    }

    
    fn spec_java() -> TaskSpec {
        TaskSpec {
            language: Language::Java,
            valid_return_codes: Some(vec![0]),
            max_runtime_ms: None,
            forbidden_outputs: vec![],
        }
    }

    #[test]
    fn java_exception_detected() {
        let ev = Evaluator::new();
        let spec = spec_java();
        let view = ev.parse(7,
            "EXIT_CODE: 1\nSTDERR: Exception in thread \"main\" java.lang.NullPointerException\n\tat Main.main(Main.java:3)\n"
        );
        let eval = ev.evaluate_task(&spec, &view);
        assert!(eval.violated.contains(&Property::Exceptions));
        assert!(eval.violated.contains(&Property::ProperTermination)); // exit!=0
        assert!(!eval.violated.contains(&Property::SegmentationFault)); // normal Java exception
    }

    #[test]
    fn java_vm_crash_counts_as_safety_and_segfault() {
        let ev = Evaluator::new();
        let spec = spec_java();
        let view = ev.parse(8,
            "EXIT_CODE=134\nSTDERR: A fatal error has been detected by the Java Runtime Environment:\nSIGSEGV (0xb) at pc 0x00007f..., pid=123, tid=456\n#  Problematic frame:\n#  C  [libsomething.so+0x1a2b]\n#  An hs_err_pid123.log file is generated\n"
        );
        let eval = ev.evaluate_task(&spec, &view);
        assert!(eval.violated.contains(&Property::Safety));            // VM fatal
        assert!(eval.violated.contains(&Property::SegmentationFault)); // SIGSEGV marker
        assert!(eval.violated.contains(&Property::ProperTermination)); // exit!=0
    }

    #[test]
    fn java_clean_run_ok() {
        let ev = Evaluator::new();
        let spec = spec_java();
        let view = ev.parse(9, "EXIT_CODE=0\nSTDERR:\n");
        let eval = ev.evaluate_task(&spec, &view);
        assert!(eval.violated.is_empty());
    }

    #[test]
    fn java_derive_props_milli_values() {
        let ev = Evaluator::new();
        let specs = vec![spec_java(), spec_java()];
        let outs = vec![
            // clean
            (1, "EXIT_CODE=0\n".to_string()),
            // Java exception ⇒ Exceptions + ProperTermination, task fails
            (2, "EXIT_CODE: 1\nSTDERR: Exception in thread \"main\" java.lang.RuntimeException: boom\n".to_string()),
        ];
        let (ltl_milli, fail_milli) = ev.derive_props(&specs, &outs);
        assert!(ltl_milli > 0, "should record some LTL violations");
        assert_eq!(fail_milli, 500, "1/2 tasks failed ⇒ 500");
    }

}

