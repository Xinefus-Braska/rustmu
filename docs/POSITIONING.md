# RustMU* Positioning

## Positioning statement

**RustMU* is a Rust-native MU* framework for building persistent, customizable multiplayer worlds. It provides a stable game-facing SDK, a modular server core, and a client portal, so creators can define their own worlds, characters, commands, systems, and rules without rebuilding the underlying engine.**

### Short form

**RustMU*: A Rust-native framework for building persistent, customizable MU* worlds.**

## What RustMU* is

RustMU* is a foundation for building text-based persistent multiplayer worlds. It is intended to support MUDs, MUSHes, MUXes, roleplay-focused communities, and other custom MU* experiences.

The project separates reusable engine infrastructure from game-specific design:

- `rustmu-sdk` defines the stable game-facing contracts and extension points.
- `rustmu-server` provides the core engine behavior.
- `rustmu-portal` handles the network boundary and client sessions.
- `rustmu-cli` provides a thin launcher and operational entry point.
- Game-specific code lives in the game layer and defines the world’s rules, content, systems, and experience.

The SDK describes what game code may do; the server decides how the engine does it.

## Core differentiators

RustMU* is differentiated by the following commitments:

- **Rust-native:** Built with modern Rust, emphasizing safety, maintainability, and a clear asynchronous architecture.
- **MU* framework, not a fixed game:** RustMU* provides the foundation; each hosted world defines its own identity and rules.
- **Persistent worlds:** Characters, world state, and authored content are designed to persist across sessions.
- **SDK-first architecture:** The stable SDK boundary protects game code from unnecessary churn in the evolving engine.
- **Modular design:** Protocol handling, sessions, simulation, persistence, content, rules, and tooling remain separable.
- **Creator control:** World builders retain ownership of genre, progression, combat, economy, roleplay norms, narrative, and community policy.
- **Telnet-first:** The initial client experience is text-based and accessible, while the portal architecture can support additional protocols later.

## Primary audience

The primary audience is the **MU* world builder who is comfortable with Rust or working alongside a Rust developer**.

This creator wants to build a distinctive persistent multiplayer world without maintaining an entire server stack from scratch. They need reusable infrastructure, but they do not want the framework to dictate their world’s genre, rules, culture, progression model, or player experience.

> **RustMU* is for creators who want to build their own persistent MU* world in Rust without giving up control over its rules, content, culture, or player experience.**

## Secondary audiences

| Audience | Need | RustMU* value |
|---|---|---|
| Rust developers | A safe, clear, maintainable foundation for text multiplayer systems | A structured workspace with explicit SDK, server, portal, and CLI boundaries |
| Server operators | Reliable startup, configuration, persistence, and operational control | A server core and CLI designed around a maintainable deployment boundary |
| Players | A responsive, persistent, accessible text multiplayer experience | A Telnet-first portal and a world experience defined by the host game |
| Content contributors | A safe way to add rooms, characters, objects, stories, and systems | A future content-and-extension model that separates authoring from engine internals |
| Game designers | Extensible systems without a forced genre or ruleset | Capability contracts and extension points rather than hard-coded game policy |

## What RustMU* is not

RustMU* deliberately avoids the following assumptions:

- It is **not a finished game** with one canonical world, campaign, or ruleset.
- It is **not a Diku/Merc port** or a Rust rewrite of a specific legacy MUD codebase.
- It is **not a rigid rules engine** that determines every world’s combat, economy, magic, progression, or roleplay policy.
- It is **not an AI game by default**; AI may later assist authoring, narration, or NPC behavior, but deterministic rules and auditable simulation remain authoritative.
- It is **not a graphical game engine**; the first experience is text-first, with graphical clients as a possible later extension.
- It is **not a monolith**; engine infrastructure, protocol handling, game rules, and world content remain separable.

## Relationship to prior art

RustMU* is informed by research into existing Rust MU* and multiplayer projects, but it is an original framework with its own architecture and product direction.

In particular:

- Diku/Merc `.are` compatibility and GMCP may be useful optional interoperability features, but they do not define RustMU*.
- AI-assisted world-building may become a future capability, but it must not replace deterministic rules or an auditable simulation.
- World content and rules should be versionable data and modular extensions, not hard-coded engine behavior.
- Scalability, persistence, authoring workflows, and client neutrality are architectural concerns from the beginning.

See [`CREDITS.md`](../CREDITS.md) for the researched projects and prior-art acknowledgements.

## Feature evaluation guardrails

A proposed feature should advance RustMU*’s positioning if it:

1. Helps someone build a persistent, customizable MU* world.
2. Preserves the separation between engine, SDK contracts, portal, and game code.
3. Can be implemented without forcing every RustMU* world to adopt the same genre or rules.
4. Makes the SDK more stable, useful, or comprehensible.
5. Supports the first end-to-end milestone: Portal connection, command dispatch through the SDK, and a response to the player.

Features that do not meet these tests may still be valuable later, but they should not become part of RustMU*’s core identity or initial implementation.

## Integration with project planning

This document is the strategic reference for future RustMU* planning. It should guide:

- The engine architecture plan.
- The SDK contract and API design.
- The Portal protocol and session model.
- The CLI and operational workflow.
- Content, persistence, and world-authoring plans.
- Future decisions about AI, compatibility, extensibility, and client support.

When a planning decision conflicts with this positioning, revisit the decision before implementing it.
