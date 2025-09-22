use super::EvaluationStrategy;

/// Heuristics for Rust.
/// - Panics are "exceptions"
/// - Sanitizer / allocator / low-level crashes are "safety"
pub struct RustStrategy;

impl EvaluationStrategy for RustStrategy {
    fn name(&self) -> &'static str { "rust" }

    fn violates_safety(&self, stderr: &str) -> bool {
        let s = stderr.to_ascii_lowercase();
        s.contains("addresssanitizer")
            || s.contains("asan:")
            || s.contains("ubsan")
            || s.contains("heap-use-after-free")
            || s.contains("use-after-free")
            || s.contains("double free")
            || s.contains("invalid pointer")
            || s.contains("memory allocation of") && s.contains("failed")
    }

    fn has_segfault(&self, stderr: &str) -> bool {
        let s = stderr.to_ascii_lowercase();
        s.contains("segmentation fault") || s.contains("sigsegv")
    }

    fn has_exception(&self, stderr: &str) -> bool {
        let s = stderr.to_ascii_lowercase();
        s.contains("thread '")
            && s.contains(" panicked at '")
            || s.contains("panicked at '")
            || s.contains("panic:")
            || s.contains("called `option::unwrap()` on a `none` value")
            || s.contains("called `result::unwrap()` on an `err` value")
    }
}

#[cfg(test)]
mod tests {
    use super::{RustStrategy, EvaluationStrategy};

    #[test]
    fn rust_detects_panics() {
        let r = RustStrategy;
        assert!(r.has_exception("thread 'main' panicked at 'oh no!', src/main.rs:1:1"));
        assert!(r.has_exception("panic: something bad happened"));
        assert!(r.has_exception("called `Option::unwrap()` on a `None` value"));
    }

    #[test]
    fn rust_detects_segfaults() {
        let r = RustStrategy;
        assert!(r.has_segfault("Segmentation fault (core dumped)"));
        assert!(r.has_segfault("SIGSEGV"));
    }

    #[test]
    fn rust_flags_sanitizer_and_alloc_errors_as_safety() {
        let r = RustStrategy;
        assert!(r.violates_safety("AddressSanitizer: heap-use-after-free"));
        assert!(r.violates_safety("double free or corruption"));
        assert!(r.violates_safety("memory allocation of 18446744073709551615 bytes failed"));
    }
}
