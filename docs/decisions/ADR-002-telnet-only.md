# ADR-002: Start with Telnet only

## Status

Accepted

## Context

The framework needs a first connection protocol. Traditional MU* clients use
Telnet over TCP. WebSocket, SSH, and HTTP-based clients are useful but add
complexity before the core game server exists.

## Decision

Phase 1 will implement Telnet over TCP only.

The Portal will own:

- TCP listener and connection lifecycle
- Telnet byte-stream decoding
- UTF-8 line assembly
- Basic Telnet negotiation handling
- Session input and output queues
- Rate limiting and connection limits

The Server will receive normalized session events, not raw sockets.

WebSocket, SSH, and HTTP services are deferred. Their future addition must not
require changes to game code.

## Consequences

- The first playable server will work with standard MUD clients.
- The Portal/Server boundary must be protocol-agnostic from the beginning.
- Rich-client features such as GMCP, MSDP, MXP, maps, and health bars are deferred.
- A WebSocket transport can be added later behind the same session interface.