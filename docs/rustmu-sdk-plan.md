# RustMU SDK Plan

## Positioning

This plan implements the SDK portion of the direction defined in [`POSITIONING.md`](POSITIONING.md).

RustMU* is a Rust-native MU* framework for building persistent, customizable multiplayer worlds. Its primary audience is the MU* world builder who is comfortable with Rust or working alongside a Rust developer.

`rustmu-sdk` exists to protect that creator-control promise: it gives game code a stable way to use engine capabilities without requiring game code to depend on engine internals, transport details, persistence details, or a particular genre’s rules.

## Purpose

`rustmu-sdk` is the stable, game-facing contract between custom game code and the evolving RustMU engine. It defines the vocabulary and extension points that game code may rely on while leaving engine implementation, persistence, scheduling, transport, and delivery mechanics to `rustmu-server` and `rustmu-portal`.

The SDK should describe **what game code may do**. The server decides **how the engine does it**.

## Current state

The SDK is a library-only crate at `crates/rustmu-sdk`.

### Implemented contracts

| Area | Current contract |
|---|---|
| Session identity | `SessionId` identifies a connected client session. |
| Session messages | `ClientToServer` and `ServerToClient` define the initial Portal-facing message vocabulary. |
| Characters | `Character` and `CharacterId` represent the in-world actor. |
| Commands | `Command`, `CommandContext`, `CommandHandler`, `CommandHandlerResult`, `CommandRegistry`, and `CommandRegistrationResult` define command handling and registration. |
| World access | `World` provides read-oriented room description and occupant access. |
| Built-in command | `LookCommand` is an initial reference implementation for room inspection. |
| Tests | `crates/rustmu-sdk/tests/look_tests.rs` exercises `LookCommand` through the public SDK API. |

### Verification status

The `LookCommand` integration tests have been committed, but their execution has not yet been confirmed in this planning record.

Before treating the current SDK baseline as validated, run:

```bash
cargo test -p rustmu-sdk
```

## Architectural boundary

The SDK must not expose or assume:

- Database connections or persistence details.
- Tokio tasks, channels, runtimes, or server scheduling.
- Socket, Telnet, WebSocket, or other Portal transport internals.
- Server process layout, service discovery, or internal error handling.
- Game-specific rules, content, balance, permissions, or presentation policy.

The SDK may expose narrow, explicit capabilities. The server owns their implementation and enforcement.

## Customization boundary

Game code under `/game` owns game-specific behavior and policy. The SDK provides reusable contracts and, where useful, optional reference handlers.

A later developer must be able to:

- Choose whether to register an SDK-provided command.
- Replace an SDK-provided handler with a game-specific implementation.
- Add aliases, validation, permissions, formatting, and game rules in `/game`.
- Use SDK world capabilities without adopting SDK reference-handler behavior.

An SDK reference handler is a convenience implementation, not a permanent gameplay rule.

For example, `/game` may replace `LookCommand` with a custom handler that uses the same `World` read capabilities but changes output, hides occupants, adds lore, or applies game-specific visibility rules.

## Current constraints

The current SDK contracts intentionally constrain what game code can express.

- `World` is currently room-oriented and read-only.
- `CommandOutput` currently carries a single text value.
- `CommandHandlerResult` currently has only success-with-output and error-with-message variants.
- `CommandRegistry` does not yet define aliases, replacement, lookup order, or authorization.
- There is no write-oriented world contract, event contract, or movement contract.

These limitations are acceptable for the initial look-command loop. They must be addressed explicitly when a feature requires them; they should not be solved through broad or speculative APIs.

## Decision principles

1. **Define capabilities, not policies.**

   The SDK may provide a way to request movement or communication. It should not fix movement costs, language rules, moderation policy, item rarity, or combat balance.

2. **Keep the actor vocabulary consistent.**

   Use `Character` for the in-world acting participant. Do not substitute `Player`.

3. **Keep transport out of game contracts.**

   Portal translates client input into SDK-level messages. Game code must not depend on a particular network protocol.

4. **Prefer explicit contracts over convenience.**

   Add a world, event, or output capability only when a concrete feature requires it.

5. **Treat built-in handlers as optional.**

   SDK handlers may demonstrate a capability, but `/game` decides which handlers are registered and how they behave.

6. **Document compatibility carefully.**

   Once an SDK type, trait method, enum variant, or public field becomes part of a released contract, changes require deliberate compatibility planning.

7. **Preserve creator control.**

   The SDK must not make one genre, combat model, economy, progression system, or roleplay policy mandatory for every RustMU* world.

## Candidate commands

The following commands are **candidates**, not approved SDK APIs or a committed implementation order.

| Command | Example | Potential capability | Important customization concern |
|---|---|---|---|
| `MoveCommand` | `north` | Request movement through an available exit | Direction names, blocked exits, movement cost |
| `SayCommand` | `say Hello.` | Emit room-scoped speech | Visibility, language, NPC response |
| `ExamineCommand` | `examine guard` | Inspect a visible entity | Hidden details, identification, permissions |
| `TellCommand` | `tell Guard Hello.` | Send a private message | Privacy, blocking, moderation |
| `InventoryCommand` | `inventory` | Read a character's inventory | Capacity, item classes, encumbrance |
| `GetCommand` | `get sword` | Request transfer from room to inventory | Ownership, weight, theft, stacking |
| `DropCommand` | `drop sword` | Request transfer from inventory to room | Ownership, visibility, item persistence |
| `WhoCommand` | `who` | List connected characters | Visibility, staff powers, sorting |

These candidates should not be implemented merely because they are conventional MUD commands. Each requires an explicit review of the world, event, output, and authorization contracts it would introduce.

## Proposed way ahead

### Immediate validation

1. Run `cargo test -p rustmu-sdk`.
2. Resolve any compile, test, formatting, or lint failures.
3. Establish a clean local baseline before expanding the SDK.

### Contract review

Before implementing another command, review whether the proposed capability:

- Can be expressed through the existing `World` contract.
- Requires a new read capability.
- Requires a write-oriented world operation.
- Requires event delivery beyond `CommandOutput`.
- Requires authorization or permission semantics.
- Can remain customizable in `/game`.
- Preserves RustMU*’s positioning as a creator-controlled MU* framework.

### Likely first expansion

`MoveCommand` is the most likely next expansion because movement is a foundational capability after room inspection. However, its contract must be designed before implementation.

A movement design should explicitly answer:

- How does the SDK represent exits?
- How does game code request movement?
- What happens when an exit is unavailable?
- Does movement produce output, events, or both?
- Does the SDK expose a write operation directly, or does it define a request contract consumed by the server?
- How can `/game` alter movement rules without replacing unrelated engine behavior?

No movement API should be committed until these questions have documented answers.

## Documentation practices

- Keep this plan current as SDK contracts evolve.
- Record durable architectural choices in `docs/decisions/`.
- Mark unimplemented ideas clearly as proposals.
- Use accurate Rust documentation (`///`) and non-documentation comments (`//`) in all SDK source files.
- Use Rust edition 2024 and prefer minimal, explicit APIs.
- Treat [`POSITIONING.md`](POSITIONING.md) as the strategic reference when evaluating new SDK capabilities.