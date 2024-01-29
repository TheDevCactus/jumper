use bevy::{ecs::system::Resource, input::keyboard::KeyCode};
use serde::{Deserialize, Serialize};

// Used as a resource to store "constant" variables in the game
// This is loaded from a file so that we can keep build times down
// for small tweaks to the game. Down the road, we should consider
// baking these values in for release builds somehow. That'd be neat.
#[derive(Serialize, Deserialize, Resource, Debug)]
pub struct Constants {
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
}

// Used to define a single trick
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrickDefinition {
    pub name: String,
    pub points: usize,
    pub takes_ms: usize,
}

// Represents all possible tricks for the game
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TrickList {
    pub tricks: Vec<(Vec<KeyCode>, TrickDefinition)>,
}
impl TrickList {
    pub fn find_trick(&mut self, keys: &Vec<KeyCode>) -> Option<&TrickDefinition> {
        self.tricks
            .iter()
            .find_map(|(trick_keys, trick_definition)| {
                if trick_keys.len() != keys.len() {
                    return None;
                }
                for i in 0..trick_keys.len() {
                    if trick_keys[i] != keys[i] {
                        return None;
                    }
                }
                Some(trick_definition)
            })
    }
}

// Resource used to track all possible tricks for the game
#[derive(Serialize, Deserialize, Resource, Debug, Clone)]
pub struct TrickListResource(pub TrickList);
