use bevy::{
    app::{App, Plugin, Update},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        schedule::{common_conditions::in_state, IntoSystemConfigs, OnEnter, OnExit},
        system::Commands,
    },
    utils::default,
};

use super::Scene;

fn setup_home() {}
fn update_home(mut commands: Commands) {
    commands.spawn(Camera2dBundle { ..default() });
}
fn exit_home() {}

pub struct MapScene;
impl Plugin for MapScene {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Scene::Map), setup_home);
        app.add_systems(Update, (update_home).run_if(in_state(Scene::Map)));
        app.add_systems(OnExit(Scene::Map), exit_home);
    }
}
