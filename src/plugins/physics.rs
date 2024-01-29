use bevy::{
    app::{App, Plugin, PreUpdate, Startup},
    ecs::system::{Commands, Res},
    math::Vec2,
    time::{Timer, TimerMode},
};
use bevy_xpbd_2d::{plugins::PhysicsPlugins, resources::Gravity};

use crate::{models::Constants, LastJumpTime};

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
