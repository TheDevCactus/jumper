use bevy::{
    app::{App, Plugin, PreUpdate, Startup},
    ecs::system::{Commands, Res},
    math::Vec2,
    time::{Timer, TimerMode},
};
use bevy_xpbd_2d::{plugins::PhysicsPlugins, prelude::PhysicsLayer, resources::Gravity};

use crate::{components_resources::LastJumpTime, models::Constants};

#[derive(PhysicsLayer)]
pub enum Layers {
    Checkpoint,
    Player,
    Enemy,
    Ground,
}

fn initialize_physics(mut commands: Commands, constants: Res<Constants>) {
    commands.insert_resource(LastJumpTime(Timer::from_seconds(
        constants.initial_jump_time,
        TimerMode::Once,
    )));
    commands.insert_resource(Gravity(Vec2::NEG_Y * constants.gravity));
}

pub struct PhysicsManager;
impl Plugin for PhysicsManager {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::new(PreUpdate));
        app.add_systems(Startup, initialize_physics);
    }
}
