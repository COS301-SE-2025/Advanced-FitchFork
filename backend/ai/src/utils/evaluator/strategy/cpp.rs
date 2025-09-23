use super::EvaluationStrategy;

pub struct CppStrategy;

/// Heuristics for the C++ toolchain/runtime.
/// - Use-after-free, double-free, invalid pointer dereference, and AddressSanitizer
///   errors are treated as "safety" violations.
/// - Segmentation faults are "segfaults"
impl EvaluationStrategy for CppStrategy {
    fn name(&self) -> &'static str {
        "cpp"
    }

    fn violates_safety(&self, stderr: &str) -> bool {
        let s = stderr.to_ascii_lowercase();
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

    fn has_segfault(&self, stderr: &str) -> bool {
        let s = stderr.to_ascii_lowercase();
        s.contains("segmentation fault") || s.contains("sigsegv")
    }

    fn has_exception(&self, stderr: &str) -> bool {
        let s = stderr.to_ascii_lowercase();
        s.contains("exception")
            || s.contains("terminate called")
            || s.contains("std::terminate")
            || s.contains("std::bad_alloc")
    }
}
