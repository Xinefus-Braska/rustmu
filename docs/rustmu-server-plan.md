# RustMU Server Plan: Command Dispatcher and Minimal World

## Purpose

This plan defines the first `rustmu-server` implementation slice: a command dispatcher and minimal in-memory world that consume the SDK contracts for `look`, `movement`, and `say`.

The SDK remains the contract boundary. The server orchestrates sessions, dispatch, and world access. The world owns the game rules and state.

## Architectural boundary

```text
Client / Portal
      │ SDK command
      ▼
rustmu-server
┌───────────────────────────────────────────────┐
│ Session layer                                 │
│  - identifies caller                          │
│  - validates session state                    │
│  - submits a Command                          │
└───────────────┬───────────────────────────────┘
                ▼
┌───────────────────────────────────────────────┐
│ Command dispatcher                            │
│  - routes Command -> handler                  │
│  - returns DispatchOutcome                    │
│  - owns no game rules                         │
└───────────────┬───────────────────────────────┘
                ▼
┌───────────────────────────────────────────────┐
│ World facade                                  │
│  - transactional entry point                  │
│  - resolves actor / location                  │
│  - emits events                               │
└───────────────┬───────────────────────────────┘
                ▼
┌───────────────────────────────────────────────┐
│ Minimal in-memory world                       │
│  - actors, rooms, exits, contents             │
│  - look / movement / say state                │
└───────────────────────────────────────────────┘
```

### Ownership

| Layer | Owns | Must not own |
|---|---|---|
| `rustmu-sdk` | Command vocabulary, arguments, outcomes, and domain errors | Server session state or world storage |
| `rustmu-server` | Session handling, dispatch, authorization boundaries, and world access | Look, movement, or speech rules |
| In-memory world | Actors, rooms, exits, occupancy, and command effects | Transport, protocol formatting, or SDK parsing |
| Portal / CLI | Builds SDK commands and renders SDK outcomes | Game-state mutation or dispatcher internals |

## Module layout

Begin with this layout inside `crates/rustmu-server/src`:

```text
src/
├── lib.rs
├── session.rs
├── command/
│   ├── mod.rs
│   ├── dispatcher.rs
│   └── handler.rs
├── world/
│   ├── mod.rs
│   ├── facade.rs
│   ├── memory.rs
│   ├── actor.rs
│   ├── room.rs
│   └── event.rs
└── error.rs
```

| Module | Responsibility | Must not do |
|---|---|---|
| `session` | Track an actor and validate that it may issue a command | Interpret `look`, `move`, or `say` |
| `command::dispatcher` | Map an SDK command to a handler and execute it | Contain room, movement, or speech rules |
| `command::handler` | Convert one SDK command into a world operation | Reach directly into world internals |
| `world::facade` | Provide the only game-facing API used by handlers | Know about transport, sessions, or SDK parsing |
| `world::memory` | Implement initial in-memory storage and queries | Define SDK types or command contracts |
| `world::event` | Describe what happened and who should observe it | Format terminal output or protocol messages |

## SDK contract boundary

The server consumes SDK types rather than defining a parallel command vocabulary. This prevents the dispatcher from becoming a second source of truth for `look`, `movement`, and `say`.

The SDK should own:

- `Command`: the command enum or command abstraction.
- `CommandContext`: caller identity and permitted session metadata.
- `CommandOutcome`: a successful result and presentation-neutral data.
- `CommandError`: typed failures such as unknown command, unauthorized actor, invalid target, or invalid movement.
- `WorldEvent`: a domain-level observation, such as `ActorLooked`, `ActorMoved`, or `ActorSpoke`.

The server may define transport-specific wrappers, but those wrappers must convert to or from SDK contracts rather than replace them.

## Dispatcher design

Use a static command registry first. A `match` over the SDK command enum is preferable to a plugin registry for the initial implementation because it is explicit, easy to test, and makes unsupported commands a compile-time concern.

```rust
// Determines how an SDK command was handled by the dispatcher.
pub enum DispatchOutcome {
    // The command completed and produced one or more world events.
    Completed(Vec<WorldEvent>),

    // The command was recognized but could not be applied.
    Rejected(CommandError),
}
```

```rust
// Routes a validated SDK command to the appropriate world operation.
pub struct CommandDispatcher<W> {
    // The world facade used by every command handler.
    world: W,
}

impl<W> CommandDispatcher<W>
where
    W: WorldFacade,
{
    // Executes one command and returns its domain-level outcome.
    pub fn dispatch(
        &self,
        context: &CommandContext,
        command: Command,
    ) -> DispatchOutcome {
        match command {
            Command::Look(arguments) => self.handle_look(context, arguments),
            Command::Move(arguments) => self.handle_move(context, arguments),
            Command::Say(arguments) => self.handle_say(context, arguments),
        }
    }
}
```

Keep `handle_look`, `handle_move`, and `handle_say` private dispatcher methods or separate handler types. Their sole responsibility is command-specific validation and delegation to the world facade.

## Minimal world model

The initial world includes only the state required by `look`, `movement`, and `say`.

```text
World
├── actors: ActorId -> Actor
├── rooms: RoomId -> Room
└── locations: ActorId -> RoomId

Actor
├── id
├── display_name
└── current_room

Room
├── id
├── name
├── description
├── exits: Direction -> RoomId
└── occupants: ActorId
```

Do not add entities, components, scripting, permissions, inventory, persistence, or asynchronous event buses in the first slice.

### Initial invariants

- Every actor has exactly one current room.
- Every room occupant is registered in `actors`.
- Every exit points to an existing room.
- Movement updates the source room, destination room, and actor location atomically.
- A `say` event reaches actors in the speaker’s current room.
- A `look` result is visible only to the acting actor unless observer rules are explicitly added.

## Command flows

### Look

```text
SDK Look
  -> dispatcher
  -> world.look(actor)
  -> resolve current room
  -> build room description
  -> emit ActorLooked
```

`look` is read-only in the first version. It returns the room name, description, exits, and visible occupants without mutating world state.

### Move

```text
SDK Move
  -> dispatcher
  -> world.move_actor(actor, direction)
  -> resolve current room
  -> resolve exit
  -> remove actor from source room
  -> add actor to destination room
  -> update actor location
  -> emit ActorMoved
```

Movement is the first stateful command. It must be atomic: an actor either remains in the source room or fully arrives in the destination room.

### Say

```text
SDK Say
  -> dispatcher
  -> world.say(actor, message)
  -> resolve current room
  -> collect room occupants
  -> emit ActorSpoke
```

`say` does not alter room or actor state. Its result is an event containing the speaker, message, room, and intended audience.

## Event model

Use domain events as the server’s stable output contract. Do not return formatted strings from the world.

```rust
// Describes an observable consequence of a completed command.
pub enum WorldEvent {
    // An actor observed a room.
    ActorLooked {
        // The actor that issued the look command.
        actor: ActorId,

        // The room that was observed.
        room: RoomId,
    },

    // An actor moved from one room to another.
    ActorMoved {
        // The actor that moved.
        actor: ActorId,

        // The room previously occupied by the actor.
        from: RoomId,

        // The room now occupied by the actor.
        to: RoomId,
    },

    // An actor spoke within a room.
    ActorSpoke {
        // The actor that spoke.
        actor: ActorId,

        // The room in which the speech occurred.
        room: RoomId,

        // The message spoken by the actor.
        message: String,
    },
}
```

The portal or CLI converts events into user-facing text. Future transports—including WebSocket, Telnet, HTTP, or a graphical client—should not depend on the world’s internal formatting.

## Concurrency approach

Use a single-threaded in-memory world behind an explicit synchronization boundary for the first implementation.

```text
Session task
      │ Command
      ▼
World handle
      │ Lock / actor mailbox
      ▼
In-memory world
```

| Option | Best fit | Tradeoff |
|---|---|---|
| `Mutex<InMemoryWorld>` | Simplest first implementation; commands serialize naturally | Can become a bottleneck as complexity grows |
| Dedicated world task with command channel | Better long-term shape for ordered simulation | More moving parts and upfront design |

Start with `Mutex<InMemoryWorld>` only when command handling is short and synchronous. If handlers may await I/O, use a dedicated world task receiving commands through a bounded channel. Never hold a world lock across an `.await`.

The core rule is that dispatch is synchronous and world mutation is serialized. This provides deterministic behavior for movement and speech before persistence or distributed state is introduced.

## Implementation sequence

Implement the first slice in five independently testable steps.

1. **Define the world facade trait**
   - Add `WorldFacade` with `look`, `move_actor`, and `say`.
   - Use SDK command arguments and return SDK-compatible outcomes.

2. **Build the in-memory world**
   - Implement `InMemoryWorld`, `Actor`, `Room`, `ActorId`, and `RoomId`.
   - Create a small room graph with valid exits.

3. **Implement `look`**
   - Resolve the caller’s room.
   - Return a room description without mutating state.

4. **Implement `move`**
   - Resolve an exit.
   - Update source occupancy, destination occupancy, and actor location atomically.
   - Emit `ActorMoved`.

5. **Implement `say`**
   - Resolve room occupants.
   - Emit `ActorSpoke`.
   - Do not mutate world state.

Implement the dispatcher after the world facade has a stable shape. This prevents the dispatcher from becoming the place where game rules live.

## Testing strategy

Use unit tests at the world boundary and integration tests at the dispatcher boundary.

- **World tests:** Verify look output, valid and invalid movement, room occupancy changes, and speech audience.
- **Dispatcher tests:** Verify SDK command routing, unknown-command handling, and conversion of world errors into SDK errors.
- **Boundary tests:** Ensure the server does not construct game rules independently of SDK contracts.
- **Concurrency tests:** Prove that concurrent movement requests cannot place one actor in two rooms or lose an occupancy update.

### Acceptance test

```text
Given two actors in Room A
When Actor 1 moves east to Room B
Then Actor 1 is in Room B
And Actor 2 remains in Room A
And Room A no longer lists Actor 1
And Room B lists Actor 1
```

## Deferred work

Do not build these capabilities in the first dispatcher and world slice:

- Persistent storage or database migrations
- Authentication, accounts, and password handling
- Permissions, roles, or moderation
- NPCs, combat, inventory, or scripting
- Asynchronous event subscriptions
- Multiple concurrent worlds or sharding
- Complex parsers, aliases, or command history

The first target is narrow: a connected actor can look, move through a small room graph, and speak to occupants of its current room, with all command contracts owned by the SDK.
