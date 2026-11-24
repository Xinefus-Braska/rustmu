# ADR-005: Command Extension Contract

## Status

Accepted

## Context

Game code needs a stable way to implement character-initiated behavior without depending on raw client input, server scheduling, or transport details.

## Decision

The SDK defines commands as structured intent rather than raw input lines.

A command consists of:

- A normalized command name.
- Optional arguments supplied after the command name.

Portal or the server owns parsing transport input into a `Command`.

A `CommandHandler` receives:

- A parsed `Command`.
- A `CommandContext` identifying the acting `Character` and optional session.
- A read-oriented `World` implementation.

A handler returns a `CommandHandlerResult`.

## Rationale

This separation keeps game handlers focused on game behavior. It also prevents game code from becoming coupled to a particular input format, network protocol, or server execution model.

## Consequences

- Game code receives structured commands rather than raw transport text.
- Command parsing remains outside the SDK command-handler contract.
- Handlers receive explicit world access rather than unrestricted server access.
- Successful handlers produce `CommandOutput`.
- Handler failures return a game-facing error message.
- The server owns delivery, persistence, logging, and broader error handling.
- The SDK does not yet define aliases, command replacement, authorization, or registry lookup order.
