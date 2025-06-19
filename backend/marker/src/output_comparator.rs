pub trait OutputComparator {
    fn compare(&self);
}

pub struct RegexComparator;

impl OutputComparator for RegexComparator {
    fn compare(&self) {
        // TODO: implement regex-based output comparison
    }
} 