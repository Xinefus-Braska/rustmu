# RustMU*

RustMU* is a Rust-native MU* framework for building persistent, customizable multiplayer worlds. It provides a stable game-facing SDK, a modular server core, and a client portal, so creators can define their own worlds, characters, commands, systems, and rules without rebuilding the underlying engine.

It is inspired by [Evennia](https://www.evennia.com/), particularly its separation of reusable engine infrastructure from game-specific code. RustMU* is an independent project with its own architecture, goals, and implementation.

## Positioning

RustMU* is a foundation for building MUDs, MUSHes, MUXes, MUCKs, and other persistent text-based multiplayer worlds. It is not a finished game, a fixed ruleset, or a Diku/Merc port.

The framework separates reusable engine infrastructure from game-specific design:

- `rustmu-sdk` defines the stable game-facing contracts and extension points.
- `rustmu-server` provides the core engine behavior.
- `rustmu-portal` handles the network boundary and client sessions.
- `rustmu-cli` provides a thin launcher and operational entry point.
- Game-specific code defines the world’s rules, content, systems, and experience.

The SDK describes what game code may do; the server decides how the engine does it.

## Goals

- Provide a reusable engine for text-based multiplayer worlds.
- Keep game-specific rules, content, and combat out of the engine.
- Use a small, stable SDK as the contract between engine components and games.
- Support SQLite for local development and PostgreSQL for production deployments.
- Favor safe Rust, structured logging, configuration-driven startup, and testable components.
- Begin with Telnet support while leaving room for WebSocket, SSH, and richer client protocols later.

## Architecture

RustMU* follows a Portal/Server separation inspired by Evennia:

```text
Client -> Portal -> SDK messages -> Server -> SDK messages -> Portal -> Client
```

| Component | Responsibility |
|---|---|
| `rustmu-portal` | TCP/Telnet listeners, sessions, protocol parsing, rate limits, and disconnect handling |
| `rustmu-server` | Accounts, world state, commands, permissions, simulation, persistence, and reusable simulation infrastructure |
| `rustmu-sdk` | Protocol-neutral session, command, world, and message contracts shared by Portal and Server |
| `rustmu-cli` | Configuration, logging, migrations, process lifecycle, and wiring components together |
| Game crate | Game rules, commands, content, combat, and custom behavior |

The Portal and Server do not depend directly on one another. They communicate through protocol-neutral messages defined by the SDK, allowing future transports without changing game code.

## Workspace layout

```text
rustmu/
├── crates/
│   ├── rustmu-portal/
│   ├── rustmu-server/
│   ├── rustmu-sdk/
│   ├── rustmu-storage-sqlite/
│   ├── rustmu-storage-postgres/
│   └── rustmu-cli/
├── docs/
│   ├── decisions/
│   ├── POSITIONING.md
│   ├── rust-mu-engine-plan.md
│   └── rustmu-sdk-plan.md
└── games/
    └── my-combat-mud/
```

## Current status

The project is in **Phase 0: Foundations and decisions**.

Completed groundwork includes:

- A Cargo workspace and initial crate structure.
- Rust toolchain configuration.
- An initial architecture and phased implementation plan.
- A Telnet Portal and GameServer session loop.
- A documented master plan, SDK plan, positioning statement, and architecture decision records.

## Roadmap

1. **Phase 0 — Foundations:** Workspace structure, database schema, migrations, configuration, logging, CI, and a runnable empty server.
2. **Phase 1 — Connections and sessions:** Telnet listener, multiple concurrent sessions, UTF-8 handling, rate limits, and integration tests.
3. **Phase 2 — Accounts and persistence:** Account creation, secure authentication, reconnect handling, and database-backed identities.
4. **Phase 3 — Minimal world model:** Objects, rooms, exits, attributes, tags, movement, and persistence.
5. **Phase 4 — Commands and permissions:** Command registry, parser, roles, permissions, and help.
6. **Phase 5 — Builder experience:** Content creation, prototypes, area import/export, and validation.
7. **Phase 6 — Scripting and extensibility:** Event hooks, Rust game extensions, and optional scripting.
8. **Phase 7 — Optional combat simulation:** Tick scheduler, active-world cache, game-domain combat extension points, and performance instrumentation.
9. **Phase 8 — Production hardening:** Metrics, backups, deployment, Docker, security review, and load testing.

## Planned CLI

```bash
rustmu run
rustmu migrate
rustmu new mygame
rustmu check
rustmu admin create-account
```

The immediate priorities are `rustmu run` and `rustmu migrate`.

## Development

### Prerequisites

- A current Rust toolchain
- Cargo
- SQLite for local development
- PostgreSQL for production-oriented development and testing

### Build and test

```bash
cargo build
cargo test
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

## Design principles

- **Engine/game separation:** The engine contains no game-specific rules.
- **Small stable SDK:** Game code depends on a narrow, versioned public API.
- **SQL-backed persistence:** SQLite for development; PostgreSQL intended for production.
- **In-memory active world:** Hot game state remains in memory, with deliberate persistence.
- **Safe Rust by default:** Use `unsafe` only in isolated, reviewed cases, if at all.
- **Data-driven content:** Rooms, objects, prototypes, help, and areas are loadable from files or the database.
- **Creator control:** The engine provides reusable capabilities; the game crate defines genre, progression, combat, economy, roleplay norms, narrative, and community policy.
- **Optional scripting:** Start with Rust game crates; consider Rhai, Lua, or WASM after the core is stable.

## Documentation

- [Positioning and target audience](docs/POSITIONING.md)
- [Rust MU* Engine Master Plan](docs/rust-mu-engine-plan.md)
- [RustMU SDK Plan](docs/rustmu-sdk-plan.md)
- [Architecture decision records](docs/decisions/)
- [Credits and prior art](CREDITS.md)

## License

RustMU* is dual-licensed under either the MIT License or Apache License 2.0.

You may choose either license for your own use.

- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-APACHE)
