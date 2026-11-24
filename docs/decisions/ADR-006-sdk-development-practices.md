# ADR-006: SDK Development Practices

## Status

Accepted

## Context

The SDK is intended to remain small, stable, understandable, and testable as RustMU evolves. Inconsistent documentation, accidental binary targets, or test fixtures in the public API would weaken that goal.

## Decision

`rustmu-sdk` is a library-only crate.

SDK source files use accurate Rust documentation and non-documentation comments:

- Use `///` for public API documentation.
- Use `//!` for module-level documentation.
- Use `//` for implementation notes and non-documentation comments.

SDK tests that exercise the public API belong in `crates/rustmu-sdk/tests/`.

The project uses Rust edition 2024.

## Rationale

Keeping the crate library-only prevents an accidental binary target from becoming part of the SDK contract. Placing public-API integration tests under `tests/` keeps test fixtures out of the production library and validates the API as external game code would use it.

## Consequences

- The SDK does not include a `main.rs`.
- Test fixtures are not exported through the public SDK API.
- Public APIs require clear documentation.
- Implementation details require explanatory non-documentation comments where useful.
- New SDK capabilities should be introduced with focused tests.
- Changes should be made one deliberate step at a time.
