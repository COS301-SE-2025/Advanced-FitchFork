use util::languages::Language;

pub trait EvaluationStrategy: Send + Sync {
    fn name(&self) -> &'static str;
    fn violates_safety(&self, _stderr: &str) -> bool { false }
    fn has_segfault(&self, _stderr: &str) -> bool { false }
    fn has_exception(&self, _stderr: &str) -> bool { false }
}

pub use cpp::CppStrategy;
pub use java::JavaStrategy;
pub use default::DefaultStrategy;

static CPP: CppStrategy = CppStrategy;
static JAVA: JavaStrategy = JavaStrategy;
static DEFAULTS: DefaultStrategy = DefaultStrategy;

pub fn strategy_for(lang: Language) -> &'static dyn EvaluationStrategy {
    match lang {
        Language::Cpp  => &CPP,
        Language::Java => &JAVA,
        _ => &DEFAULTS,
    }
}

mod cpp;
mod java;
mod default;
