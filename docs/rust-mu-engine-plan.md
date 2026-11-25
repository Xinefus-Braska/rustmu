# Rust MU* Building Framework — Master Plan

## Positioning

This plan implements the direction defined in [`POSITIONING.md`](POSITIONING.md).

RustMU* is a Rust-native MU* framework for building persistent, customizable multiplayer worlds. It provides a stable game-facing SDK, a modular server core, and a client portal, so creators can define their own worlds, characters, commands, systems, and rules without rebuilding the underlying engine.

The primary audience is the MU* world builder who is comfortable with Rust or working alongside a Rust developer. RustMU* must therefore remain a general framework rather than become a single canonical game, ruleset, or genre.

## Vision

Build a Rust-based, database-backed MU* framework inspired by Evennia's architecture:

- A **Portal** layer that owns external connections and protocol handling.
- A **Server** layer that owns game state, simulation, commands, and persistence.
- A separately maintained **game directory/crate** where developers implement their own MU* without modifying the engine.

The framework is general enough for MUD, MUSH, MUX, MUCK, and MOO-style games. A combat-style MUD may be used as an early example or validation scenario, but combat is not a required or universal RustMU* feature.

## Architectural principles

1. **Engine/game separation**: The engine contains no game-specific rules. Combat, classes, spells, quests, and content belong to the game crate.
2. **Small stable SDK**: Game code depends on a narrow, versioned public API, not engine internals.
3. **SQL-backed persistence**: SQLite for local development; PostgreSQL intended for production.
4. **In-memory active world**: Keep hot game state in memory; persist deliberately and efficiently.
5. **Safe Rust by default**: Use `unsafe` only in isolated, reviewed places, if at all.
6. **Data-driven content**: Rooms, objects, prototypes, help, and areas should be loadable from files or the database.
7. **Scripting is optional**: Start with a Rust game crate; add Rhai/Lua or WASM only after the core is stable.
8. **Creator control**: The engine provides reusable capabilities; the game crate defines genre, progression, combat, economy, roleplay norms, narrative, and community policy.

## Proposed crate layout

```text
rustmu/
├── Cargo.toml
├── crates/
│   ├── rustmu-portal/          # Telnet/WebSocket/SSH listeners, sessions, protocol
│   ├── rustmu-server/          # world state, commands, scheduler, persistence
│   ├── rustmu-sdk/             # public traits, events, entity model, plugin API
│   ├── rustmu-storage-sqlite/  # SQLite backend
│   ├── rustmu-storage-postgres/# PostgreSQL backend
│   └── rustmu-cli/             # rustmu new / migrate / run
├── games/
│   └── my-combat-mud/
│       ├── Cargo.toml
│       ├── migrations/
│       ├── content/
│       └── src/
└── docs/
    ├── architecture/
    └── decisions/
```

## Core responsibilities

| Component | Owns | Does not own |
|---|---|---|
| Portal | Network listeners, protocol parsing, sessions, rate limits, reconnects | Game rules, SQL, world state |
| Server | Accounts, entities, commands, permissions, ticks, scripts, persistence | Raw sockets |
| SDK | Public traits, events, IDs, context types, plugin contract | Implementation details |
| Game crate | Combat, custom commands, world content, game rules | Engine internals |

## Phase roadmap

### Phase 0 — Foundations and decisions

**Goal:** Establish the technical foundation, repository structure, database strategy, and development workflow before building features.

**Deliverables**

- Cargo workspace with `portal`, `server`, `sdk`, `storage-sqlite`, `storage-postgres`, and `cli` crates.
- Initial architecture decision records.
- Database schema v0 and migration tooling.
- CI pipeline with formatting, linting, tests, and dependency auditing.
- Runnable empty server that starts, loads configuration, connects to a database, and shuts down cleanly.
- Project generator or manual template for creating a new game crate.

**Key decisions**

- Async runtime: likely Tokio.
- Database access: `sqlx` or `sea-orm`.
- Error handling: `thiserror` for library errors, `anyhow` for application-level errors.
- Logging and tracing: `tracing` plus `tracing-subscriber`.
- Configuration: `figment`, `config`, or TOML + environment overrides.
- Password hashing: Argon2.
- Migration tool: `sqlx migrate` or `refinery`.
- Testing: unit tests, integration tests, and a small end-to-end Telnet test.

**Initial database schema**

```sql
CREATE TABLE accounts (
    id UUID PRIMARY KEY,
    username TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    email TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_login_at TIMESTAMPTZ,
    is_superuser BOOLEAN NOT NULL DEFAULT FALSE
);

CREATE TABLE objects (
    id UUID PRIMARY KEY,
    typeclass TEXT NOT NULL,
    key TEXT NOT NULL,
    location_id UUID REFERENCES objects(id),
    owner_id UUID REFERENCES accounts(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE attributes (
    object_id UUID NOT NULL REFERENCES objects(id) ON DELETE CASCADE,
    category TEXT NOT NULL DEFAULT '',
    key TEXT NOT NULL,
    value_json JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (object_id, category, key)
);

CREATE TABLE tags (
    object_id UUID NOT NULL REFERENCES objects(id) ON DELETE CASCADE,
    category TEXT NOT NULL DEFAULT '',
    key TEXT NOT NULL,
    PRIMARY KEY (object_id, category, key)
);

CREATE TABLE exits (
    object_id UUID PRIMARY KEY REFERENCES objects(id) ON DELETE CASCADE,
    destination_id UUID NOT NULL REFERENCES objects(id)
);

CREATE TABLE channels (
    id UUID PRIMARY KEY,
    key TEXT NOT NULL UNIQUE,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE channel_members (
    channel_id UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    PRIMARY KEY (channel_id, account_id)
);

CREATE TABLE scripts (
    id UUID PRIMARY KEY,
    object_id UUID REFERENCES objects(id),
    typeclass TEXT NOT NULL,
    interval_ms BIGINT,
    next_run_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    config_json JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

**Phase 0 tasks**

1. Create the GitHub/Git repository and branch policy.
2. Initialize the Cargo workspace and crate boundaries.
3. Add Rust toolchain configuration, `rustfmt`, `clippy`, and editor settings.
4. Add CI: build, test, clippy, fmt check, dependency audit, and database migration check.
5. Choose and document the async runtime, database access layer, logging, configuration, and error strategy.
6. Implement configuration loading for database URL, bind address, port, and environment.
7. Implement structured logging.
8. Implement database connection pooling and health checks.
9. Create migration v0 with the tables above.
10. Add a minimal `rustmu` CLI with `new`, `migrate`, and `run` subcommands.
11. Create a game template that depends only on `rustmu-sdk`.
12. Write architecture decision records for:
    - Portal/Server separation.
    - SQL persistence strategy.
    - Entity/attribute/typeclass model.
    - SDK stability and semantic versioning.
    - Scripting approach.
13. Define testing strategy and create the first integration test.

**Definition of done**

- `rustmu new mygame` creates a game project.
- `rustmu migrate` applies the initial schema.
- `rustmu run` starts the configured server.
- The server starts and stops cleanly.
- CI passes on every commit.
- Documentation explains how to create a game without modifying engine code.

### Phase 1 — Connections and sessions

**Goal:** Multiple users can connect and exchange text.

**Deliverables**

- TCP/Telnet listener.
- Session creation, identification, input buffering, and output queues.
- UTF-8 and line-ending handling.
- Basic Telnet negotiation.
- Rate limiting and connection limits.
- Portal-to-server message protocol.
- Graceful disconnect handling.
- End-to-end Telnet integration tests.

**Definition of done**

- Multiple Telnet clients can connect simultaneously.
- Each client receives a banner and can send/receive text.
- Malformed or oversized input cannot disrupt the server.
- Disconnects are handled cleanly.

### Phase 2 — Accounts and persistence

**Goal:** Durable identities and secure authentication.

**Deliverables**

- Account creation, login, logout.
- Argon2 password hashing.
- Session persistence and reconnect handling.
- Account-to-character relationship.
- Audit logging.
- Database-backed account storage.

**Definition of done**

- A user can create an account, log in, disconnect, reconnect, and still be recognized.
- Passwords are never stored in plaintext.
- Login attempts and privileged actions are logged.

### Phase 3 — Minimal world model

**Goal:** Implement the core persistent object model.

**Deliverables**

- `Object`, `Location`, `Attribute`, `Tag`, `Exit`, and `Typeclass` model.
- Entity cache and database synchronization.
- Room creation and movement.
- Exit traversal.
- Persistence across restarts.
- Basic `look` command.

**Definition of done**

- A builder can create rooms and exits.
- A character can move between rooms.
- World state survives a server restart.

### Phase 4 — Commands and permissions

**Goal:** Make the engine programmable and secure.

**Deliverables**

- Command trait and registry.
- Parser with aliases, quoting, and error handling.
- Roles: player, builder, admin, superuser.
- Command sets on accounts, characters, rooms, and objects.
- Help system.
- Permission checks.

**Definition of done**

- Game authors can register custom commands from the game crate.
- Commands enforce permissions.
- Help is available for built-in and custom commands.

### Phase 5 — Builder experience

**Goal:** Let authors create content without editing engine code.

**Deliverables**

- `create`, `dig`, `open`, `link`, `set`, and `destroy`.
- Object prototypes.
- Area import/export.
- Content validation.
- Versioned content format.

**Definition of done**

- A small multi-room area can be built and loaded without changing engine code.

### Phase 6 — Scripting and extensibility

**Goal:** Support game-specific behavior cleanly.

**Deliverables**

- Event hooks: login, enter room, command, tick, death, etc.
- Rust game-crate extension model.
- Optional Rhai or Lua scripting.
- Optional WASM plugin host, deferred until needed.

**Definition of done**

- A game can react to common events without modifying the engine.
- Simple builder-authored logic can be added through scripting.

### Phase 7 — Optional combat simulation

**Goal:** Provide reusable simulation infrastructure that a combat-focused game can use.

Combat is an optional game-domain capability, not a universal RustMU* rule. The engine may provide scheduling, events, persistence, and extension points; the game crate owns combat balance, rules, presentation, and policy.

**Deliverables**

- Fixed-step tick scheduler.
- In-memory active-world cache.
- Batched or transactional persistence.
- Combat events and NPC AI hooks, where a concrete game needs them.
- Performance metrics.
- A combat example or reference implementation, only if it clarifies the extension model.

**Definition of done**

- A game crate can implement a simple combat encounter without modifying engine internals.
- Important game state can be recovered after restart.
- Tick duration and database load are measurable.

### Phase 8 — Production hardening

**Goal:** Make the framework dependable for a real game.

**Deliverables**

- Metrics and dashboards.
- Backup and restore procedures.
- Deployment documentation.
- Docker support.
- Security review.
- Load testing.

**Definition of done**

- The server can be deployed, monitored, backed up, and restarted safely.

## Connection protocol decision

### ADR-002: Start with Telnet only

**Status:** Accepted

**Context:** The framework needs a first connection protocol. Traditional MU* clients use Telnet over TCP. WebSocket, SSH, and HTTP-based clients are useful but add complexity before the core game server exists.

**Decision:** Phase 1 implements Telnet over TCP only.

The Portal owns:

- TCP listener and connection lifecycle
- Telnet byte-stream decoding
- UTF-8 line assembly
- Basic Telnet negotiation handling
- Session input and output queues
- Rate limiting and connection limits

The Server receives normalized session events, not raw sockets.

WebSocket, SSH, and HTTP services are deferred. Their future addition must not require changes to game code.

**Consequences**

- The first playable server works with standard MUD clients.
- The Portal/Server boundary must be protocol-agnostic from the beginning.
- Rich-client features such as GMCP, MSDP, MXP, maps, and health bars are deferred.
- A WebSocket transport can be added later behind the same session interface.

## Portal reference: MUD Standards

The primary external reference for Portal protocol development is:

https://mudstandards.org/

MUD Standards is a centralized collection of documents and links for MUD
protocols. It should be consulted before implementing protocol features beyond
basic Telnet input and output.

### Portal protocol roadmap

1. **Phase 1 — Core Telnet**
   - TCP connections
   - UTF-8 line input
   - CRLF normalization
   - Basic IAC handling
   - Session lifecycle and disconnect handling

2. **Phase 2 — Telnet negotiation**
   - WILL / WONT / DO / DONT negotiation
   - Subnegotiation
   - Terminal size negotiation, where applicable
   - Safe rejection of unsupported options

3. **Later — Rich-client protocols**
   - Evaluate GMCP, MSDP, MXP, MCP, and related standards from MUD Standards.
   - Add protocols only when they serve a concrete game or client need.
   - Keep protocol-specific handling isolated in the Portal.
   - Expose protocol capabilities to the Server through normalized session events.

### Architectural rule

The Portal may understand MUD wire protocols, but it must not contain game rules.
Protocol capabilities should be represented as normalized events, session
metadata, or capability flags passed to the Server through the SDK contract.

### Reference scope

Use MUD Standards as the reference for:

- Telnet negotiation behavior.
- Client capability negotiation.
- Out-of-band data protocols.
- Compatibility with existing MUD clients.
- Future web or rich-client integration.

Do not use it as the reference for:

- Combat balance.
- Room and object modeling.
- Database schema.
- Command permissions.
- Game-specific content.

For now, the Portal remains focused on the Telnet-only decision recorded in
ADR-002. When Telnet negotiation begins, use MUD Standards as the specification
source and add one protocol capability at a time behind tests.

## Crate architecture plan

The framework is organized into four crates with a deliberate dependency direction:

```text
rustmu-cli
├── rustmu-portal
├── rustmu-server
└── rustmu-sdk

rustmu-portal ──► rustmu-sdk
rustmu-server ──► rustmu-sdk
```

The Portal and Server must not depend on each other directly. They communicate through protocol-neutral messages defined in `rustmu-sdk`. This preserves Evennia-like separation: the Portal understands network protocols, while the Server understands the game.

## rustmu-sdk

**Purpose:** Define the stable vocabulary shared by the Portal and Server.

This is the smallest and most important crate. It should contain no networking, SQL, game rules, or Tokio dependency.

### Owns

- `SessionId`
- `ClientToServer`
- `ServerToClient`
- `DisconnectReason`
- Basic role and permission enums, when needed
- Shared public error types, only when part of the API contract

### Does not own

- TCP sockets, Telnet parsing, WebSocket frames
- Database connections or migrations
- Rooms, characters, combat, commands
- Configuration loading
- Concrete channel implementations

### Initial API

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SessionId(pub u64);

#[derive(Debug, Clone)]
pub enum ClientToServer {
    Connected {
        session_id: SessionId,
        peer: String,
    },

    Input {
        session_id: SessionId,
        line: String,
    },

    Disconnected {
        session_id: SessionId,
    },
}

#[derive(Debug, Clone)]
pub enum ServerToClient {
    Output {
        session_id: SessionId,
        text: String,
    },

    Disconnect {
        session_id: SessionId,
    },
}
```

### Planning rule

Treat this crate as a public API even if only one game uses it today. Changes to enum variants, trait methods, or public types can be breaking.

### Near-term additions

- `AuthEvent` for login, logout, and account creation.
- `CommandRequest` and `CommandResponse` once the command dispatcher exists.
- `EntityId` once the world model exists.
- Event variants for room entry, combat, ticks, and scripts.

### Do not add yet

- Database models.
- Game-specific combat types.
- Telnet-specific types.
- Configuration structures.

## rustmu-portal

**Purpose:** Connect external clients to the Server and translate their wire protocol into normalized session events.

The Portal is the only crate that knows Telnet exists. It accepts TCP connections, decodes Telnet framing, assigns session IDs, and forwards lines to the Server.

### Owns

- TCP listener and accept loop
- Telnet protocol parsing and negotiation
- UTF-8 decoding and line assembly
- Per-session input and output channels
- Session registry and cleanup
- Connection limits, rate limits, and idle timeouts
- Protocol-specific encoding of output

### Does not own

- Accounts, characters, rooms, combat
- SQL or migrations
- Command parsing beyond extracting a line
- Game permissions
- World state

### Current responsibility

```text
Telnet bytes -> line -> ClientToServer::Input
ServerToClient::Output -> Telnet bytes
```

### Planned modules

```text
crates/rustmu-portal/src/
├── lib.rs
├── config.rs
├── listener.rs
├── session.rs
├── telnet/
│   ├── mod.rs
│   ├── codec.rs
│   └── negotiation.rs
└── limits.rs
```

### Near-term work

1. Move TCP/Telnet code out of `lib.rs` into focused modules.
2. Implement a proper Telnet state machine: IAC, WILL/WONT/DO/DONT, and subnegotiation.
3. Add `PortalConfig` with bind address, port, maximum connections, and rate limits.
4. Improve the per-session output-channel design.
5. Add integration tests for multiple sessions, CRLF normalization, long-line rejection, invalid UTF-8, Telnet IAC handling, and disconnect cleanup.

### Important design decision

The Portal should expose a constructor such as:

```rust
Portal::bind(config, client_tx, server_rx)
```

rather than hardcoding `"0.0.0.0:4000"`. The CLI supplies configuration; the Portal should not read files itself.

## rustmu-server

**Purpose:** Own game state, commands, accounts, persistence, and reusable simulation infrastructure.

This is the MU* engine core. It receives protocol-neutral events from the Portal and decides what happens. It should never know whether a character arrived through Telnet, WebSocket, SSH, or another future protocol.

### Owns

- Game loop and tick scheduler
- World state and entity cache
- Accounts and characters
- Command dispatcher
- Permissions and roles
- Rooms, exits, objects, attributes, tags
- Persistence and database access
- Server-side event handling
- Reusable simulation infrastructure

### Does not own

- TCP sockets
- Telnet byte parsing
- Client protocol negotiation
- Direct knowledge of whether input came from Telnet or another protocol
- Genre-specific rules such as combat balance, progression, economy, or roleplay policy

### Current responsibility

```text
ClientToServer event -> game decision -> ServerToClient response
```

Currently, that decision is only an echo response.

### Planned modules

```text
crates/rustmu-server/src/
├── lib.rs
├── config.rs
├── server.rs
├── session.rs
├── command/
│   ├── mod.rs
│   ├── dispatcher.rs
│   ├── parser.rs
│   └── commands/
├── world/
│   ├── mod.rs
│   ├── entity.rs
│   ├── room.rs
│   ├── exit.rs
│   └── attribute.rs
├── account/
│   ├── mod.rs
│   ├── auth.rs
│   └── character.rs
├── persistence/
│   ├── mod.rs
│   ├── sqlite.rs
│   └── postgres.rs
└── simulation/
    └── mod.rs
```

### Near-term work

1. Replace the echo handler with a command dispatcher.
2. Add a `Command` trait and command registry.
3. Implement `look`, `say`, `pose`, and `help` against an in-memory world.
4. Add accounts and password authentication.
5. Add persistence behind a `Storage` trait.
6. Add a tick scheduler before optional combat work.
7. Add combat only as a game-domain example or extension after rooms, movement, entities, and commands are stable.

### Important design decision

The Server should expose:

```rust
GameServer::new(config, storage)
GameServer::run(client_rx, server_tx)
```

The CLI supplies configuration and a storage implementation. The Server should not decide whether it uses SQLite or PostgreSQL.

## rustmu-cli

**Purpose:** Compose and launch the application.

The CLI is a thin binary. It should contain as little logic as possible: parse arguments, load configuration, initialize tracing, construct the Portal and Server, connect their channels, and run until shutdown.

### Owns

- Command-line arguments
- Configuration file discovery and loading
- Logging initialization
- Database migration invocation
- Process lifecycle and graceful shutdown
- Wiring Portal and Server together

### Does not own

- Game rules
- Telnet protocol details
- Database queries
- World simulation
- Command implementations

### Planned commands

```bash
rustmu run
rustmu migrate
rustmu new mygame
rustmu check
rustmu admin create-account
```

For Phase 0 and Phase 1, prioritize:

```bash
rustmu run
rustmu migrate
```

### Planned structure

```text
crates/rustmu-cli/src/
├── main.rs
├── args.rs
├── config.rs
├── logging.rs
└── commands/
    ├── mod.rs
    ├── run.rs
    ├── migrate.rs
    └── new.rs
```

### Near-term work

1. Add `clap` argument parsing.
2. Load `rustmu.toml` and environment overrides.
3. Move the bind address out of `main.rs`.
4. Add graceful shutdown on `Ctrl+C`.
5. Add `rustmu migrate`.
6. Add `rustmu new mygame` once the SDK and game-template layout are stable.

## Crate rules

| Rule | Why |
|---|---|
| SDK has no Tokio dependency | Keeps the contract portable and easy to test |
| Portal never imports Server | Prevents protocol details from leaking into game logic |
| Server never imports Portal | Keeps game logic protocol-agnostic |
| CLI imports both | It is the composition root |
| No game rules in Portal | Portal remains reusable for any MU* |
| No Telnet code in Server | Server can later support WebSocket without changes |
| Configuration enters through CLI | Components remain testable and configurable |
| Database access stays behind a trait | SQLite and PostgreSQL can coexist |
| Combat and genre systems remain in game code | RustMU* remains a general creator-controlled framework |

## Immediate crate sequence

1. **SDK:** Stabilize `SessionId`, `ClientToServer`, and `ServerToClient`; add tests for event semantics.
2. **Portal:** Extract Telnet handling into modules; add configuration and robust protocol tests.
3. **Server:** Introduce `GameServer`, command dispatcher, and an in-memory world.
4. **CLI:** Add `clap`, configuration loading, graceful shutdown, and `rustmu migrate`.
5. **Persistence:** Add a `Storage` trait in Server, then implement SQLite and PostgreSQL backends.

The first meaningful Server milestone is not combat. It is a command dispatcher that can handle `look`, `say`, and `pose` in an in-memory world. That provides the foundation for accounts, persistence, building, and eventually game-specific systems such as combat.