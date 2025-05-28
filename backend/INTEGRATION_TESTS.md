# Integration Testing Guide

## Purpose

Integration tests verify how multiple components of your application work together. These tests often involve full HTTP requests, database access, or interacting with services across module boundaries.

## Location

Place integration tests in the `tests/` directory at the root of the crate. Each file inside this folder is compiled as a separate crate.

Example structure:

```
api/
├── src/
│   └── ...
├── tests/
│   └── api_behavior.rs
```

## Best Practices

- Name files and functions clearly (e.g., `test_auth_login_success()`).
- Import and reuse public APIs from your main crate (exposed via `lib.rs`).
- Use `tower::ServiceExt::oneshot()` to test Axum routers without a real server.
- Prefer a stable, seeded or in-memory database for reproducibility.
- Group related tests in the same file when possible.

---
