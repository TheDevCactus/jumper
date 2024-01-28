use std::time::Duration;

use bevy::ecs::world;
use bevy::input::keyboard::KeyboardInput;
use bevy::time::Stopwatch;
use bevy::{asset, prelude::*};
use bevy_xpbd_2d::prelude::*;
use bevy_xpbd_2d::{
    components::{Collider, LinearVelocity, RigidBody},
    plugins::PhysicsPlugins,
};
use serde::{Deserialize, Serialize};
use serde_json;
use tiled::{Frame, Loader, PropertyValue};

#[derive(Serialize, Deserialize, Resource, Debug)]
struct Constants {
    dash_force: f32,
    trick_time: f32,
    squish_bounce_force: f32,
    character_sheet: String,
    player_speed: f32,
    max_player_speed: f32,
    jump_force: f32,
    initial_jump_time: f32,
    gravity: f32,
    curve_pow: f32,
    grounded_decay: f32,
    grounded_threshold: f32,
}

fn initialize_constants(mut commands: Commands) {
    let raw = std::fs::read_to_string("./assets/constants.toml").unwrap();
    let constants = toml::from_str::<Constants>(&raw).unwrap();
    // print the constants to the console
    commands.insert_resource(constants);
}

#[derive(Component)]
struct LoopingIncrementer {
    start: usize,
    end: usize,
    current: usize,
}

impl LoopingIncrementer {
    pub fn increment(&mut self) -> usize {
        self.current += 1;
        if self.current > self.end {
            self.current = self.start;
        }
        self.current
    }
}

#[derive(Component)]
struct AnimationTimer(Timer);

struct SpriteAnimationController;

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

fn initialize_tiled_map(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut loader = Loader::new();
    let map = loader.load_tmx_map("./assets/untitled_old.tmx").unwrap();
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
        TilesetName(characters.name.clone()),
        TextureAtlasHandle(handle.clone()),
        Tileset(characters),
    ));
    commands.insert_resource(TiledMap(map));
}

enum Checkpoint {
    End,
}


#[derive(Component)]
struct CheckpointResource(Checkpoint);

#[derive(Resource)]
struct MapResource(OldMap);

#[derive(Resource)]
struct LastJumpTime(Timer);

fn hit_checkmark(
    constants: Res<Constants>,
    player_query: Query<(&Transform, &Collider), With<Player>>,
    checkmark_query: Query<(&RayCaster, &RayHits), With<CheckpointCheck>>,    
) {
    checkmark_query.iter().next().and_then(|(ray, hits)| {
        hits.iter_sorted().next().map(|hit| {
            let hit_point = ray.origin + ray.direction * hit.time_of_impact;
            let distance_to_hit =  player_query.iter().next().map(|(transform, collider)| {
                let player_y = transform.translation.y
                    - collider.shape().as_cuboid().unwrap().half_extents[1];
                player_y - hit_point.y
            }).unwrap_or(99999.);
            if constants.grounded_threshold < distance_to_hit {
                return;
            }
            println!("hit checkmark");
        });
        Some(())
    });
}

fn initialize_checkmarks(mut commands: Commands, map: Res<TiledMap>) {
    map.0.layers().into_iter().for_each(|layer| {
        layer.as_object_layer().and_then(|object_layer| {
            object_layer.objects().into_iter().for_each(|object| {
                println!("{:?}", object.properties);
                let mut object_dimensions = (0., 0.);
                match object.shape {
                    tiled::ObjectShape::Rect { width, height } => {
                        object_dimensions = (width, height);
                    }
                    _ => {
                        return;
                    }
                }
                let object_dimensions = 
                object.properties.get("checkpoint").and_then(
                    |checkpoint_type| match checkpoint_type {
                        PropertyValue::StringValue(checkpoint_type) => {
                            if checkpoint_type == "end" {
                                commands.spawn((
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
                                            color: Color::hex( "FF0000").unwrap(),
                                            custom_size: Some(Vec2::new(object_dimensions.0, object_dimensions.1)),
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
                                println!("spawned start");
                            }
                            Some(())
                        }
                        _ => None,
                    },
                );
            });
            Some(())
        });
    })
}

fn initialize_enemy_spawns(
    mut commands: Commands,
    map: Res<TiledMap>,
    constants: Res<Constants>,
    texture_atlas: Query<(&TextureAtlasHandle, &TilesetName, &Tileset)>,
) {
    map.0.layers().into_iter().for_each(|layer| {
        layer.as_object_layer().and_then(|object_layer| {
            object_layer.objects().into_iter().for_each(|object| {
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
            Some(())
        });
    });
}

fn initialize_map_collisions(
    mut commands: Commands,
    map: Res<TiledMap>,
    texture_atlas: Res<TextureAtlasHandle>,
) {
    map.0
        .layers()
        .into_iter()
        .enumerate()
        .for_each(|(layer_index, layer)| {
            layer.as_tile_layer().and_then(|tile_layer| {
                let layer_width = tile_layer.width().unwrap();
                let layer_height = tile_layer.height().unwrap();
                (0..layer_height).for_each(|row| {
                    (0..layer_width).for_each(|col| {
                        tile_layer.get_tile(col as i32, row as i32).and_then(|t| {
                            t.get_tile().and_then(|tile| {
                                match tile.collision.as_ref() {
                                    Some(collision) => {
                                        collision.object_data().iter().for_each(|object| {
                                            match object.shape {
                                                tiled::ObjectShape::Rect { width, height } => {
                                                    let tile_pos = (
                                                        col * map.0.tile_width,
                                                        row * map.0.tile_height,
                                                    );
                                                    commands.spawn((
                                                        Platform,
                                                        Collision,
                                                        Collider::cuboid(width, height),
                                                        LinearVelocity::ZERO,
                                                        RigidBody::Static,
                                                        CollisionLayers::new(
                                                            [Layers::Ground],
                                                            [Layers::Player, Layers::Enemy, Layers::Checkpoint],
                                                        ),
                                                        ObjectComponent(Object {
                                                            position: Point {
                                                                x: tile_pos.0 as f32,
                                                                y: -(tile_pos.1 as f32),
                                                            },
                                                            size: Size {
                                                                width: width,
                                                                height: height,
                                                            },
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
                                        let tile_pos =
                                            (col * map.0.tile_width, row * map.0.tile_height);
                                        commands.spawn((
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
                                }
                                Some(())
                            });
                            Some(())
                        });
                    });
                });
                Some(())
            });
        });
}

#[derive(Component)]
struct Score(usize);

#[derive(Component)]
struct CheckpointCheck;

#[derive(Component)]
struct LastKeyPressed((KeyCode, usize));

fn initialize_player(
    mut commands: Commands,
    map: Res<TiledMap>,
    constants: Res<Constants>,
    other_atlases: Query<(&TextureAtlasHandle, &TilesetName, &Tileset)>,
) {
    let player_spawn = map.0.layers().into_iter().find_map(|layer| {
        layer
            .as_object_layer()
            .and_then(|object_layer| {
                object_layer.objects().into_iter().find(|object| {
                    object
                        .properties
                        .get("spawn")
                        .and_then(|spawn_id| match spawn_id {
                            PropertyValue::StringValue(id) => {
                                if id == "player" {
                                    Some(())
                                } else {
                                    None
                                }
                            }
                            _ => None,
                        })
                        .is_some()
                })
            })
            .map(|object| (object.x, -object.y))
    });
    if let None = player_spawn {
        panic!("No player spawn found");
    }
    let player_spawn = player_spawn.unwrap();

    let (char_atlas, _, char_tileset) = other_atlases
        .iter()
        .find(|(_, name, _)| name.0 == constants.character_sheet)
        .unwrap();
    // @TODO handle error here
    commands.spawn((
        Player,
        Trick::new(),
        Score(0),
        RigidBody::Dynamic,
        LockedAxes::ROTATION_LOCKED,
        Restitution::ZERO,
        Collider::cuboid(
            char_tileset.0.tile_width as f32,
            char_tileset.0.tile_height as f32,
        ),
        LastKeyPressed((KeyCode::A, 0)),
        CollisionLayers::new([Layers::Player], [Layers::Ground, Layers::Enemy]),
        LinearVelocity::ZERO,
        SpriteSheetBundle {
            transform: Transform::from_translation(Vec3::new(player_spawn.0, player_spawn.1, 0.)),
            sprite: TextureAtlasSprite::new(26),
            texture_atlas: char_atlas.0.clone(),
            ..Default::default()
        },
        SpriteAnimationController::new(26, 29, 100.),
    ));
    commands.spawn((
        RayCaster::new(Vec2::ZERO, Vec2::NEG_Y)
            .with_query_filter(SpatialQueryFilter::new().with_masks([Layers::Ground])),
        GroundedCheck,
        BottomOfPlayerRayCast,
    ));

    commands.spawn((
        RayCaster::new(Vec2::ZERO, Vec2::NEG_Y)
            .with_query_filter(SpatialQueryFilter::new().with_masks([Layers::Checkpoint])),
        CheckpointCheck,
        BottomOfPlayerRayCast,
    ));

    commands.spawn((
        RayCaster::new(Vec2::ZERO, Vec2::NEG_Y)
            .with_query_filter(SpatialQueryFilter::new().with_masks([Layers::Enemy])),
        SquishCheck,
        BottomOfPlayerRayCast,
    ));
}

fn follow_player(
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    player_query: Query<&Transform, (With<Player>, Without<Camera2d>)>,
) {
    player_query.iter().next().and_then(|player_transform| {
        camera_query
            .iter_mut()
            .next()
            .and_then(|mut camera_transform| {
                camera_transform.translation.x = player_transform.translation.x;
                camera_transform.translation.y = player_transform.translation.y;
                Some(())
            })
    });
}

fn update_velocity_with_input(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut time_since_last_jump: ResMut<LastJumpTime>,
    constants: Res<Constants>,
    mut player_query: Query<
        (&mut LinearVelocity, &Transform, &Collider, &LastKeyPressed),
        With<Player>,
    >,
    mut object_below_query: Query<(&mut RayCaster, &RayHits), With<GroundedCheck>>,
) {
    player_query.iter_mut().next().and_then(
        |(mut velocity, transform, collider, last_key_pressed)| {
            let distance_to_closest_ground =
                object_below_query
                    .iter_mut()
                    .next()
                    .and_then(|(ray, hits)| {
                        hits.iter_sorted().next().map(|hit| {
                            (transform.translation.y
                                - collider.shape().as_cuboid().unwrap().half_extents[1])
                                - (ray.origin + ray.direction * hit.time_of_impact).y
                        })
                    });
            if let Some(distance_to_ground) = distance_to_closest_ground {
                if distance_to_ground < constants.grounded_threshold {
                    if keyboard_input.pressed(KeyCode::A) {
                        velocity.x -= constants.player_speed * time.delta_seconds();
                        if constants.max_player_speed < velocity.x.abs() {
                            velocity.x = -constants.max_player_speed;
                        }
                    }
                    if keyboard_input.pressed(KeyCode::D) {
                        velocity.x += constants.player_speed * time.delta_seconds();
                        if constants.max_player_speed < velocity.x.abs() {
                            velocity.x = constants.max_player_speed;
                        }
                    }
                }
                if keyboard_input.pressed(KeyCode::Back) {
                    if distance_to_ground <= constants.grounded_threshold {
                        time_since_last_jump.0.reset();
                    }
                    if !time_since_last_jump.0.finished() && velocity.y >= 0. {
                        time_since_last_jump.0.tick(time.delta());
                        let force = constants.jump_force
                            * time_since_last_jump
                                .0
                                .percent_left()
                                .powf(constants.curve_pow);
                        velocity.y += force * time.delta_seconds();
                    }
                }
            }
            Some(())
        },
    );
}

fn read_map(file_path: String) -> OldMap {
    let map = std::fs::read_to_string(file_path).unwrap();
    let map: OldMap = serde_json::from_str(&map).unwrap();
    map
}

fn map_spawner(mut commands: Commands) {
    let map = read_map("./maps/map.json".to_string());
    commands.insert_resource(MapResource(map));
}

#[derive(Component)]
struct DeleteMe;

fn if_enemy_directly_below_player_and_falling_kill_enemy(
    mut commands: Commands,
    constants: Res<Constants>,
    object_below_query: Query<(&RayCaster, &RayHits), With<SquishCheck>>,
    mut player_query: Query<(&Transform, &mut LinearVelocity, &Collider), With<Player>>,
) {
    let (player_transform, mut player_lin_vel, player_collider) =
        player_query.iter_mut().next().unwrap();
    object_below_query.iter().next().and_then(|(ray, hits)| {
        hits.iter_sorted()
            .next()
            .map(|hit| (hit.entity, ray.origin + ray.direction * hit.time_of_impact))
            .and_then(|(entity, point_hit)| {
                let player_y = player_transform.translation.y
                    - player_collider.shape().as_cuboid().unwrap().half_extents[1];
                let difference_between_y = player_y - point_hit.y;
                if difference_between_y < 1. {
                    commands.get_entity(entity).map(|mut entity| {
                        entity.insert(DeleteMe);
                        player_lin_vel.y += constants.squish_bounce_force;
                    });
                }
                Some(())
            });
        Some(())
    });
}

#[derive(Serialize, Deserialize, Debug, Clone)]
enum Direction {
    Up,
    Left,
    Down,
    Right,
}

#[derive(Component, Resource)]
struct Trick {
    last_trick_definition: Option<TrickDefinition>,
    last_trick_over: Timer,
    keys: Vec<KeyCode>,
}
impl Trick {
    pub fn new() -> Self {
        Self {
            keys: vec![],
            last_trick_definition: None,
            last_trick_over: Timer::from_seconds(0.5, TimerMode::Once),
        }
    }
    pub fn add_key(&mut self, key: KeyCode) {
        self.keys.push(key);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct TrickDefinition {
    name: String,
    points: usize,
    takes_ms: usize,
}

#[derive(Resource, Serialize, Deserialize, Debug)]
struct TrickList(Vec<(Vec<KeyCode>, TrickDefinition)>);

impl TrickList {
    pub fn find_trick(&mut self, keys: &Vec<KeyCode>) -> Option<&TrickDefinition> {
        self.0.iter().find_map(|(trick_keys, trick_definition)| {
            if trick_keys.len() != keys.len() {
                return None;
            }
            for i in 0..trick_keys.len() {
                if trick_keys[i] != keys[i] {
                    return None;
                }
            }
            Some(trick_definition)
        })
    }
}

fn initialize_trick_list(mut commands: Commands) {
    let raw = std::fs::read_to_string("./assets/trick_list.json").unwrap();
    let mut trick_list = serde_json::from_str::<TrickList>(&raw).unwrap();
    // sort the trick list by most keys first
    trick_list
        .0
        .sort_by(|(keys_a, _), (keys_b, _)| keys_b.len().cmp(&keys_a.len()));
    commands.insert_resource(trick_list);
}

fn trick_manager(
    time: Res<Time>,
    constants: Res<Constants>,
    mut object_below_query: Query<(&RayCaster, &RayHits), With<GroundedCheck>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut trick_list: ResMut<TrickList>,
    mut player_query: Query<
        (
            &mut LinearVelocity,
            &mut Trick,
            &mut Score,
            &mut LastKeyPressed,
            &Transform,
            &Collider,
        ),
        With<Player>,
    >,
) {
    player_query.iter_mut().next().and_then(
        |(_, mut current_trick, mut score, mut last_key_pressed, transform, collider)| {
            current_trick.last_trick_over.tick(time.delta());

            let current_ms = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis();
            if current_ms - (last_key_pressed.0 .1 as u128) > constants.trick_time as u128 {
                current_trick.keys.clear();
            }
            let hit_point = object_below_query
                .iter_mut()
                .next()
                .and_then(|(ray, hits)| {
                    hits.iter_sorted()
                        .next()
                        .map(|hit| (ray.origin + ray.direction * hit.time_of_impact).y)
                })
                .unwrap_or(99999.);
            let distance_to_ground = transform.translation.y
                - hit_point
                - collider.shape().as_cuboid().unwrap().half_extents[1];
            if distance_to_ground < constants.grounded_threshold
                && !current_trick.last_trick_over.finished()
                && current_trick.last_trick_definition.is_some()
            {
                println!("Failed trick");
                current_trick.keys.clear();
                current_trick.last_trick_definition = None;
                return Some(());
            }
            if current_trick.last_trick_over.just_finished()
                && current_trick.last_trick_definition.is_some()
            {
                score.0 += current_trick.last_trick_definition.as_ref().unwrap().points;
                println!(
                    "Score: {}, Landed: {:?}",
                    score.0 * 10,
                    current_trick.last_trick_definition.as_ref().unwrap().name
                );
                current_trick.keys.clear();
                return Some(());
            }

            let mut key: Option<KeyCode> = None;
            if keyboard_input.just_pressed(KeyCode::W) {
                key = Some(KeyCode::W);
            }
            if keyboard_input.just_pressed(KeyCode::A) {
                key = Some(KeyCode::A);
            }
            if keyboard_input.just_pressed(KeyCode::S) {
                key = Some(KeyCode::S);
            }
            if keyboard_input.just_pressed(KeyCode::D) {
                key = Some(KeyCode::D);
            }
            if let None = key {
                return Some(());
            }

            let current_key = key.unwrap();
            last_key_pressed.0 = (current_key, current_ms as usize);
            match key.unwrap() {
                KeyCode::A | KeyCode::W | KeyCode::S | KeyCode::D => {
                    if !current_trick.last_trick_over.finished() {
                        if current_trick.keys.len() > 1 {
                            current_trick.keys.clear();
                            current_trick.last_trick_over.reset();
                        };
                        return Some(());
                    }
                    current_trick.add_key(current_key);
                    trick_list
                        .find_trick(&current_trick.keys)
                        .and_then(|trick| {
                            current_trick.keys.clear();
                            current_trick
                                .last_trick_over
                                .set_duration(Duration::from_millis(trick.takes_ms as u64));
                            current_trick.last_trick_over.reset();
                            current_trick.last_trick_definition = Some(trick.clone());
                            println!("executing: {:?}", trick.name);
                            Some(())
                        })
                        .or_else(|| {
                            if current_trick.keys.len() > 1 {
                                current_trick.keys.clear();
                                current_trick.last_trick_over.reset();
                            };
                            Some(())
                        });
                }
                _ => {}
            }

            Some(())
        },
    );
}

fn delete_me(mut commands: Commands, query: Query<Entity, With<DeleteMe>>) {
    query.iter().for_each(|entity| {
        commands.entity(entity).despawn();
    });
}

fn update_bottom_of_player_raycasts(
    mut ray_query: Query<&mut RayCaster, With<BottomOfPlayerRayCast>>,
    player_query: Query<&Transform, With<Player>>,
) {
    let player_transform = player_query.iter().next().unwrap();
    ray_query.iter_mut().for_each(|mut ray| {
        ray.origin = player_transform.translation.truncate();
    });
}

fn startup(mut commands: Commands, constants: Res<Constants>) {
    commands.spawn(Camera2dBundle { ..default() });
    commands.insert_resource(LastJumpTime(Timer::from_seconds(
        constants.initial_jump_time,
        TimerMode::Once,
    )));
    commands.insert_resource(Gravity(Vec2::NEG_Y * constants.gravity));
}

fn adjust_camera(mut camera_query: Query<&mut OrthographicProjection, With<Camera2d>>) {
    camera_query.iter_mut().next().and_then(|mut projection| {
        projection.scale /= 2.5;
        Some(())
    });
}

impl SpriteAnimationController {
    pub fn new(
        start: usize,
        end: usize,
        ms_per_frame: f32,
    ) -> (LoopingIncrementer, AnimationTimer) {
        (
            LoopingIncrementer {
                start,
                end,
                current: start,
            },
            AnimationTimer(Timer::from_seconds(
                ms_per_frame / 1000.,
                TimerMode::Repeating,
            )),
        )
    }
}

fn update_animated_sprites(
    time: Res<Time>,
    mut query: Query<(
        &mut AnimationTimer,
        &mut TextureAtlasSprite,
        &mut LoopingIncrementer,
    )>,
) {
    query
        .iter_mut()
        .for_each(|(mut timer, mut sprite, mut incrementer)| {
            timer.0.tick(time.delta());
            if timer.0.finished() {
                sprite.index = incrementer.increment();
            }
        });
}

pub struct StartupPlugin;
impl Plugin for StartupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DefaultPlugins, PhysicsPlugins::new(PreUpdate)))
            .add_systems(
                Startup,
                (
                    initialize_trick_list,
                    initialize_constants,
                    apply_deferred,
                    startup,
                    map_spawner,
                    apply_deferred,
                    initialize_tiled_map,
                    apply_deferred,
                    initialize_player,
                    initialize_checkmarks,
                    initialize_map_collisions,
                    initialize_enemy_spawns,
                )
                    .chain(),
            )
            .add_systems(PostStartup, adjust_camera)
            .add_systems(
                Update,
                (
                    update_bottom_of_player_raycasts,
                    if_enemy_directly_below_player_and_falling_kill_enemy,
                    update_velocity_with_input,
                    trick_manager,
                    hit_checkmark,
                    follow_player,
                ),
            )
            .add_systems(Update, update_animated_sprites)
            .add_systems(PostUpdate, (delete_me, apply_deferred).chain());
    }
}

fn main() {
    App::new().add_plugins(StartupPlugin).run();
}
