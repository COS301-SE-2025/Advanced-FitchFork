// This is the strategy module for evaluating program crashes based on language-specific heuristics.
// If you add a new language, consider adding a corresponding strategy here.

use util::languages::Language;
/// An evaluation strategy defines heuristics for determining whether a program
/// crash was due to a safety violation, segmentation fault, or exception.
/// The default implementation returns false for all heuristics. As a language cannot be found
pub trait EvaluationStrategy: Send + Sync {
    fn name(&self) -> &'static str;
    fn violates_safety(&self, _stderr: &str) -> bool { false }
    fn has_segfault(&self, _stderr: &str) -> bool { false }
    fn has_exception(&self, _stderr: &str) -> bool { false }
}

/// Module exports and strategy selection
pub use go::GoStrategy;
pub use rust_lang::RustStrategy;
pub use cpp::CppStrategy;
pub use java::JavaStrategy;
pub use default::DefaultStrategy;


// Static instances of each strategy to avoid repeated allocations
static CPP: CppStrategy = CppStrategy;
static JAVA: JavaStrategy = JavaStrategy;
static GO_S: GoStrategy = GoStrategy;
static RUST_S: RustStrategy = RustStrategy;
static DEFAULTS: DefaultStrategy = DefaultStrategy;

/// Get the appropriate evaluation strategy for a given language.
/// If no specific strategy exists for the language, returns the default strategy.
pub fn strategy_for(lang: Language) -> &'static dyn EvaluationStrategy {
    match lang {
        Language::Cpp => &CPP,
        Language::Java => &JAVA,
        Language::Go => &GO_S,
        Language::Rust => &RUST_S,
        _ => &DEFAULTS,
    }
}

// Submodules for each language strategy
mod cpp;
mod java;
mod go;
mod rust_lang;
mod default;
