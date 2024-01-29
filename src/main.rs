mod components_resources;
mod models;
mod plugins;

use bevy::prelude::*;
use plugins::animation_manager::AnimationManager;
use plugins::camera_controls::CameraControls;
use plugins::config_loader::ConfigLoader;
use plugins::delete_manager::DeleteManager;
use plugins::level_loader::LevelLoader;
use plugins::physics::PhysicsManager;
use plugins::player_manager::PlayerManager;
use plugins::trick_manager::TrickManager;

pub struct Game;
impl Plugin for Game {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultPlugins,
            ConfigLoader,
            CameraControls,
            LevelLoader,
            PhysicsManager,
            AnimationManager,
            DeleteManager,
            TrickManager,
            PlayerManager,
        ));
    }
}

fn main() {
    App::new().add_plugins(Game).run();
}
