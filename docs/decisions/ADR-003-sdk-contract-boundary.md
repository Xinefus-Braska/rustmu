# ADR-003: SDK Contract Boundary

## Status

Accepted

## Context

RustMU separates custom game code from the evolving base engine. Without a clear boundary, game code could become dependent on server internals, transport details, persistence mechanisms, or implementation choices that need to change as the engine evolves.

## Decision

`rustmu-sdk` is the stable, game-facing contract layer between game code and the engine.

The SDK defines:

- Core domain vocabulary.
- Command handling and registration contracts.
- Read-oriented world access.
- Session and message contracts where needed.
- Explicit extension points for future engine capabilities.

The SDK does not expose:

- Database connections or persistence details.
- Tokio tasks, channels, runtimes, or scheduling.
- Socket, Telnet, WebSocket, or other transport internals.
- Server process layout or internal diagnostics.
- Game-specific rules, content, permissions, or balance policy.

## Consequences

- Game code depends on SDK contracts rather than server internals.
- The server can evolve its implementation without unnecessarily changing the game-facing API.
- New capabilities must be introduced through explicit SDK contracts.
- The SDK must avoid speculative APIs that constrain future engine or game design.
- `/game` retains ownership of game-specific behavior and policy.
