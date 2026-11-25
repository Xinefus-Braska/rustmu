//! Command dispatcher for the RustMU server.
//!
//! This module maps normalized SDK command names to registered command
//! handlers. It uses the SDK's command-handler and registration contracts,
//! while leaving world state, persistence, and transport delivery to the
//! server's world and session layers.

use std::collections::HashMap;

use rustmu_sdk::command::{
    Command, CommandContext, CommandHandler, CommandHandlerResult, CommandRegistrationResult,
    CommandRegistry, LookCommand,
};
use rustmu_sdk::communication::SayCommand;
use rustmu_sdk::movement::MoveCommand;
use rustmu_sdk::world::World;

/// Routes SDK commands to registered command handlers.
///
/// The dispatcher owns command-name normalization and handler lookup. It
/// does not interpret room connectivity, movement rules, speech policy, or
/// any other game-specific behavior.
#[derive(Default)]
pub struct CommandDispatcher {
    /// Handlers indexed by normalized command name.
    handlers: HashMap<String, Box<dyn CommandHandler>>,
}

impl CommandDispatcher {
    /// Creates an empty command dispatcher.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers the SDK's initial look, movement, and speech handlers.
    ///
    /// The canonical command names are intentionally minimal. Aliases,
    /// localized command names, and direction shortcuts can be added later
    /// through additional registrations.
    pub fn with_default_handlers() -> Self {
        let mut dispatcher = Self::new();

        // Register the SDK-provided handlers under their canonical names.
        let _ = dispatcher.register("look", Box::new(LookCommand::new()));
        let _ = dispatcher.register("move", Box::new(MoveCommand::new()));
        let _ = dispatcher.register("say", Box::new(SayCommand::new()));

        dispatcher
    }

    /// Returns the normalized form of a command name.
    ///
    /// Command names are trimmed and lowercased before lookup. This keeps
    /// dispatch deterministic without changing the SDK command contract.
    fn normalize(name: &str) -> String {
        name.trim().to_ascii_lowercase()
    }
}

impl CommandRegistry for CommandDispatcher {
    /// Registers a handler for the supplied command name.
    ///
    /// A blank command name or duplicate registration is rejected. The
    /// dispatcher does not replace an existing handler because replacement
    /// policy should be explicit rather than accidental.
    fn register(
        &mut self,
        name: &str,
        handler: Box<dyn CommandHandler>,
    ) -> CommandRegistrationResult {
        let normalized_name = Self::normalize(name);

        if normalized_name.is_empty() {
            return CommandRegistrationResult::Error(String::from(
                "A command name is required.",
            ));
        }

        if self.handlers.contains_key(&normalized_name) {
            return CommandRegistrationResult::Error(format!(
                "The command '{normalized_name}' is already registered."
            ));
        }

        self.handlers.insert(normalized_name, handler);

        CommandRegistrationResult::Ok
    }
}

impl CommandDispatcher {
    /// Dispatches a structured SDK command to its registered handler.
    ///
    /// # Parameters
    ///
    /// - `command`: The parsed command intent.
    /// - `context`: The acting character and associated session information.
    /// - `world`: Read- and request-oriented access to the game world.
    ///
    /// # Returns
    ///
    /// The handler's game-facing result, or an error when no handler is
    /// registered for the command name.
    pub fn dispatch(
        &self,
        command: &Command,
        context: &CommandContext,
        world: &dyn World,
    ) -> CommandHandlerResult {
        let normalized_name = Self::normalize(&command.name);

        let Some(handler) = self.handlers.get(&normalized_name) else {
            return CommandHandlerResult::Error(format!(
                "Unknown command: {}",
                command.name
            ));
        };

        handler.handle(command, context, world)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustmu_sdk::{Character, CharacterId, CommandHandlerResult};

    use crate::world::{InMemoryWorld, RoomId};

    /// Creates a dispatcher with the default SDK command handlers.
    fn test_dispatcher() -> CommandDispatcher {
        CommandDispatcher::with_default_handlers()
    }

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

        world.connect_rooms(RoomId(1), rustmu_sdk::Direction::East, RoomId(2));
        world.connect_rooms(RoomId(2), rustmu_sdk::Direction::West, RoomId(1));

        world.place_character(CharacterId(1), RoomId(1));
        world.place_character(CharacterId(2), RoomId(1));

        world
    }

    /// Creates a command context for Alice.
    fn test_context(world: &InMemoryWorld) -> CommandContext {
        let character = world
            .character(CharacterId(1))
            .expect("Alice should be registered");

        CommandContext::new(character, None)
    }

    #[test]
    fn dispatches_look_to_the_sdk_handler() {
        let dispatcher = test_dispatcher();
        let world = test_world();
        let context = test_context(&world);
        let command = Command {
            name: String::from("LOOK"),
            arguments: Vec::new(),
        };

        let result = dispatcher.dispatch(&command, &context, &world);

        let CommandHandlerResult::Ok(output) = result else {
            panic!("look should dispatch successfully");
        };

        assert!(output.text.contains("Entry Hall"));
        assert!(output.text.contains("Bob"));
    }

    #[test]
    fn dispatches_movement_to_the_sdk_handler() {
        let dispatcher = test_dispatcher();
        let world = test_world();
        let context = test_context(&world);
        let command = Command {
            name: String::from("move"),
            arguments: vec![String::from("east")],
        };

        let result = dispatcher.dispatch(&command, &context, &world);

        let CommandHandlerResult::Ok(output) = result else {
            panic!("movement should dispatch successfully");
        };

        assert_eq!(output.text, "You arrive in Library.");
    }

    #[test]
    fn dispatches_speech_to_the_sdk_handler() {
        let dispatcher = test_dispatcher();
        let world = test_world();
        let context = test_context(&world);
        let command = Command {
            name: String::from("say"),
            arguments: vec![String::from("Hello,"), String::from("Bob!")],
        };

        let result = dispatcher.dispatch(&command, &context, &world);

        let CommandHandlerResult::Ok(output) = result else {
            panic!("speech should dispatch successfully");
        };

        assert_eq!(output.text, "You say, \"Hello, Bob!\"");
    }

    #[test]
    fn rejects_an_unknown_command() {
        let dispatcher = test_dispatcher();
        let world = test_world();
        let context = test_context(&world);
        let command = Command {
            name: String::from("dance"),
            arguments: Vec::new(),
        };

        let result = dispatcher.dispatch(&command, &context, &world);

        let CommandHandlerResult::Error(message) = result else {
            panic!("an unknown command should be rejected");
        };

        assert_eq!(message, "Unknown command: dance");
    }

    #[test]
    fn rejects_duplicate_registration() {
        let mut dispatcher = CommandDispatcher::new();

        let first = dispatcher.register("look", Box::new(LookCommand::new()));
        let second = dispatcher.register("look", Box::new(LookCommand::new()));

        assert!(matches!(first, CommandRegistrationResult::Ok));
        assert!(matches!(second, CommandRegistrationResult::Error(_)));
    }
}
