// ai/src/utils/evaluator.rs

use util::execution_config::ExecutionConfig;
use crate::HashMap;

// IF YOU WANT TO ADD SUPPORT FOR OTHER LANGUAGES, ADD THEM HERE
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
    ExpectedExact,     // ExpectedExact,  
    ExpectedContains,  // ExpectedContains,/
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
use util::execution_config::execution_config::Language as ExecLanguage;

impl TaskSpec {
    pub fn from_execution_config(config: &ExecutionConfig) -> Self {
        Self {
            language: match config.project.language {
                ExecLanguage::Cpp => Language::Cpp,
                ExecLanguage::Java => Language::Java,
            },
            valid_return_codes: Some(config.gatlam.task_spec.valid_return_codes.clone()),
            max_runtime_ms: config.gatlam.task_spec.max_runtime_ms,
            forbidden_outputs: config.gatlam.task_spec.forbidden_outputs.clone(),
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
    pub fn new() -> Self {
        Self
    }

    /// Parse raw blob into a structured view.
    /// Expected markers (case-insensitive):
    ///   EXIT_CODE: <n>  or EXIT_CODE=<n>
    ///   RUNTIME_MS: <n> or RUNTIME_MS=<n>
    ///   STDERR:         (starts stderr section to EOF)
    pub fn parse(&self, task_id: i64, blob: &str) -> TaskView {
        let (exit_code, stdout, stderr) = split_exit_stdout_stderr(blob);
        let runtime_ms = extract_marker_int(blob, "runtime_ms").map(|v| v.max(0) as u64);
        let terminated = exit_code.is_some();
        TaskView {
            task_id,
            exit_code,
            runtime_ms,
            stdout,
            stderr,
            terminated,
        }
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
            let forb = spec
                .forbidden_outputs
                .iter()
                .map(|s| s.trim().to_string())
                .collect::<Vec<_>>();
            if outs.iter().any(|o| forb.iter().any(|x| x == o)) {
                violated.push(Property::IllegalOutput);
            }
        }

        TaskEvaluation {
            task_id: view.task_id,
            violated,
        }
    }

    /// Helper for GA.
    /// Returns milli-fractions in 0..=1000:
    ///   (ltl_milli, fail_milli)
    /// - ltl_milli: fraction of violated LTL-ish properties across all tasks checked
    /// - fail_milli: fraction of tasks considered failed
pub fn derive_props(
    &self,
    specs: &[TaskSpec],
    outs: &[(i64, String)],
    memo: &[(i64, String)],
) -> (usize, usize) {
    let total_tasks = outs.len().max(1);

    let mut ltl_checks     = 0usize;
    let mut ltl_violations = 0usize;
    let mut failed_tasks   = 0usize;

    let memo_map: HashMap<i64, &str> = memo.iter().map(|(tid, s)| (*tid, s.as_str())).collect();

    for (i, (task_id, blob)) in outs.iter().enumerate() {
        let spec = specs.get(i).unwrap_or_else(|| &specs[0]);
        let view = self.parse(*task_id, blob);
        let eval = self.evaluate_task(spec, &view);

        let mut checks = 0usize;
        let mut viols  = 0usize;

        checks += 1; if eval.violated.contains(&Property::Safety)            { viols += 1; }
        checks += 1; if eval.violated.contains(&Property::ProperTermination) { viols += 1; }
        checks += 1; if eval.violated.contains(&Property::SegmentationFault) { viols += 1; }
        checks += 1; if eval.violated.contains(&Property::Exceptions)        { viols += 1; }

        if spec.max_runtime_ms.is_some() && view.terminated && view.runtime_ms.is_some() {
            checks += 1;
            if eval.violated.contains(&Property::ExecutionTime) { viols += 1; }
        }

        if let Some(memo_text) = memo_map.get(task_id) {
            let out_lines  = normalized_lines(&view.stdout);
            let memo_lines = normalized_lines(memo_text);

            checks += 1;
            let contains_ok = memo_lines.iter().all(|needle|
                out_lines.iter().any(|hay| hay.contains(needle))
            );
            if !contains_ok {
                viols += 1;
            }
        }

        if !spec.forbidden_outputs.is_empty() && view.terminated {
            checks += 1;
            if eval.violated.contains(&Property::IllegalOutput) { viols += 1; }
        }

        ltl_checks     += checks;
        ltl_violations += viols;

        let ret_ok = is_valid_return_code(view.exit_code, spec.valid_return_codes.as_deref());
        let failed = !ret_ok
            || (view.terminated && has_segfault(spec.language, &view.stderr))
            || (view.terminated && has_exception(spec.language, &view.stderr))
            || self.contains_forbidden_output(&view.stdout, &spec.forbidden_outputs);

        if failed { failed_tasks += 1; }
    }

    let ltl_milli  = if ltl_checks == 0 { 0 } else { ((ltl_violations * 1000) / ltl_checks).min(1000) };
    let fail_milli = ((failed_tasks   * 1000) / total_tasks).min(1000);

    (ltl_milli, fail_milli)
}

    fn contains_forbidden_output(&self, stdout: &str, forbidden: &[String]) -> bool {
        if forbidden.is_empty() {
            return false;
        }
        let hay = stdout.to_ascii_lowercase();
        forbidden
            .iter()
            .any(|needle| hay.contains(&needle.to_ascii_lowercase()))
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
            "error:",
            "exception",
            "segmentation fault",
            "sigsegv",
            "addresssanitizer",
            "asan",
            "double free",
            "invalid pointer",
            "use-after-free",
            "heap-use-after-free",
            "free(): invalid pointer",
            "munmap_chunk(): invalid pointer",
        ]
        .iter()
        .any(|needle| lower.contains(needle));
        if looks_error {
            stderr.push_str(blob);
        } else {
            stdout.push_str(blob);
        }
    }

    (
        exit_code,
        stdout.trim().to_string(),
        stderr.trim().to_string(),
    )
}

fn extract_marker_int(blob: &str, key: &str) -> Option<i32> {
    let key_lower = key.to_ascii_lowercase();
    for line in blob.lines() {
        let l = line.trim();
        let ll = l.to_ascii_lowercase();
        if let Some(idx) = ll.find(&key_lower) {
            let after = &ll[idx + key_lower.len()..];
            // strip separators
            let after =
                after.trim_start_matches(|c: char| c == ':' || c == '=' || c.is_whitespace());
            // take signed int prefix
            let mut end = 0;
            for ch in after.chars() {
                if ch.is_ascii_digit() || ch == '-' || ch == '+' {
                    end += ch.len_utf8();
                } else {
                    break;
                }
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
    if n.is_empty() {
        return Some(0);
    }
    for i in 0..=h.len().saturating_sub(n.len()) {
        if h[i..i + n.len()]
            .iter()
            .zip(n.iter())
            .all(|(a, b)| a.to_ascii_lowercase() == b.to_ascii_lowercase())
        {
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

// IF YOU WANT TO ADD SUPPORT FOR OTHER LANGUAGES, ADD THEM HERE
fn violates_safety(lang: Language, stderr: &str) -> bool {
    let s = stderr.to_ascii_lowercase();

    match lang {
        Language::Cpp => {
            s.contains("double free")
                || s.contains("double-free")
                || s.contains("invalid pointer")
                || s.contains("use-after-free")
                || s.contains("heap-use-after-free")
                || s.contains("segmentation fault")
                || s.contains("sigsegv")
                || s.contains("addresssanitizer")
                || s.contains("asan:")
        }
        Language::Java => {
            s.contains("hs_err_pid")                      // JVM fatal log header
                || s.contains("a fatal error has been detected by the java runtime environment")
                || s.contains("sigsegv")                  // native segv bubbled up by JVM
                || s.contains("exception_access_violation")
                || s.contains("problematic frame:")
                || s.contains("outofmemoryerror: direct buffer memory") // catastrophic OOM kind
                || s.contains("internal error (") // hotspot internal error
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
        (Some(0), None) => true,
        (Some(_), None) => false,
        (None, _) => false,
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    // ---------- helpers ----------
    fn spec_cpp() -> TaskSpec {
        TaskSpec {
            language: Language::Cpp,
            valid_return_codes: Some(vec![0]),
            max_runtime_ms: None,
            forbidden_outputs: vec![],
        }
    }

    fn spec_cpp_time(bound: u64) -> TaskSpec {
        TaskSpec { max_runtime_ms: Some(bound), ..spec_cpp() }
    }

    fn spec_cpp_forbidden(xs: &[&str]) -> TaskSpec {
        TaskSpec {
            forbidden_outputs: xs.iter().map(|s| s.to_string()).collect(),
            ..spec_cpp()
        }
    }

    fn spec_java() -> TaskSpec {
        TaskSpec {
            language: Language::Java,
            valid_return_codes: Some(vec![0]),
            max_runtime_ms: None,
            forbidden_outputs: vec![],
        }
    }

    // Convenience: make (task_id, blob) tuple
    fn out(task_id: i64, blob: &str) -> (i64, String) {
        (task_id, blob.to_string())
    }

    // ---------- low-level parsing ----------
    #[test]
    fn extract_marker_int_supports_colon_and_equals() {
        let blob = "EXIT_CODE: 0\nRUNTIME_MS=123\nfoo\n";
        assert_eq!(super::extract_marker_int(blob, "exit_code"), Some(0));
        assert_eq!(super::extract_marker_int(blob, "RUNTIME_MS"), Some(123));
    }

    #[test]
    fn split_exit_stdout_stderr_explicit_stderr() {
        let blob = "hello\nline\nSTDERR: bad stuff\nmore\n";
        let (exit, out, err) = super::split_exit_stdout_stderr(blob);
        assert_eq!(exit, None);
        assert_eq!(out, "hello\nline");
        assert!(err.starts_with("bad stuff"));
        assert!(err.contains("more"));
    }

    #[test]
    fn split_exit_heuristic_routes_errors_to_stderr() {
        let blob = "Segmentation fault (core dumped)";
        let (exit, out, err) = super::split_exit_stdout_stderr(blob);
        assert_eq!(exit, None);
        assert!(out.is_empty());
        assert!(err.to_ascii_lowercase().contains("segmentation fault"));
    }

    #[test]
    fn parse_sets_terminated_if_exit_code_present() {
        let ev = Evaluator::new();
        let v = ev.parse(7, "EXIT_CODE: 0\nHello\n");
        assert_eq!(v.task_id, 7);
        assert_eq!(v.exit_code, Some(0));
        assert!(v.terminated);
        assert!(v.stdout.contains("EXIT_CODE"));
    }

    #[test]
    fn parse_parses_runtime_ms() {
        let ev = Evaluator::new();
        let v = ev.parse(1, "RUNTIME_MS=250\nEXIT_CODE=0\n");
        assert_eq!(v.runtime_ms, Some(250));
        assert!(v.terminated);
    }

    // ---------- C++ properties ----------
    #[test]
    fn cpp_proper_termination_ok_zero_exit() {
        let ev = Evaluator::new();
        let spec = spec_cpp();
        let view = ev.parse(1, "EXIT_CODE: 0\n");
        let eval = ev.evaluate_task(&spec, &view);
        assert!(!eval.violated.contains(&Property::ProperTermination));
    }

    #[test]
    fn cpp_proper_termination_violates_nonzero_exit() {
        let ev = Evaluator::new();
        let spec = spec_cpp();
        let view = ev.parse(2, "EXIT_CODE=2\n");
        let eval = ev.evaluate_task(&spec, &view);
        assert!(eval.violated.contains(&Property::ProperTermination));
    }

    #[test]
    fn cpp_safety_detects_asan_and_uaf() {
        let ev = Evaluator::new();
        let spec = spec_cpp();
        let view = ev.parse(3, "STDERR: AddressSanitizer: heap-use-after-free");
        let eval = ev.evaluate_task(&spec, &view);
        assert!(eval.violated.contains(&Property::Safety));
    }

    #[test]
    fn cpp_segmentation_fault_detected() {
        let ev = Evaluator::new();
        let spec = spec_cpp();
        let view = ev.parse(4, "STDERR: Segmentation fault");
        let eval = ev.evaluate_task(&spec, &view);
        assert!(eval.violated.contains(&Property::SegmentationFault));
    }

    #[test]
    fn cpp_exception_detected() {
        let ev = Evaluator::new();
        let spec = spec_cpp();
        let view = ev.parse(5, "STDERR: terminate called after throwing an instance of 'std::exception'");
        let eval = ev.evaluate_task(&spec, &view);
        assert!(eval.violated.contains(&Property::Exceptions));
    }

    // ---------- Java properties ----------
    #[test]
    fn java_exception_detected() {
        let ev = Evaluator::new();
        let spec = spec_java();
        let view = ev.parse(10, "STDERR: Exception in thread \"main\" java.lang.NullPointerException");
        let eval = ev.evaluate_task(&spec, &view);
        assert!(eval.violated.contains(&Property::Exceptions));
    }

    #[test]
    fn java_segfault_patterns_detected() {
        let ev = Evaluator::new();
        let spec = spec_java();
        let view = ev.parse(11, "STDERR:  #  A fatal error has been detected by the Java Runtime Environment\nSIGSEGV");
        let eval = ev.evaluate_task(&spec, &view);
        assert!(eval.violated.contains(&Property::SegmentationFault));
        assert!(ev.evaluate_task(&spec, &view).violated.contains(&Property::Safety));
    }

    // ---------- timing & forbidden ----------
    #[test]
    fn execution_time_violates_if_over_bound() {
        let ev = Evaluator::new();
        let spec = spec_cpp_time(100);
        let view = ev.parse(20, "EXIT_CODE: 0\nRUNTIME_MS: 150\n");
        let eval = ev.evaluate_task(&spec, &view);
        assert!(eval.violated.contains(&Property::ExecutionTime));
    }

    #[test]
    fn execution_time_not_checked_when_no_bound() {
        let ev = Evaluator::new();
        let spec = spec_cpp();
        let view = ev.parse(21, "EXIT_CODE=0\nRUNTIME_MS=999\n");
        let eval = ev.evaluate_task(&spec, &view);
        assert!(!eval.violated.contains(&Property::ExecutionTime));
    }

    #[test]
    fn illegal_output_detected_exact_line_match() {
        let ev = Evaluator::new();
        let spec = spec_cpp_forbidden(&["BAD", "forbidden"]);
        let view = ev.parse(22, "EXIT_CODE: 0\nok\n BAD \n");
        let eval = ev.evaluate_task(&spec, &view);
        assert!(eval.violated.contains(&Property::IllegalOutput));
    }

    #[test]
    fn contains_forbidden_output_substring_ci() {
        let ev = Evaluator::new();
        assert!(ev.contains_forbidden_output("Hello FORB\n", &[String::from("forb")]));
        assert!(!ev.contains_forbidden_output("clean\n", &[String::from("bad")]));
    }

    // ---------- derive_props (ltl_milli, fail_milli) ----------
    #[test]
    fn derive_props_all_clean_zero_zero() {
        let ev = Evaluator::new();
        let specs = vec![spec_cpp(), spec_cpp()];
        let outs = vec![
            out(100, "EXIT_CODE: 0\nSTDERR:\n"),
            out(101, "EXIT_CODE=0\n"),
        ];
        let memo: Vec<(i64, String)> = vec![]; // no memo checks
        let (ltl_milli, fail_milli) = ev.derive_props(&specs, &outs, &memo);
        assert_eq!(ltl_milli, 0);
        assert_eq!(fail_milli, 0);
    }

    #[test]
    fn derive_props_one_task_failed_yields_half_fail_fraction() {
        let ev = Evaluator::new();
        let specs = vec![spec_cpp(), spec_cpp()];
        // Task 1 OK; Task 2 segfaults + nonzero exit
        let outs = vec![
            out(200, "EXIT_CODE=0\n"),
            out(201, "EXIT_CODE=139\nSTDERR: Segmentation fault\n"),
        ];
        let memo: Vec<(i64, String)> = vec![];
        let (ltl_milli, fail_milli) = ev.derive_props(&specs, &outs, &memo);
        assert!(ltl_milli > 0);
        assert_eq!(fail_milli, 500); // 1 of 2 tasks failed
    }

    // ---------- memo-based: exact + contains ----------
    #[test]
    fn memo_exact_match_does_not_add_violations() {
        let ev = Evaluator::new();
        let specs = vec![spec_cpp()];
        let outs = vec![
            out(300, "EXIT_CODE=0\nhello\nworld\n")
        ];
        let memo = vec![
            out(300, "hello\nworld\n") // exact same lines (after trim)
        ];
        let (ltl_milli, fail_milli) = ev.derive_props(&specs, &outs, &memo);
        // No extra memo violations expected
        assert_eq!(ltl_milli, 0);
        assert_eq!(fail_milli, 0);
    }

    #[test]
    fn memo_exact_mismatch_increases_ltl() {
        let ev = Evaluator::new();
        let specs = vec![spec_cpp()];
        let outs = vec![
            out(301, "EXIT_CODE=0\nhello\nplanet\n") // planet vs world
        ];
        let memo = vec![
            out(301, "hello\nworld\n")
        ];
        let (ltl_milli, _fail_milli) = ev.derive_props(&specs, &outs, &memo);
        assert!(ltl_milli > 0);
    }

    #[test]
    fn memo_contains_must_find_all_memo_lines_somewhere() {
        let ev = Evaluator::new();
        let specs = vec![spec_cpp()];
        // Output has only "hello"
        let outs = vec![ out(302, "EXIT_CODE=0\n  hello   \n") ];
        // Memo requires "hello" and "world" (world missing)
        let memo = vec![ out(302, "hello\nworld\n") ];
        let (ltl_milli, _fail_milli) = ev.derive_props(&specs, &outs, &memo);
        assert!(ltl_milli > 0);
    }

    #[test]
    fn memo_contains_passes_if_every_memo_line_is_substring_of_some_output_line() {
        let ev = Evaluator::new();
        let specs = vec![spec_cpp()];
        let outs = vec![
            out(303, "EXIT_CODE=0\nThe quick brown fox\njumps over the lazy dog\n")
        ];
        let memo = vec![
            out(303, "quick brown\nlazy dog\n")
        ];
        let (ltl_milli, _fail_milli) = ev.derive_props(&specs, &outs, &memo);
        assert_eq!(ltl_milli, 0);
    }

    // ---------- Java in derive_props ----------
    #[test]
    fn derive_props_with_java_exception_counts_violation_but_not_failure_if_exit_ok() {
        let ev = Evaluator::new();
        let specs = vec![spec_java()];
        // stdout empty, stderr has exception, but assume EXIT_CODE=0 (some runners swallow it)
        let outs = vec![
            out(400, "EXIT_CODE=0\nSTDERR: Exception in thread \"main\" java.lang.RuntimeException")
        ];
        let memo: Vec<(i64, String)> = vec![];
        let (ltl_milli, fail_milli) = ev.derive_props(&specs, &outs, &memo);
        assert!(ltl_milli > 0);
        // By our current "failed task" definition: failed if !ret_ok OR segfault OR exception OR forbidden
        // We do include exception in failed predicate, so fail_milli should be 1000.
        assert_eq!(fail_milli, 1000);
    }
}