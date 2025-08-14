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
    /// I treat “terminated” as: we have an exit code OR any output. May change later!
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
        let terminated = exit_code.is_some() || !(stdout.is_empty() && stderr.is_empty());
        println!("Parsed TaskView for task {}: exit_code={:?}, runtime_ms={:?}, stdout='{}', stderr='{}', terminated={}",
                 task_id, exit_code, runtime_ms, stdout, stderr, terminated);
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
        if view.terminated && !is_valid_return_code(view.exit_code, spec.valid_return_codes.as_deref()) {
            violated.push(Property::ProperTermination);
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

    /// Helper for GA
    /// Count total violations across tasks as `num_ltl_props`.
    pub fn derive_props(&self, specs: &[TaskSpec], outs: &[(i64, String)]) -> (usize, usize) {
        let num_tasks = outs.len();
        let mut total = 0usize;

        for ((task_id, blob), spec) in outs.iter().zip(specs.iter()) {
            let view = self.parse(*task_id, blob);
            let eval = self.evaluate_task(spec, &view);
            total += eval.violated.len();
        }

        (total.max(1), num_tasks.max(1))
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
        // Heuristic: if it looks like an error, call it stderr
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
    // matches KEY: <n> or KEY=<n> (case-insensitive)
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
            // fill later
            false
        }
    }
}


fn has_segfault(lang: Language, stderr: &str) -> bool {
    let s = stderr.to_ascii_lowercase();
    match lang {
        Language::Cpp => s.contains("segmentation fault") || s.contains("sigsegv"),
        Language::Java => false,
    }
}

fn has_exception(_lang: Language, stderr: &str) -> bool {
    // For C++,  I am treating "terminate called", "std::exception", etc. as exception-ish signals.
    let s = stderr.to_ascii_lowercase();
    s.contains("exception")
        || s.contains("terminate called")
        || s.contains("std::terminate")
        || s.contains("std::bad_alloc")
}


fn is_valid_return_code(exit: Option<i32>, valid: Option<&[i32]>) -> bool {
    match (exit, valid) {
        (Some(code), Some(list)) => list.contains(&code),
        (Some(0), None)          => true,
        (Some(_), None)          => false,
        (None, _)                => false, 
    }
}