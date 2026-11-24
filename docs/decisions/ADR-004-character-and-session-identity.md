# ADR-004: Character and Session Identity

## Status

Accepted

## Context

RustMU needs a consistent vocabulary for the participant that acts within the game world. It also needs to distinguish a connected client session from the in-world character associated with that session.

## Decision

Use `Character` as the SDK vocabulary for the in-world acting participant.

Use `CharacterId` to identify a character.

Use `SessionId` to identify a connected client session.

A `CommandContext` may contain both a `Character` and an optional `SessionId`.

## Rationale

A character can exist independently of a connected client. For example, an NPC, scripted actor, or future non-client actor may initiate a command without possessing a session.

Therefore, game code must not assume that every command has a session.

## Consequences

- `Character` is the required term for in-world actors in SDK APIs.
- `Player` must not replace `Character` in SDK contracts.
- Session presence is optional and explicitly represented with `Option<SessionId>`.
- Commands can support both client-initiated and non-client-initiated actors.
- The relationship between characters and sessions remains a server concern.
