# Credits and Prior Art

RustMU* is an original project. This document acknowledges the public projects, frameworks, and communities that informed its early research and design direction.

No claim is made that RustMU* copies, derives from, or is affiliated with any project listed below. Where an idea is borrowed, it is credited as prior art or inspiration; RustMU*’s implementation, architecture, content model, rules model, and product direction remain independent.

## Major inspiration: Evennia

[Evennia](https://www.evennia.com/) is one of the main inspirations for RustMU*.

RustMU* draws conceptual inspiration from Evennia’s separation of reusable engine infrastructure from game-specific code. In particular, RustMU* follows a similar high-level idea: a Portal layer handles external connections and protocols, while a Server layer handles game state, commands, simulation, and persistence; game-specific code remains separate from the engine.

RustMU* is not a port, fork, or implementation of Evennia. It is an independent Rust-native framework with its own SDK-first architecture, crate boundaries, persistence model, development roadmap, and product positioning.

## Researched projects

| Project | Author / Organization | Repository | Relevance to RustMU* |
|---|---|---|---|
| RustMUD | raretypeoffox | https://github.com/raretypeoffox/RustMUD | A Rust multi-user dungeon supporting Diku/Merc `.are` files and GMCP. Its work is acknowledged as prior art for legacy-content compatibility and client protocol considerations. |
| rustmu | Geal | https://github.com/Geal/rustmu | A MU* server written in Rust. Acknowledged as prior art for Rust-based MU* server design and the general technical problem space. |
| rustmud | lijingmt | https://github.com/lijingmt/rustmud | A Rust MUD project. Acknowledged as part of the researched ecosystem of Rust MUD implementations. |
| rustmud | srwmay08 | https://github.com/srwmay08/rustmud | A Rust MUD project. Acknowledged as part of the researched ecosystem of Rust MUD implementations. |
| rustmush | conclavio | https://github.com/conclavio/rustmush | A modern TinyMUSH/MUX-like Rust game server with LLM support for world creation, NPCs, storytelling, rules adjudication, combat, economy, astronomy, and pluggable rules systems. Its public description was especially valuable for understanding the overlap between MUSH-style servers, AI-assisted authoring, and modular rules. |
| RustMultiplayer | Pyrograf | https://github.com/Pyrograf/RustMultiplayer | A server-and-client multiplayer project using Tokio and WGPU. Acknowledged as a useful reference for Rust-native realtime networking and client architecture concepts. |
| Rustmud | areadox | https://github.com/areadox/Rustmud | A Rust MUD project. Acknowledged as part of the researched ecosystem of Rust MUD implementations. |
| RustMud | valvecore | https://github.com/valvecore/RustMud | A Rust learning project. Acknowledged as part of the researched ecosystem of Rust MUD implementations. |
| rustmud | shmup | https://github.com/shmup/rustmud | A MUD written in Rust. Acknowledged as part of the researched ecosystem of Rust MUD implementations. |
| RustMud | segfahlt | https://github.com/segfahlt/RustMud | A simple Rust-based MUD described as coded from scratch with agentic support. Acknowledged as prior art for interest in agent-assisted MUD development. |
| RustMUD-Bigworld | Fiztba | https://github.com/Fiztba/RustMUD-Bigworld | A RustMUD variant using 32-bit vnums to support substantially larger worlds. Acknowledged as prior art for world-scale and identifier-capacity considerations. |

## How these projects shaped RustMU*

The research reinforced several planning principles for RustMU*:

- RustMU* should be a persistent, simulation-driven multiplayer world platform rather than another conventional Diku-style MUD clone.
- The project should separate client protocols, sessions, world simulation, persistence, content, and rules into clear modules.
- Compatibility with Diku/Merc `.are` files and GMCP may be useful as optional interoperability features, but they should not define RustMU*’s core identity.
- AI can assist world-building, narrative generation, NPC dialogue, and content workflows, while deterministic rules and an auditable simulation should remain the source of truth for gameplay outcomes.
- World content—including rooms, exits, objects, factions, quests, cultures, history, and rules—should be treated as versionable data rather than hard-coded game logic.
- Scalability, persistence, authoring workflows, and client neutrality should be architectural concerns from the beginning.

## Thanks

Thank you to the Evennia project and community for demonstrating the value of separating a reusable game engine from game-specific code.

Thank you to every author and contributor whose public work made this research possible. Open-source projects are valuable not only as code to reuse, but as records of design decisions, trade-offs, successes, and limitations.

RustMU* is grateful for the broader Rust, MUD, MU*, multiplayer-game, and open-source communities.
