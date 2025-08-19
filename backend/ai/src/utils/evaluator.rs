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
    ExpectedContains,  // ExpectedContains,
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
    delimiter: &str,
) -> (usize, usize) {
    let total_tasks = outs.len().max(1);

    let mut ltl_checks     = 0usize;
    let mut ltl_violations = 0usize;
    let mut failed_tasks   = 0usize;

    // Build a single label→expected-lines map from ALL memo entries (ignores task_id).
    let mut memo_sections: HashMap<String, Vec<String>> = HashMap::new();
    for (_tid, memotxt) in memo {
        let secs = parse_labeled_sections_with_delim(memotxt, delimiter);
        for (k, v) in secs {
            memo_sections.insert(k, v);
        }
    }

    for (i, (task_id, blob)) in outs.iter().enumerate() {
        eprintln!("--- Evaluating blob ---");
        eprintln!("task_id={task_id}, index={i}");
        eprintln!("{blob}");
        eprintln!("------------------------");

        let spec = specs.get(i).unwrap_or_else(|| &specs[0]);
        let view = self.parse(*task_id, blob);
        let eval = self.evaluate_task(spec, &view);

        let mut checks = 0usize;
        let mut viols  = 0usize;

        // Core LTL-ish checks
        checks += 1; if eval.violated.contains(&Property::Safety)            { viols += 1; }
        checks += 1; if eval.violated.contains(&Property::ProperTermination) { viols += 1; }
        checks += 1; if eval.violated.contains(&Property::SegmentationFault) { viols += 1; }
        checks += 1; if eval.violated.contains(&Property::Exceptions)        { viols += 1; }

        if spec.max_runtime_ms.is_some() && view.terminated && view.runtime_ms.is_some() {
            checks += 1;
            if eval.violated.contains(&Property::ExecutionTime) { viols += 1; }
        }

        // ---------- Labeled memo comparison (by subtask label via delimiter) ----------
        let out_sections = parse_labeled_sections_with_delim(&view.stdout, delimiter);

        for (label, memo_lines) in &memo_sections {
            // Exact match within this label
            checks += 1;
            match out_sections.get(label) {
                Some(out_lines) => {
                    if out_lines != memo_lines {
                        viols += 1;
                        // If you want to tag the property, push Property::ExpectedExact into a separate vector you track here
                    }
                }
                None => {
                    // Section missing -> violation
                    viols += 1;
                }
            }

            // "Contains" check within the same label
            checks += 1;
            match out_sections.get(label) {
                Some(out_lines) => {
                    let contains_ok = memo_lines.iter().all(|needle|
                        out_lines.iter().any(|hay| hay.contains(needle))
                    );
                    if !contains_ok {
                        viols += 1;
                        // Likewise, this corresponds to ExpectedContains
                    }
                }
                None => {
                    viols += 1;
                }
            }
        }
        // ------------------------------------------------------------------------------

        if !spec.forbidden_outputs.is_empty() && view.terminated {
            checks += 1;
            if eval.violated.contains(&Property::IllegalOutput) { viols += 1; }
        }

        ltl_checks     += checks;
        ltl_violations += viols;

        // Failure metric (separate from LTL)
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

fn parse_labeled_sections_with_delim(s: &str, delim: &str) -> std::collections::HashMap<String, Vec<String>> {
    let mut map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let mut current: Option<String> = None;

    for raw in s.lines() {
        let line = raw.trim();
        if line.starts_with(delim) {
            let label = line[delim.len()..].trim().to_string();
            current = Some(label);
            continue;
        }
        if line.is_empty() { continue; }
        if let Some(lbl) = &current {
            map.entry(lbl.clone()).or_default().push(line.to_string());
        }
    }
    map
}


fn split_exit_stdout_stderr(blob: &str) -> (Option<i32>, String, String) {
    let mut exit_code: Option<i32> = None;
    let mut stderr = String::new();
    let mut stdout = String::new();

    // EXIT_CODE
    if let Some(v) = extract_marker_int(blob, "Retcode") {
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

    // ---------------- helpers ----------------

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

    // Simple out/memo tuple makers
    fn out(task_id: i64, blob: &str) -> (i64, String) { (task_id, blob.to_string()) }
    fn memo(task_id: i64, text: &str) -> (i64, String) { (task_id, text.to_string()) }

    // Build a stdout with labeled sections + optional Retcode
    fn build_labeled_stdout(
        delim: &str,
        sections: &[(&str, &[&str])],
        retcode: i32,
        with_runtime_ms: Option<u64>,
    ) -> String {
        let mut s = String::new();
        for (label, lines) in sections {
            s.push_str(delim);
            s.push_str(label);
            s.push('\n');
            for &ln in *lines {
                s.push_str(ln);
                s.push('\n');
            }
        }
        if let Some(ms) = with_runtime_ms {
            s.push_str(&format!("\nRUNTIME_MS: {ms}\n"));
        } else {
            s.push('\n');
        }
        s.push_str(&format!("Retcode: {retcode}\n"));
        s
    }

    // ---------------- parse/utility tests ----------------

    #[test]
    fn extract_marker_int_retcode_and_runtime() {
        let blob = "hello\nRUNTIME_MS=250\nRetcode: 0\nbye";
        assert_eq!(super::extract_marker_int(blob, "runtime_ms"), Some(250));
        assert_eq!(super::extract_marker_int(blob, "Retcode"), Some(0));
    }

    #[test]
    fn find_case_insensitive_basic() {
        assert_eq!(super::find_case_insensitive("AbCde", "bcd"), Some(1));
        assert_eq!(super::find_case_insensitive("xyz", "AB"), None);
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
    fn split_exit_parses_retcode() {
        let blob = "some output\n\nRetcode: 0\n";
        let (exit, out, err) = super::split_exit_stdout_stderr(blob);
        assert_eq!(exit, Some(0));
        assert!(err.is_empty());
        assert!(out.contains("some output"));
    }

    #[test]
    fn normalized_lines_trims_and_drops_empty() {
        let v = super::normalized_lines(" a \n\nb\n  \n c ");
        assert_eq!(v, vec!["a", "b", "c"]);
    }

    #[test]
    fn parse_labeled_sections_with_delim_basic() {
        let delim = "&-=-&";
        let txt = "&-=-&task1\n12\n34\n\n&-=-&task2\nx\ny\n";
        let map = super::parse_labeled_sections_with_delim(txt, delim);
        assert_eq!(map.get("task1").unwrap(), &vec!["12".to_string(), "34".to_string()]);
        assert_eq!(map.get("task2").unwrap(), &vec!["x".to_string(), "y".to_string()]);
    }

    // ---------------- language/safety/exception tests ----------------

    #[test]
    fn cpp_safety_detects_asan_and_uaf() {
        let ev = Evaluator::new();
        let spec = spec_cpp();
        let blob = "STDERR: AddressSanitizer: heap-use-after-free\nRetcode: 1\n";
        let view = ev.parse(1, blob);
        let eval = ev.evaluate_task(&spec, &view);
        assert!(eval.violated.contains(&Property::Safety));
    }

    #[test]
    fn cpp_segmentation_fault_detected() {
        let ev = Evaluator::new();
        let spec = spec_cpp();
        let blob = "STDERR: Segmentation fault\nRetcode: 139\n";
        let view = ev.parse(1, blob);
        let eval = ev.evaluate_task(&spec, &view);
        assert!(eval.violated.contains(&Property::SegmentationFault));
    }

    #[test]
    fn cpp_exception_detected() {
        let ev = Evaluator::new();
        let spec = spec_cpp();
        let blob = "STDERR: terminate called after throwing an instance of 'std::exception'\nRetcode: 1\n";
        let view = ev.parse(1, blob);
        let eval = ev.evaluate_task(&spec, &view);
        assert!(eval.violated.contains(&Property::Exceptions));
    }

    #[test]
    fn java_exception_detected() {
        let ev = Evaluator::new();
        let spec = spec_java();
        let blob = "STDERR: Exception in thread \"main\" java.lang.NullPointerException\nRetcode: 1\n";
        let view = ev.parse(1, blob);
        let eval = ev.evaluate_task(&spec, &view);
        assert!(eval.violated.contains(&Property::Exceptions));
    }

    #[test]
    fn java_segfault_patterns_detected() {
        let ev = Evaluator::new();
        let spec = spec_java();
        let blob = "STDERR: A fatal error has been detected by the Java Runtime Environment\nSIGSEGV (0xb)\nRetcode: 134\n";
        let view = ev.parse(1, blob);
        let eval = ev.evaluate_task(&spec, &view);
        assert!(eval.violated.contains(&Property::SegmentationFault) || eval.violated.contains(&Property::Safety));
    }

    // ---------------- termination/forbidden/runtime tests ----------------

    #[test]
    fn proper_termination_ok_when_zero_exit() {
        let ev = Evaluator::new();
        let spec = spec_cpp();
        let view = ev.parse(1, "Retcode: 0\n");
        let eval = ev.evaluate_task(&spec, &view);
        assert!(!eval.violated.contains(&Property::ProperTermination));
    }

    #[test]
    fn proper_termination_violates_on_nonzero_exit() {
        let ev = Evaluator::new();
        let spec = spec_cpp();
        let view = ev.parse(1, "Retcode: 2\n");
        let eval = ev.evaluate_task(&spec, &view);
        assert!(eval.violated.contains(&Property::ProperTermination));
    }

    #[test]
    fn execution_time_violates_if_over_bound() {
        let ev = Evaluator::new();
        let spec = spec_cpp_time(100);
        let view = ev.parse(1, "RUNTIME_MS: 150\nRetcode: 0\n");
        let eval = ev.evaluate_task(&spec, &view);
        assert!(eval.violated.contains(&Property::ExecutionTime));
    }

    #[test]
    fn execution_time_not_checked_if_no_bound() {
        let ev = Evaluator::new();
        let spec = spec_cpp(); // no bound
        let view = ev.parse(1, "RUNTIME_MS=1000\nRetcode: 0\n");
        let eval = ev.evaluate_task(&spec, &view);
        assert!(!eval.violated.contains(&Property::ExecutionTime));
    }

    #[test]
    fn forbidden_output_detected_on_exact_line_match() {
        let ev = Evaluator::new();
        let spec = spec_cpp_forbidden(&["BAD", "forbidden"]);
        let blob = "&-=-&X\nok\nBAD\n\nRetcode: 0\n";
        let view = ev.parse(1, blob);
        let eval = ev.evaluate_task(&spec, &view);
        assert!(eval.violated.contains(&Property::IllegalOutput));
    }

    #[test]
    fn contains_forbidden_output_substring_ci() {
        let ev = Evaluator::new();
        assert!(ev.contains_forbidden_output("Hello FORB\n", &[String::from("forb")]));
        assert!(!ev.contains_forbidden_output("clean\n", &[String::from("bad")]));
    }

    // ---------------- delimiter-based memo tests ----------------

    #[test]
    fn memo_exact_and_contains_both_pass_yield_zero_ltl() {
        let ev = Evaluator::new();
        let delim = "&-=-&";

        // Memo has two labels with exact lines
        let memo_txt = build_labeled_stdout(
            delim,
            &[
                ("task1Subtask1", &["24"]),
                ("task1Subtask2", &["24"]),
            ],
            0,
            None,
        );

        // Output matches memo exactly
        let out_txt = build_labeled_stdout(
            delim,
            &[
                ("task1Subtask1", &["24"]),
                ("task1Subtask2", &["24"]),
            ],
            0,
            None,
        );

        let specs = vec![spec_cpp()];
        let outs  = vec![out(48, &out_txt)];
        let memo  = vec![memo(1, &memo_txt)];

        let (ltl_milli, fail_milli) = ev.derive_props(&specs, &outs, &memo, delim);
        assert_eq!(fail_milli, 0);
        assert_eq!(ltl_milli, 0);
    }

    #[test]
    fn memo_exact_fails_but_contains_passes_yields_fractional_ltl() {
        let ev = Evaluator::new();
        let delim = "&-=-&";

        // Memo: one label with one line
        let memo_txt = build_labeled_stdout(delim, &[("L", &["abc"])], 0, None);

        // Output: same label, but line has prefix/suffix -> exact fails, contains ok
        let out_txt = build_labeled_stdout(delim, &[("L", &["--abc--"])], 0, None);

        let specs = vec![spec_cpp()];
        let outs  = vec![out(10, &out_txt)];
        let memo  = vec![memo(1, &memo_txt)];

        // checks: 4 core + 2 memo = 6; viols: 1 (exact) -> 1/6 = 166
        let (ltl_milli, fail_milli) = ev.derive_props(&specs, &outs, &memo, delim);
        assert_eq!(fail_milli, 0);
        assert_eq!(ltl_milli, 166);
    }

    #[test]
    fn memo_contains_fails_when_output_missing_line() {
        let ev = Evaluator::new();
        let delim = "&-=-&";

        // Memo expects two lines
        let memo_txt = build_labeled_stdout(delim, &[("L", &["a", "b"])], 0, None);
        // Output only has "a"
        let out_txt  = build_labeled_stdout(delim, &[("L", &["a"])], 0, None);

        let specs = vec![spec_cpp()];
        let outs  = vec![out(11, &out_txt)];
        let memo  = vec![memo(1, &memo_txt)];

        // exact fails (lines differ), contains fails (b missing): viols=2
        // checks: 4 core + 2 memo = 6; 2/6 -> 333
        let (ltl_milli, fail_milli) = ev.derive_props(&specs, &outs, &memo, delim);
        assert_eq!(fail_milli, 0);
        assert_eq!(ltl_milli, 333);
    }

    #[test]
    fn memo_missing_label_counts_as_two_violations_for_that_label() {
        let ev = Evaluator::new();
        let delim = "&-=-&";

        // Memo has label L with one line
        let memo_txt = build_labeled_stdout(delim, &[("L", &["xyz"])], 0, None);
        // Output has no labels at all
        let out_txt  = "Retcode: 0\n".to_string();

        let specs = vec![spec_cpp()];
        let outs  = vec![out(12, &out_txt)];
        let memo  = vec![memo(1, &memo_txt)];

        // For that one label: exact missing -> viol, contains missing -> viol => +2
        // checks: 4 core + 2 memo = 6; 2/6 -> 333
        let (ltl_milli, _) = ev.derive_props(&specs, &outs, &memo, delim);
        assert_eq!(ltl_milli, 333);
    }

    #[test]
    fn memo_multiple_labels_some_match_some_dont() {
        let ev = Evaluator::new();
        let delim = "&-=-&";

        // Memo has 2 labels
        let memo_txt = build_labeled_stdout(
            delim,
            &[
                ("A", &["1", "2"]),
                ("B", &["x"]),
            ],
            0,
            None,
        );

        // Output: A matches exactly; B has "xx" -> exact fails, contains ok
        let out_txt = build_labeled_stdout(
            delim,
            &[
                ("A", &["1", "2"]),
                ("B", &["xx"]),
            ],
            0,
            None,
        );

        let specs = vec![spec_cpp()];
        let outs  = vec![out(13, &out_txt)];
        let memo  = vec![memo(1, &memo_txt)];

        // For A: exact ok, contains ok (0)
        // For B: exact fail (1), contains ok (0)
        // Total checks: 4 core + 2*2 memo = 8; viols=1 -> floor(1000/8)=125
        let (ltl_milli, _) = ev.derive_props(&specs, &outs, &memo, delim);
        assert_eq!(ltl_milli, 125);
    }

    // #[test]
    // fn derive_props_failure_fraction_when_nonzero_retcode() {
    //     let ev = Evaluator::new();
    //     let delim = "&-=-&";

    //     let memo_txt = build_labeled_stdout(delim, &[("L", &["ok"])], 0, None);
    //     // Out has Retcode 1 -> failure; memo label present and matching to avoid LTL noise
    //     let out_txt  = build_labeled_stdout(delim, &[("L", &["ok"])], 1, None);

    //     let specs = vec![spec_cpp()];
    //     let outs  = vec![out(14, &out_txt)];
    //     let memo  = vec![memo(1, &memo_txt)];

    //     let (ltl_milli, fail_milli) = ev.derive_props(&specs, &outs, &memo, delim);
    //     assert_eq!(ltl_milli, 0);
    //     assert_eq!(fail_milli, 1000);
    // }

    // #[test]
    // fn derive_props_forbidden_output_violation_included() {
    //     let ev = Evaluator::new();
    //     let delim = "&-=-&";

    //     let memo_txt = build_labeled_stdout(delim, &[("L", &["ok"])], 0, None);
    //     let out_txt  = format!("{}L\nok\nforbidden\n\nRetcode: 0\n", delim);

    //     let spec = spec_cpp_forbidden(&["forbidden"]);
    //     let specs = vec![spec];
    //     let outs  = vec![out(16, &out_txt)];
    //     let memo  = vec![memo(1, &memo_txt)];

    //     // Checks: 4 core + 2 memo + 1 forbidden = 7; violations: 1 (forbidden) -> 142
    //     let (ltl_milli, fail_milli) = ev.derive_props(&specs, &outs, &memo, delim);
    //     assert_eq!(fail_milli, 0);
    //     assert_eq!(ltl_milli, 142);
    // } 

    #[test]
    fn memo_and_core_both_violate_accumulate() {
        let ev = Evaluator::new();
        let delim = "&-=-&";

        // Memo expects "good"
        let memo_txt = build_labeled_stdout(delim, &[("L", &["good"])], 0, None);

        // Output has different line "bad" -> memo exact & contains fail (2)
        // and Retcode: 2 -> failure & ProperTermination violation (but ProperTermination
        // only contributes to ltl if counted as violation among checks)
        let out_txt  = build_labeled_stdout(delim, &[("L", &["bad"])], 2, None);

        let specs = vec![spec_cpp()];
        let outs  = vec![out(17, &out_txt)];
        let memo  = vec![memo(1, &memo_txt)];

        // LTL checks per task:
        //   4 core (Safety/PT/Segfault/Exceptions) -> PT will violate (non-zero ret) => +1
        //   2 memo (exact+contains) -> both violate => +2
        // Total checks = 6; violations = 3 => 3/6 = 500
        let (ltl_milli, fail_milli) = ev.derive_props(&specs, &outs, &memo, delim);
        assert_eq!(ltl_milli, 500);
        assert_eq!(fail_milli, 1000);
    }

    // #[test]
    // fn multiple_tasks_aggregate_fail_fraction() {
    //     let ev = Evaluator::new();
    //     let delim = "&-=-&";

    //     let memo_txt = build_labeled_stdout(delim, &[("L", &["ok"])], 0, None);

    //     // Two tasks: one ok (Retcode 0), one failure (Retcode 1)
    //     let out_ok  = build_labeled_stdout(delim, &[("L", &["ok"])], 0, None);
    //     let out_bad = build_labeled_stdout(delim, &[("L", &["ok"])], 1, None);

    //     let specs = vec![spec_cpp(), spec_cpp()];
    //     let outs  = vec![out(20, &out_ok), out(21, &out_bad)];
    //     let memo  = vec![memo(1, &memo_txt)];

    //     let (ltl_milli, fail_milli) = ev.derive_props(&specs, &outs, &memo, delim);
    //     assert_eq!(ltl_milli, 0);           
    //     assert_eq!(fail_milli, 500);        
    // }
}

