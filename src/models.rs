use bevy::{
    ecs::{component::Component, system::Resource},
    input::keyboard::KeyCode,
};
use serde::{Deserialize, Serialize};

use crate::scenes::Scene;

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
        let filtered = self
            .tricks
            .iter()
            .filter(|(trick_keys, trick_definition)| {
                if trick_keys.len() > keys.len() {
                    return true;
                }
                if trick_keys.len() != keys.len() {
                    return false;
                }
                for i in 0..trick_keys.len() {
                    if trick_keys[i] != keys[i] {
                        return false;
                    }
                }
                true
            })
            .map(|(_, trick_definition)| trick_definition);
        let filtered = filtered.collect::<Vec<&TrickDefinition>>();
        match filtered.len() {
            1 => {
                let trick = filtered[0];
                return Some(trick);
            }
            _ => {
                return None;
            }
        }
    }
}

// Resource used to track all possible tricks for the game
#[derive(Serialize, Deserialize, Resource, Debug, Clone)]
pub struct TrickListResource(pub TrickList);

#[derive(Component)]
pub struct BelongsToScene(pub Scene);
