use super::EvaluationStrategy;

pub struct DefaultStrategy;

impl EvaluationStrategy for DefaultStrategy {
    fn name(&self) -> &'static str { "default" }
}