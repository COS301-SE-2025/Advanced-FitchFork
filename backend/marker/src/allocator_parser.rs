pub trait AllocatorParser {
    fn parse(&self);
}

pub struct JsonAllocatorParser;

impl AllocatorParser for JsonAllocatorParser {
    fn parse(&self) {
        // TODO: implement JSON allocator parsing
    }
} 