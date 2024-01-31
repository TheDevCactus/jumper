use bevy::ecs::system::Resource;
use serde::{Deserialize, Serialize};

// Used as a resource to store "constant" variables in the game
// This is loaded from a file so that we can keep build times down
// for small tweaks to the game. Down the road, we should consider
// baking these values in for release builds somehow. That'd be neat.
#[derive(Serialize, Deserialize, Resource, Debug)]
pub struct Constants {
    pub post_level_secs: u64,
    pub map_name: String,
    pub dash_force: f32,
    pub trick_time: f32,
    pub squish_bounce_force: f32,
    pub character_sheet: String,
    pub player_speed: f32,
    pub max_player_speed: f32,
    pub jump_force: f32,
    pub initial_jump_time: f32,
    pub gravity: f32,
    pub curve_pow: f32,
    pub grounded_decay: f32,
    pub grounded_threshold: f32,
    pub wall_threshold: f32,
    pub path_to_player_data: String,
}

impl Constants {
    pub fn read_from_file() -> Constants {
        let raw = std::fs::read_to_string("./assets/constants.toml").unwrap();
        toml::from_str::<Constants>(&raw).unwrap()
    }
}
