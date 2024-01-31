use bevy::{
    app::{App, Plugin, Update},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        schedule::{common_conditions::in_state, IntoSystemConfigs, NextState, OnEnter, OnExit},
        system::{Commands, Res, ResMut},
    },
    input::{keyboard::KeyCode, Input},
    utils::default,
};

use crate::scenes::level::LevelID;

use super::Scene;

fn setup_map() {
    println!("setup_map");
}
fn update_home(
    mut commands: Commands,
    keyboard_input: Res<Input<KeyCode>>,
    mut scene_state: ResMut<NextState<Scene>>,
) {
    let mut should_nav = false;
    // if keyboard input number 1 is pressed
    // set the resource LevelID to "plains_1"
    if keyboard_input.just_pressed(KeyCode::Key1) {
        println!("LevelID set to plains_1");
        commands.insert_resource(LevelID("untitled_old.tmx".to_string()));
        should_nav = true;
    }
    // if keyboard input number 2 is pressed
    // set the resource LevelID to "plains_2"
    if keyboard_input.just_pressed(KeyCode::Key2) {
        println!("LevelID set to plains_2");
        commands.insert_resource(LevelID("plains_2.tmx".to_string()));
        should_nav = true;
    }
    // if keyboard input number 3 is pressed
    // set the resource LevelID to "plains_3"
    if keyboard_input.just_pressed(KeyCode::Key3) {
        println!("LevelID set to plains_3");
        commands.insert_resource(LevelID("jumping.tmx".to_string()));
        should_nav = true;
    }
    // if keyboard input number 4 is pressed
    // set the resource LevelID to "plains_4"
    if keyboard_input.just_pressed(KeyCode::Key4) {
        println!("LevelID set to plains_4");
        commands.insert_resource(LevelID("long.tmx".to_string()));
        should_nav = true;
    }
    // if keyboard input number 5 is pressed
    // set the resource LevelID to "plains_5"
    if keyboard_input.just_pressed(KeyCode::Key5) {
        println!("LevelID set to plains_5");
        commands.insert_resource(LevelID("plains_5".to_string()));
        should_nav = true;
    }
    if should_nav {
        scene_state.set(Scene::Level);
    }
}
fn exit_home() {}

pub struct MapScene;
impl Plugin for MapScene {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Scene::Map), setup_map);
        app.add_systems(Update, (update_home).run_if(in_state(Scene::Map)));
        app.add_systems(OnExit(Scene::Map), exit_home);
    }
}
