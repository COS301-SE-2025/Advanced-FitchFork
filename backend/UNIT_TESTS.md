# Unit Testing Guide

## Purpose

Unit tests are designed to verify the smallest testable parts of your application in isolation. They should be fast, deterministic, and not depend on any external systems (like a real database or HTTP service).

## Location

Place unit tests inside the module they are testing, typically at the bottom of the `.rs` source file, within a `#[cfg(test)] mod tests` block.

Example:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn some_unit_test() {
        // Test code here
    }
}
```

## Best Practices

- Keep unit tests close to the logic they test.
- Use dependency injection or trait abstraction to avoid side effects.
- Mock out database or network access using test stubs or crates like `mockall`.
- Focus on logic, data transformations, and error handling.

---
