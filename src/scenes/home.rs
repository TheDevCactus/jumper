use bevy::{
    app::{App, Plugin, Update},
    core_pipeline::core_2d::Camera2dBundle,
    ecs::{
        schedule::{
            common_conditions::in_state, IntoSystemConfigs, NextState, OnEnter, OnExit, State,
        },
        system::{Commands, Res, ResMut},
    },
    input::{
        keyboard::{KeyCode, KeyboardInput},
        Input,
    },
    math::Vec2,
    render::{color::Color, render_resource::Texture},
    sprite::{Sprite, SpriteBundle},
    transform::{commands, components::Transform},
    utils::default,
};

use super::Scene;

fn setup_home(mut commands: Commands) {
    commands.spawn(SpriteBundle {
        sprite: Sprite {
            color: Color::hex("FF0000").unwrap(),
            custom_size: Some(Vec2::new(100.0, 100.0)),
            ..Default::default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..Default::default()
    });
}
fn update_home(mut scene_state: ResMut<NextState<Scene>>, keyboard_input: Res<Input<KeyCode>>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        scene_state.set(Scene::Level);
    }
}

pub struct HomeScene;
impl Plugin for HomeScene {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Scene::Home), setup_home);
        app.add_systems(Update, update_home.run_if(in_state(Scene::Home)));
    }
}
