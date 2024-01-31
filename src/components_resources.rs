use bevy::{
    asset::Handle,
    ecs::{component::Component, system::Resource},
    input::keyboard::KeyCode,
    sprite::TextureAtlas,
    time::Timer,
};
use serde::{Deserialize, Serialize};

pub enum Checkpoint {
    End,
}

#[derive(Component)]
pub struct Score(pub usize);

#[derive(Component)]
pub struct CheckpointCheck;

#[derive(Component)]
pub struct LastKeyPressed(pub (KeyCode, usize));

#[derive(Serialize, Deserialize, Component, Copy, Clone, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct Platform;

#[derive(Component)]
pub struct Collision;

#[derive(Component)]
pub struct Enemy;

#[derive(Component)]
pub struct RightSideOfPlayerCast;

#[derive(Component)]
pub struct LeftSideOfPlayerCast;

#[derive(Component)]
pub struct BottomOfPlayerRayCast;

#[derive(Component)]
pub struct ObjectComponent(pub Object);

#[derive(Component)]
pub struct GroundedCheck;

#[derive(Component)]
pub struct SquishCheck;

#[derive(Serialize, Deserialize, Component, Clone)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

#[derive(Resource)]
pub struct TiledMap(pub tiled::Map);
#[derive(Resource)]
pub struct TiledTileset(pub tiled::Tileset);

#[derive(Component, Resource)]
pub struct TextureAtlasHandle(pub Handle<TextureAtlas>);

#[derive(Component)]
pub struct Tileset(pub tiled::Tileset);

#[derive(Component)]
pub struct TilesetName(pub String);

#[derive(Component)]
pub struct CheckpointResource(pub Checkpoint);

#[derive(Resource)]
pub struct MapResource(pub OldMap);

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Path {
    pub end: Point,
    pub speed: f32,
    pub looped: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Object {
    pub position: Point,
    pub size: Size,
    pub color: String,
}

#[derive(Serialize, Deserialize)]
pub struct OldMap {
    pub enemies: Vec<Object>,
    pub platforms: Vec<Object>,
}

#[derive(Resource)]
pub struct LastJumpTime(pub Timer);
