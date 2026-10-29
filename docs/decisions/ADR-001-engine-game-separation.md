# ADR-001: Engine and game separation

## Status

Accepted

## Context

The framework should allow game developers to create MU* games without modifying
engine source code.

## Decision

The repository will contain reusable engine crates and separately maintained game
crates. Game crates may depend only on the public SDK crate.

## Consequences

- Engine updates should not require changes to game logic.
- The SDK must be small, stable, and semantically versioned.
- Game-specific rules must not be placed in engine crates.