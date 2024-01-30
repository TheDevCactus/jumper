mod components_resources;
mod models;
mod plugins;
mod scenes;
mod service;

use bevy::{ecs::schedule::ScheduleLabel, prelude::*};
use plugins::animation_manager::AnimationManager;
use plugins::config_loader::ConfigLoader;
use plugins::delete_manager::DeleteManager;

use scenes::{home::HomeScene, level::LevelScene, map::MapScene, Scene};
pub struct Game;
impl Plugin for Game {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            DefaultPlugins,
            ConfigLoader {
                pre_startup: PreStartup.intern(),
            },
            AnimationManager,
            DeleteManager,
        ))
        .add_state::<Scene>()
        .add_plugins(HomeScene)
        .add_plugins(MapScene)
        .add_plugins(LevelScene);
    }
}

fn main() {
    App::new().add_plugins(Game).run();
}
