mod models;
mod plugins;

use bevy::prelude::*;

use bevy_xpbd_2d::prelude::*;

use plugins::animation_manager::AnimationManager;
use plugins::camera_controls::CameraControls;
use plugins::config_loader::ConfigLoader;
use plugins::delete_manager::{DeleteManager};
use plugins::level_loader::LevelLoader;
use plugins::physics::PhysicsManager;
use plugins::player_manager::PlayerManager;
use plugins::trick_manager::TrickManager;
use serde::{Deserialize, Serialize};

#[derive(Component)]
struct Score(usize);

#[derive(Component)]
struct CheckpointCheck;

#[derive(Component)]
struct LastKeyPressed((KeyCode, usize));

#[derive(Serialize, Deserialize, Component, Copy, Clone, Debug)]
struct Point {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct Player;

#[derive(PhysicsLayer)]
enum Layers {
    Checkpoint,
    Player,
    Enemy,
    Ground,
}

#[derive(Component)]
struct Platform;

#[derive(Component)]
struct Collision;

#[derive(Component)]
struct Enemy;

#[derive(Component)]
struct BottomOfPlayerRayCast;

#[derive(Component)]
struct ObjectComponent(Object);

#[derive(Component)]
struct GroundedCheck;

#[derive(Component)]
struct SquishCheck;

#[derive(Serialize, Deserialize, Component, Clone)]
struct Size {
    width: f32,
    height: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct Path {
    end: Point,
    speed: f32,
    looped: bool,
}

#[derive(Serialize, Deserialize, Clone)]
struct Object {
    position: Point,
    size: Size,
    color: String,
}

#[derive(Serialize, Deserialize)]
struct OldMap {
    enemies: Vec<Object>,
    platforms: Vec<Object>,
}

#[derive(Resource)]
struct TiledMap(tiled::Map);
#[derive(Resource)]
struct TiledTileset(tiled::Tileset);

#[derive(Component, Resource)]
struct TextureAtlasHandle(Handle<TextureAtlas>);

#[derive(Component)]
struct Tileset(tiled::Tileset);

#[derive(Component)]
struct TilesetName(String);

enum Checkpoint {
    End,
}

#[derive(Component)]
struct CheckpointResource(Checkpoint);

#[derive(Resource)]
struct MapResource(OldMap);

#[derive(Resource)]
struct LastJumpTime(Timer);

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
