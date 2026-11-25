//! In-memory implementation of the RustMU server world facade.
//!
//! This module provides the first concrete world implementation. It stores
//! characters, rooms, exits, and room occupancy in memory. It is intended
//! for the initial command-dispatcher slice and local testing; persistent
//! storage, distributed state, and advanced simulation are deferred.

use std::collections::HashMap;
use std::sync::Mutex;

use rustmu_sdk::communication::{Speech, SpeechResult};
use rustmu_sdk::movement::{Direction, MovementResult};
use rustmu_sdk::world::World;
use rustmu_sdk::{Character, CharacterId};

use crate::world::WorldFacade;

/// Identifies a room within the in-memory world.
///
/// Room identifiers are stable for the lifetime of a room record. The
/// server owns allocation and uniqueness; game code treats them as opaque
/// values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RoomId(pub u64);

/// An in-memory room and its connections to other rooms.
#[derive(Debug, Clone)]
struct Room {
    /// Human-readable room name.
    name: String,

    /// Human-readable room description.
    description: String,

    /// Available exits, keyed by canonical movement direction.
    exits: HashMap<Direction, RoomId>,

    /// Characters currently located in the room.
    occupants: Vec<CharacterId>,
}

impl Room {
    /// Creates an empty room with the supplied identity and description.
    fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            exits: HashMap::new(),
            occupants: Vec::new(),
        }
    }

    /// Connects this room to another room in a canonical direction.
    fn connect(&mut self, direction: Direction, destination: RoomId) {
        self.exits.insert(direction, destination);
    }

    /// Adds a character to the room if it is not already present.
    fn add_occupant(&mut self, character_id: CharacterId) {
        if !self.occupants.contains(&character_id) {
            self.occupants.push(character_id);
        }
    }

    /// Removes a character from the room if it is present.
    fn remove_occupant(&mut self, character_id: CharacterId) {
        self.occupants.retain(|occupant| *occupant != character_id);
    }
}

/// Mutable state owned by an [`InMemoryWorld`].
#[derive(Debug, Default)]
struct InMemoryWorldState {
    /// Characters known to the world, keyed by character identifier.
    characters: HashMap<CharacterId, Character>,

    /// Rooms known to the world, keyed by room identifier.
    rooms: HashMap<RoomId, Room>,

    /// Current room for each character, keyed by character identifier.
    locations: HashMap<CharacterId, RoomId>,
}

impl InMemoryWorldState {
    /// Returns the room occupied by a character, if any.
    fn current_room(&self, character_id: CharacterId) -> Option<&Room> {
        self.locations
            .get(&character_id)
            .and_then(|room_id| self.rooms.get(room_id))
    }

    /// Builds a human-readable description of a character's current room.
    fn room_description(&self, character_id: CharacterId) -> Option<String> {
        let room = self.current_room(character_id)?;

        // Start with the room identity and description, then append a
        // compact exit summary when exits are available.
        let mut description = format!("{}\n{}", room.name, room.description);

        if !room.exits.is_empty() {
            let mut directions: Vec<&str> = room
                .exits
                .keys()
                .map(|direction| direction.as_str())
                .collect();
            directions.sort_unstable();

            description.push_str("\n\nExits: ");
            description.push_str(&directions.join(", "));
        }

        Some(description)
    }

    /// Returns the other characters in the same room as the supplied
    /// character.
    fn occupants(&self, character_id: CharacterId) -> Vec<Character> {
        let Some(room) = self.current_room(character_id) else {
            return Vec::new();
        };

        room.occupants
            .iter()
            .copied()
            .filter(|occupant| *occupant != character_id)
            .filter_map(|occupant| self.characters.get(&occupant).cloned())
            .collect()
    }

    /// Moves a character through an exit, updating both rooms atomically.
    fn move_character(
        &mut self,
        character_id: CharacterId,
        direction: Direction,
    ) -> MovementResult {
        let Some(current_room_id) = self.locations.get(&character_id).copied() else {
            return MovementResult::Blocked(String::from("You are not in a room."));
        };

        let Some(destination_room_id) = self
            .rooms
            .get(&current_room_id)
            .and_then(|room| room.exits.get(&direction))
            .copied()
        else {
            return MovementResult::Blocked(String::from(
                "You cannot move in that direction.",
            ));
        };

        // Remove the character from the source room before inserting it in
        // the destination room. These updates are intentionally kept together
        // so a caller cannot observe a partially completed movement.
        if let Some(source_room) = self.rooms.get_mut(&current_room_id) {
            source_room.remove_occupant(character_id);
        }

        if let Some(destination_room) = self.rooms.get_mut(&destination_room_id) {
            destination_room.add_occupant(character_id);
        }

        self.locations.insert(character_id, destination_room_id);

        let destination_name = self
            .rooms
            .get(&destination_room_id)
            .map(|room| room.name.clone())
            .unwrap_or_default();

        MovementResult::Moved(format!("You arrive in {destination_name}."))
    }

    /// Accepts a room-scoped speech request from a character.
    fn say(&self, speech: Speech) -> SpeechResult {
        let Some(_room) = self.current_room(speech.speaker_id()) else {
            return SpeechResult::Rejected(String::from("You are not in a room."));
        };

        // The initial implementation accepts all speech from a located
        // character. Audience selection, moderation, and delivery remain
        // server responsibilities.
        SpeechResult::Accepted(format!("You say, \"{}\"", speech.text()))
    }
}

/// In-memory implementation of the server world facade.
///
/// This type is intended for the initial server slice. It provides
/// deterministic behavior for local tests and can later be replaced by a
/// persistent or distributed world implementation without changing the SDK
/// command contracts.
#[derive(Debug, Default)]
pub struct InMemoryWorld {
    /// Mutable world state, serialized by a mutex.
    state: Mutex<InMemoryWorldState>,
}

impl InMemoryWorld {
    /// Creates an empty in-memory world.
    pub fn new() -> Self {
        Self {
            state: Mutex::new(InMemoryWorldState::default()),
        }
    }

    /// Inserts or replaces a character record.
    ///
    /// This is a server setup operation, not a player-facing command. It does
    /// not place the character in a room; use [`InMemoryWorld::place_character`]
    /// for that purpose.
    pub fn insert_character(&mut self, character: Character) {
        self.state
            .lock()
            .expect("in-memory world state must not be poisoned")
            .characters
            .insert(character.id, character);
    }

    /// Inserts or replaces a room record.
    ///
    /// This is a server setup operation, not a player-facing command.
    pub fn insert_room(
        &mut self,
        id: RoomId,
        name: impl Into<String>,
        description: impl Into<String>,
    ) {
        self.state
            .lock()
            .expect("in-memory world state must not be poisoned")
            .rooms
            .insert(id, Room::new(name, description));
    }

    /// Connects two rooms with a one-way exit.
    ///
    /// Call this method twice with opposing directions to create a two-way
    /// connection.
    pub fn connect_rooms(&mut self, source: RoomId, direction: Direction, destination: RoomId) {
        if let Some(room) = self
            .state
            .lock()
            .expect("in-memory world state must not be poisoned")
            .rooms
            .get_mut(&source)
        {
            room.connect(direction, destination);
        }
    }

    /// Places a character in a room.
    ///
    /// This is a server setup operation, not a player-facing command. The
    /// character must already be registered with [`InMemoryWorld::insert_character`].
    pub fn place_character(&mut self, character_id: CharacterId, room_id: RoomId) {
        let mut state = self
            .state
            .lock()
            .expect("in-memory world state must not be poisoned");

        if let Some(room) = state.rooms.get_mut(&room_id) {
            room.add_occupant(character_id);
        }

        state.locations.insert(character_id, room_id);
    }
}

impl World for InMemoryWorld {
    fn room_description(&self, character_id: CharacterId) -> Option<String> {
        self.state
            .lock()
            .expect("in-memory world state must not be poisoned")
            .room_description(character_id)
    }

    fn occupants(&self, character_id: CharacterId) -> Vec<Character> {
        self.state
            .lock()
            .expect("in-memory world state must not be poisoned")
            .occupants(character_id)
    }

    fn move_character(&self, character_id: CharacterId, direction: Direction) -> MovementResult {
        self.state
            .lock()
            .expect("in-memory world state must not be poisoned")
            .move_character(character_id, direction)
    }

    fn say(&self, speech: Speech) -> SpeechResult {
        self.state
            .lock()
            .expect("in-memory world state must not be poisoned")
            .say(speech)
    }
}

impl WorldFacade for InMemoryWorld {
    fn character(&self, character_id: CharacterId) -> Option<Character> {
        self.state
            .lock()
            .expect("in-memory world state must not be poisoned")
            .characters
            .get(&character_id)
            .cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustmu_sdk::communication::Speech;
    use rustmu_sdk::movement::Direction;

    /// Creates a world with two connected rooms and two characters.
    fn test_world() -> InMemoryWorld {
        let mut world = InMemoryWorld::new();

        world.insert_character(Character {
            id: CharacterId(1),
            name: String::from("Alice"),
        });

        world.insert_character(Character {
            id: CharacterId(2),
            name: String::from("Bob"),
        });

        world.insert_room(
            RoomId(1),
            "Entry Hall",
            "A small hall with a doorway to the east.",
        );

        world.insert_room(
            RoomId(2),
            "Library",
            "Shelves line the walls of a quiet library.",
        );

        world.connect_rooms(RoomId(1), Direction::East, RoomId(2));
        world.connect_rooms(RoomId(2), Direction::West, RoomId(1));

        world.place_character(CharacterId(1), RoomId(1));
        world.place_character(CharacterId(2), RoomId(1));

        world
    }

    #[test]
    fn room_description_includes_name_and_exits() {
        let world = test_world();

        let description = world
            .room_description(CharacterId(1))
            .expect("character should have a room");

        assert!(description.contains("Entry Hall"));
        assert!(description.contains("Exits: east"));
    }

    #[test]
    fn occupants_exclude_the_acting_character() {
        let world = test_world();

        let occupants = world.occupants(CharacterId(1));

        assert_eq!(occupants.len(), 1);
        assert_eq!(occupants[0].id, CharacterId(2));
        assert_eq!(occupants[0].name, "Bob");
    }

    #[test]
    fn movement_updates_the_character_location() {
        let world = test_world();

        let result = world.move_character(CharacterId(1), Direction::East);

        assert!(matches!(result, MovementResult::Moved(_)));

        let description = world
            .room_description(CharacterId(1))
            .expect("moved character should have a room");

        assert!(description.contains("Library"));
        assert!(description.contains("Exits: west"));
    }

    #[test]
    fn movement_updates_room_occupancy() {
        let world = test_world();

        let result = world.move_character(CharacterId(1), Direction::East);

        assert!(matches!(result, MovementResult::Moved(_)));

        let source_occupants = world.occupants(CharacterId(2));
        let destination_occupants = world.occupants(CharacterId(1));

        assert!(source_occupants.is_empty());
        assert_eq!(destination_occupants.len(), 1);
        assert_eq!(destination_occupants[0].id, CharacterId(2));
    }

    #[test]
    fn movement_without_an_exit_is_blocked() {
        let world = test_world();

        let result = world.move_character(CharacterId(1), Direction::North);

        assert!(matches!(result, MovementResult::Blocked(_)));

        let description = world
            .room_description(CharacterId(1))
            .expect("blocked character should remain in a room");

        assert!(description.contains("Entry Hall"));
    }

    #[test]
    fn speech_is_accepted_for_a_character_in_a_room() {
        let world = test_world();

        let result = world.say(Speech::new(CharacterId(1), "Hello."));

        assert!(matches!(result, SpeechResult::Accepted(_)));
    }
}
