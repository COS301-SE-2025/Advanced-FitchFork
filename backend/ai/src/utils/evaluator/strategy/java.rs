use super::EvaluationStrategy;

pub struct JavaStrategy;

/// Heuristics for the Java toolchain/runtime.
/// - OutOfMemoryError and StackOverflowError are treated as "safety" violations.
/// - Segmentation faults are "segfaults"
/// - Exceptions (including RuntimeException and its subclasses) are "exceptions"
impl EvaluationStrategy for JavaStrategy {
    fn name(&self) -> &'static str { "java" }

    fn violates_safety(&self, stderr: &str) -> bool {
        let s = stderr.to_ascii_lowercase();
        s.contains("hs_err_pid")
            || s.contains("a fatal error has been detected by the java runtime environment")
            || s.contains("sigsegv")
            || s.contains("exception_access_violation")
            || s.contains("problematic frame:")
            || s.contains("outofmemoryerror: direct buffer memory")
            || s.contains("internal error (")
    }

    fn has_segfault(&self, stderr: &str) -> bool {
        let s = stderr.to_ascii_lowercase();
        s.contains("sigsegv")
            || s.contains("exception_access_violation")
            || s.contains("hs_err_pid")
            || s.contains("problematic frame:")
    }

    fn has_exception(&self, stderr: &str) -> bool {
        let s = stderr.to_ascii_lowercase();
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
