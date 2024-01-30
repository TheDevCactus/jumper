use bevy::{
    app::{App, Plugin, PostStartup, Startup},
    asset::AssetServer,
    ecs::{
        schedule::{apply_deferred, IntoSystemConfigs, ScheduleLabel, State},
        system::{Commands, Query, Res},
    },
    log::warn,
    math::{Vec2, Vec3},
    render::color::Color,
    sprite::{Sprite, SpriteBundle, SpriteSheetBundle, TextureAtlas, TextureAtlasSprite},
    transform::components::Transform,
    utils::intern::Interned,
};
use bevy_xpbd_2d::components::{Collider, CollisionLayers, LinearVelocity, LockedAxes, RigidBody};
use tiled::{Loader, PropertyValue};

use crate::{
    components_resources::{
        Checkpoint, CheckpointResource, Collision, Enemy, Object, ObjectComponent, Platform, Point,
        Size, TextureAtlasHandle, TiledMap, Tileset, TilesetName,
    },
    models::BelongsToScene,
    plugins::physics::Layers,
    scenes::{level::LevelID, Scene},
    service::constants::Constants,
};

use super::player_manager::initialize_player;

pub fn initialize_tiled_map(
    mut commands: Commands,
    constants: Res<Constants>,
    level_id: Res<LevelID>,
    asset_server: Res<AssetServer>,
    scene: Res<State<Scene>>,
) {
    let mut loader = Loader::new();
    let file_path = format!("./assets/{}", level_id.0);
    let map = loader.load_tmx_map(file_path).unwrap();
    let tilesets = map.tilesets();
    for tileset in tilesets {
        let p = tileset.image.as_ref().unwrap().source.to_str().unwrap();
        let x = String::from(p);
        let x = x.split("assets").collect::<Vec<&str>>()[1];
        let x = format!(".{}", x);
        let atlas = TextureAtlas::from_grid(
            asset_server.load(x),
            Vec2::new(tileset.tile_width as f32, tileset.tile_height as f32),
            tileset.columns as usize,
            tileset.tilecount as usize / tileset.columns as usize,
            Some(Vec2::new(tileset.offset_x as f32, tileset.offset_y as f32)),
            Some(Vec2::new(0., -0.5)),
        );
        let handle = asset_server.add(atlas);
        commands.spawn((
            BelongsToScene(scene.clone()),
            TilesetName(tileset.name.clone()),
            TextureAtlasHandle(handle.clone()),
        ));
        commands.insert_resource(TextureAtlasHandle(handle))
    }
    let characters = loader.load_tsx_tileset("./assets/characters.tsx").unwrap();
    let p = characters.image.as_ref().unwrap().source.to_str().unwrap();
    let x = String::from(p);
    let x = x.split("assets").collect::<Vec<&str>>()[1];
    let x = format!(".{}", x);
    let atlas = TextureAtlas::from_grid(
        asset_server.load(x),
        Vec2::new(characters.tile_width as f32, characters.tile_height as f32),
        characters.columns as usize,
        characters.tilecount as usize / characters.columns as usize,
        Some(Vec2::new(
            characters.offset_x as f32,
            characters.offset_y as f32,
        )),
        Some(Vec2::new(0., -0.5)),
    );
    let handle = asset_server.add(atlas);
    commands.spawn((
        BelongsToScene(scene.clone()),
        TilesetName(characters.name.clone()),
        TextureAtlasHandle(handle.clone()),
        Tileset(characters),
    ));
    commands.insert_resource(TiledMap(map));
}

fn initialize_checkmarks(mut commands: Commands, map: Res<TiledMap>, scene: Res<State<Scene>>) {
    map.0.layers().for_each(|layer| {
        if let Some(object_layer) = layer.as_object_layer() {
            object_layer.objects().for_each(|object| {
                let mut object_dimensions = (0., 0.);
                match object.shape {
                    tiled::ObjectShape::Rect { width, height } => {
                        object_dimensions = (width, height);
                    }
                    _ => {
                        return;
                    }
                }
                let _object_dimensions =
                    object
                        .properties
                        .get("checkpoint")
                        .and_then(|checkpoint_type| match checkpoint_type {
                            PropertyValue::StringValue(checkpoint_type) => {
                                if checkpoint_type == "end" {
                                    commands.spawn((
                                        BelongsToScene(scene.clone()),
                                        ObjectComponent(Object {
                                            position: Point {
                                                x: object.x,
                                                y: -object.y,
                                            },
                                            size: Size {
                                                width: object_dimensions.0,
                                                height: object_dimensions.1,
                                            },
                                            color: "#ff0000".to_string(),
                                        }),
                                        SpriteBundle {
                                            transform: Transform::from_translation(Vec3::new(
                                                object.x, -object.y, 0.,
                                            )),
                                            sprite: Sprite {
                                                color: Color::hex("FF0000").unwrap(),
                                                custom_size: Some(Vec2::new(
                                                    object_dimensions.0,
                                                    object_dimensions.1,
                                                )),
                                                ..Default::default()
                                            },
                                            ..Default::default()
                                        },
                                        Collider::cuboid(100., 100.),
                                        CollisionLayers::new(
                                            [Layers::Checkpoint],
                                            [Layers::Player],
                                        ),
                                        CheckpointResource(Checkpoint::End),
                                    ));
                                }
                                Some(())
                            }
                            _ => None,
                        });
            });
        }
    })
}

fn initialize_enemy_spawns(
    mut commands: Commands,
    map: Res<TiledMap>,
    constants: Res<Constants>,
    texture_atlas: Query<(&TextureAtlasHandle, &TilesetName, &Tileset)>,
    scene: Res<State<Scene>>,
) {
    map.0.layers().for_each(|layer| {
        if let Some(object_layer) = layer.as_object_layer() {
            object_layer.objects().for_each(|object| {
                object
                    .properties
                    .get("spawn")
                    .and_then(|spawn_id| match spawn_id {
                        PropertyValue::StringValue(id) => {
                            let character_atlas = texture_atlas
                                .iter()
                                .find(|(_, name, _)| name.0 == constants.character_sheet)
                                .unwrap();
                            if id == "enemy_1" {
                                commands.spawn((
                                    BelongsToScene(scene.clone()),
                                    Enemy,
                                    Collider::cuboid(
                                        character_atlas.2 .0.tile_width as f32,
                                        character_atlas.2 .0.tile_height as f32,
                                    ),
                                    LinearVelocity::ZERO,
                                    RigidBody::Dynamic,
                                    LockedAxes::ROTATION_LOCKED,
                                    CollisionLayers::new(
                                        [Layers::Enemy],
                                        [Layers::Ground, Layers::Player],
                                    ),
                                    ObjectComponent(Object {
                                        position: Point {
                                            x: object.x,
                                            y: -object.y,
                                        },
                                        size: Size {
                                            width: character_atlas.2 .0.tile_width as f32,
                                            height: character_atlas.2 .0.tile_height as f32,
                                        },
                                        color: "#ff0000".to_string(),
                                    }),
                                    SpriteSheetBundle {
                                        transform: Transform::from_translation(Vec3::new(
                                            object.x, -object.y, 0.,
                                        )),
                                        sprite: TextureAtlasSprite::new(0),
                                        texture_atlas: character_atlas.0 .0.clone(),
                                        ..Default::default()
                                    },
                                ));
                                Some(())
                            } else {
                                None
                            }
                        }
                        _ => None,
                    });
            });
        }
    });
}

fn initialize_map_collisions(
    mut commands: Commands,
    map: Res<TiledMap>,
    texture_atlas: Res<TextureAtlasHandle>,
    scene: Res<State<Scene>>,
) {
    map.0.layers().enumerate().for_each(|(layer_index, layer)| {
        layer.as_tile_layer().map(|tile_layer| {
            let layer_width = tile_layer.width().unwrap();
            let layer_height = tile_layer.height().unwrap();
            (0..layer_height).for_each(|row| {
                (0..layer_width).for_each(|col| {
                    if let Some(t) = tile_layer.get_tile(col as i32, row as i32) {
                        t.get_tile().map(|tile| match tile.collision.as_ref() {
                            Some(collision) => {
                                collision.object_data().iter().for_each(|object| {
                                    match object.shape {
                                        tiled::ObjectShape::Rect { width, height } => {
                                            let tile_pos =
                                                (col * map.0.tile_width, row * map.0.tile_height);
                                            commands.spawn((
                                                BelongsToScene(scene.clone()),
                                                Platform,
                                                Collision,
                                                Collider::cuboid(width, height),
                                                LinearVelocity::ZERO,
                                                RigidBody::Static,
                                                CollisionLayers::new(
                                                    [Layers::Ground],
                                                    [
                                                        Layers::Player,
                                                        Layers::Enemy,
                                                        Layers::Checkpoint,
                                                    ],
                                                ),
                                                ObjectComponent(Object {
                                                    position: Point {
                                                        x: tile_pos.0 as f32,
                                                        y: -(tile_pos.1 as f32),
                                                    },
                                                    size: Size { width, height },
                                                    color: "#ff0000".to_string(),
                                                }),
                                                SpriteSheetBundle {
                                                    transform: Transform::from_translation(
                                                        Vec3::new(
                                                            tile_pos.0 as f32,
                                                            -(tile_pos.1 as f32),
                                                            layer_index as f32,
                                                        ),
                                                    ),
                                                    sprite: TextureAtlasSprite {
                                                        flip_x: t.flip_h,
                                                        index: t.id() as usize,
                                                        ..Default::default()
                                                    },
                                                    texture_atlas: texture_atlas.0.clone(),
                                                    ..Default::default()
                                                },
                                            ));
                                        }
                                        _ => {
                                            warn!("Unsupported shape");
                                        }
                                    };
                                });
                            }
                            None => {
                                let tile_pos = (col * map.0.tile_width, row * map.0.tile_height);
                                commands.spawn((
                                    BelongsToScene(scene.clone()),
                                    ObjectComponent(Object {
                                        position: Point {
                                            x: tile_pos.0 as f32,
                                            y: -(tile_pos.1 as f32),
                                        },
                                        size: Size {
                                            width: map.0.tile_width as f32,
                                            height: map.0.tile_height as f32,
                                        },
                                        color: "#ff0000".to_string(),
                                    }),
                                    SpriteSheetBundle {
                                        transform: Transform::from_translation(Vec3::new(
                                            tile_pos.0 as f32,
                                            -(tile_pos.1 as f32),
                                            layer_index as f32,
                                        )),
                                        sprite: TextureAtlasSprite {
                                            flip_x: t.flip_h,
                                            index: t.id() as usize,
                                            ..Default::default()
                                        },
                                        texture_atlas: texture_atlas.0.clone(),
                                        ..Default::default()
                                    },
                                ));
                            }
                        });
                    }
                });
            });
        });
    });
}

pub struct LevelLoader {
    pub startup: Interned<dyn ScheduleLabel>,
    // pub post_startup: Interned<dyn ScheduleLabel>,
}

impl Plugin for LevelLoader {
    fn build(&self, app: &mut App) {
        // @todo this is kinda jank, fix later
        app.add_systems(
            self.startup,
            (
                initialize_tiled_map,
                apply_deferred,
                (
                    initialize_checkmarks,
                    initialize_map_collisions,
                    initialize_enemy_spawns,
                    // @TODO this should be in player but needs to be ran after tiled map is initialized
                    initialize_player,
                ),
            )
                .chain(),
        );
    }
}
