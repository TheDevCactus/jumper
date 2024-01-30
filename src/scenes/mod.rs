use bevy::{app::Plugin, ecs::schedule::States};
use serde::{Deserialize, Serialize};

use self::{home::HomeScene, level::LevelScene, map::MapScene};

pub mod home;
pub mod level;
pub mod map;

#[derive(States, Default, Clone, Copy, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
pub enum Scene {
    Map,
    #[default]
    Home,
    Level,
}

pub struct SceneManager;
impl Plugin for SceneManager {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_state::<Scene>();
        app.add_plugins(HomeScene);
        app.add_plugins(LevelScene);
        app.add_plugins(MapScene);
    }
}

// trait SceneManager<M> {
//     fn on_startup(y: impl IntoSystemConfigs<M>) {}
//     fn on_update(y: impl IntoSystemConfigs<M>) {}
// fn on_exit(y: impl IntoSystemConfigs<M>) {}
// }

// the below is interesting and could lead to a better scene management solution
// fn add_plugin<M>(app: &mut App, y: impl IntoSystemConfigs<M>) {
//     app.add_systems(Update, y);
// }
