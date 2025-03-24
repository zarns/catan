use log::info;

pub mod deck_slices;
pub mod decks;
pub mod enums;
pub mod game;
pub mod global_state;
pub mod map_instance;
pub mod map_template;
mod ordered_hashmap;
pub mod players;
pub mod state;
pub mod state_vector;

// Initialize logger for the library
pub fn init_logger() {
    env_logger::init();
    info!("Initialized catanatron_rust logging");
}

// Export public types
pub use game::Game;
pub use players::{Player, RandomPlayer};
pub use enums::Action;
pub use state::State;
