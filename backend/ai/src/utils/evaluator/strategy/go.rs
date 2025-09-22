use super::EvaluationStrategy;

/// Heuristics for the Go toolchain/runtime.
/// - Panics are treated as "exceptions"
/// - OS signals and fatal runtime errors are "safety" or "segfault"
pub struct GoStrategy;

impl EvaluationStrategy for GoStrategy {
    fn name(&self) -> &'static str { "go" }

    fn violates_safety(&self, stderr: &str) -> bool {
        let s = stderr.to_ascii_lowercase();
        s.contains("fatal error: runtime: out of memory")
            || s.contains("fatal error: stack overflow")
            || s.contains("fatal error: concurrent map read and map write")
            || s.contains("fatal error: ")
            || s.contains("signal sigbus")
            || s.contains("signal sigsegv")
    }

    fn has_segfault(&self, stderr: &str) -> bool {
        let s = stderr.to_ascii_lowercase();
        s.contains("signal sigsegv")
            || s.contains("segmentation violation")
            || s.contains("segmentation fault")
    }

    fn has_exception(&self, stderr: &str) -> bool {
        let s = stderr.to_ascii_lowercase();
        s.contains("panic: ")
            || s.contains("panic: runtime error:")
            || s.contains("invalid memory address or nil pointer dereference")
            || s.contains("index out of range")
            || s.contains("slice bounds out of range")
    }
}

#[cfg(test)]
mod tests {
    use super::{GoStrategy, EvaluationStrategy};

    #[test]
    fn go_sees_panics_as_exceptions() {
        let g = GoStrategy;
        assert!(g.has_exception("panic: runtime error: index out of range [3] with length 3"));
        assert!(g.has_exception("panic: runtime error: invalid memory address or nil pointer dereference"));
    }

    #[test]
    fn go_detects_segfault_signals() {
        let g = GoStrategy;
        assert!(g.has_segfault("signal SIGSEGV: segmentation violation code=0x1 addr=0x0"));
    }

    #[test]
    fn go_flags_fatal_runtime_as_safety() {
        let g = GoStrategy;
        assert!(g.violates_safety("fatal error: runtime: out of memory"));
        assert!(g.violates_safety("fatal error: stack overflow"));
    }
}
