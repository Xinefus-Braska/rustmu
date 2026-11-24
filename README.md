# RustMU*

RustMU* is a Rust-based, database-backed framework for building MU* games—MUDs, MUSHes, MUXes, MUCKs, and MOO-style worlds.

It separates the networking layer from game logic so that a game can be implemented in its own crate without modifying the engine.

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
| `rustmu-server` | Accounts, world state, commands, permissions, simulation, persistence, and future combat |
| `rustmu-sdk` | Protocol-neutral session and message types shared by Portal and Server |
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
│   └── rust-mu-engine-plan.md
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
- A documented master plan and architecture decision records.

## Roadmap

1. **Phase 0 — Foundations:** Workspace structure, database schema, migrations, configuration, logging, CI, and a runnable empty server.
2. **Phase 1 — Connections and sessions:** Telnet listener, multiple concurrent sessions, UTF-8 handling, rate limits, and integration tests.
3. **Phase 2 — Accounts and persistence:** Account creation, secure authentication, reconnect handling, and database-backed identities.
4. **Phase 3 — Minimal world model:** Objects, rooms, exits, attributes, tags, movement, and persistence.
5. **Phase 4 — Commands and permissions:** Command registry, parser, roles, permissions, and help.
6. **Phase 5 — Builder experience:** Content creation, prototypes, area import/export, and validation.
7. **Phase 6 — Scripting and extensibility:** Event hooks, Rust game extensions, and optional scripting.
8. **Phase 7 — Combat-ready simulation:** Tick scheduler, active-world cache, combat skeleton, and performance instrumentation.
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
- **Optional scripting:** Start with Rust game crates; consider Rhai, Lua, or WASM after the core is stable.

## Documentation

- [Rust MU* Engine Master Plan](docs/rust-mu-engine-plan.md)
- [Architecture decision records](docs/decisions/)

## License

RustMU* is dual-licensed under either the MIT License or Apache License 2.0.

You may choose either license for your own use.

- [MIT License](LICENSE-MIT)
- [Apache License 2.0](LICENSE-APACHE)
