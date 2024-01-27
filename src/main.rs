use bevy::ecs::world;
use bevy::{asset, prelude::*};
use bevy_xpbd_2d::prelude::*;
use bevy_xpbd_2d::{
    components::{Collider, LinearVelocity, RigidBody},
    plugins::PhysicsPlugins,
};
use serde::{Deserialize, Serialize};
use serde_json;
use tiled::{Loader, PropertyValue};

const PLAYER_SPEED: f32 = 1500.;
const MAX_PLAYER_SPEED: f32 = 300.;

const JUMP_FORCE: f32 = 11000.;
const INITIAL_JUMP_TIME: f32 = 0.20;
const GRAVITY: f32 = 1000.;
const CURVE_POW: f32 = 6.;

const GROUNDED_DECAY: f32 = 0.9;
const GROUNDED_THRESHOLD: f32 = 1.;

#[derive(Serialize, Deserialize, Component, Copy, Clone, Debug)]
struct Point {
    x: f32,
    y: f32,
}

#[derive(Component)]
struct Player;

#[derive(PhysicsLayer)]
enum Layers {
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

#[derive(Resource)]
struct Tileset(Handle<TextureAtlas>);

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
        commands.insert_resource(Tileset(handle));
    }
    commands.insert_resource(TiledMap(map));
}

#[derive(Resource)]
struct MapResource(OldMap);

#[derive(Resource)]
struct LastJumpTime(Timer);

const ENEMY_WIDTH: f32 = 10.;
const ENEMY_HEIGHT: f32 = 10.;

fn initialize_enemy_spawns(
    mut commands: Commands,
    map: Res<TiledMap>,
    texture_atlas: Res<Tileset>,
) {
    map.0.layers().into_iter().for_each(|layer| {
        layer.as_object_layer().and_then(|object_layer| {
            object_layer.objects().into_iter().for_each(|object| {
                object
                    .properties
                    .get("spawn")
                    .and_then(|spawn_id| match spawn_id {
                        PropertyValue::StringValue(id) => {
                            if id == "enemy_1" {
                                commands.spawn((
                                    Enemy,
                                    Collider::cuboid(ENEMY_WIDTH, ENEMY_HEIGHT),
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
                                            width: ENEMY_WIDTH,
                                            height: ENEMY_HEIGHT,
                                        },
                                        color: "#ff0000".to_string(),
                                    }),
                                    SpriteSheetBundle {
                                        transform: Transform::from_translation(Vec3::new(
                                            object.x, -object.y, 0.,
                                        )),
                                        sprite: TextureAtlasSprite::new(0),
                                        texture_atlas: texture_atlas.0.clone(),
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
    texture_atlas: Res<Tileset>,
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
                                                            [Layers::Player, Layers::Enemy],
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
                                                            sprite: TextureAtlasSprite::new(
                                                                t.id() as usize
                                                            ),
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
                                                sprite: TextureAtlasSprite::new(t.id() as usize),
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

fn initialize_player(mut commands: Commands, map: Res<TiledMap>) {
    let player_size = Vec2::new(10., 25.);

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

    // @TODO handle error here
    commands.spawn((
        Player,
        RigidBody::Dynamic,
        LockedAxes::ROTATION_LOCKED,
        Restitution::PERFECTLY_INELASTIC,
        Collider::cuboid(player_size.x, player_size.y),
        CollisionLayers::new([Layers::Player], [Layers::Ground, Layers::Enemy]),
        LinearVelocity::ZERO,
        SpriteBundle {
            sprite: Sprite {
                color: Color::hex("ff0000").unwrap(),
                custom_size: Some(player_size),
                ..default()
            },
            transform: Transform {
                translation: Vec3::new(player_spawn.0, player_spawn.1, 0.),
                ..default()
            },
            ..default()
        },
    ));
    commands.spawn((
        RayCaster::new(Vec2::ZERO, Vec2::NEG_Y)
            .with_query_filter(SpatialQueryFilter::new().with_masks([Layers::Ground])),
        GroundedCheck,
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
    mut player_query: Query<(&mut LinearVelocity, &Transform, &Sprite), With<Player>>,
    mut object_below_query: Query<(&mut RayCaster, &RayHits), With<GroundedCheck>>,
) {
    player_query
        .iter_mut()
        .next()
        .and_then(|(mut velocity, transform, sprite)| {
            let distance_to_closest_ground =
                object_below_query
                    .iter_mut()
                    .next()
                    .and_then(|(ray, hits)| {
                        hits.iter_sorted().next().map(|hit| {
                            (transform.translation.y - sprite.custom_size.unwrap().y / 2.)
                                - (ray.origin + ray.direction * hit.time_of_impact).y
                        })
                    });
            if keyboard_input.pressed(KeyCode::A) {
                velocity.x -= PLAYER_SPEED * time.delta_seconds();
                if MAX_PLAYER_SPEED < velocity.x.abs() {
                    velocity.x = -MAX_PLAYER_SPEED;
                }
            }
            if keyboard_input.pressed(KeyCode::D) {
                velocity.x += PLAYER_SPEED * time.delta_seconds();
                if MAX_PLAYER_SPEED < velocity.x.abs() {
                    velocity.x = MAX_PLAYER_SPEED;
                }
            }
            if let Some(distance_to_ground) = distance_to_closest_ground {
                if distance_to_ground <= GROUNDED_THRESHOLD
                    && velocity.x.abs() > 1.
                    && !keyboard_input.pressed(KeyCode::A)
                    && !keyboard_input.pressed(KeyCode::D)
                {
                    velocity.x *= GROUNDED_DECAY * time.delta_seconds();
                }
                if keyboard_input.pressed(KeyCode::Back) {
                    if distance_to_ground <= GROUNDED_THRESHOLD {
                        time_since_last_jump.0.reset();
                    }
                    if !time_since_last_jump.0.finished() && velocity.y >= 0. {
                        time_since_last_jump.0.tick(time.delta());
                        let force =
                            JUMP_FORCE * time_since_last_jump.0.percent_left().powf(CURVE_POW);
                        velocity.y += force * time.delta_seconds();
                    }
                }
            }
            Some(())
        });
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
    object_below_query: Query<(&RayCaster, &RayHits), With<SquishCheck>>,
    player_query: Query<(&Transform, &Sprite), With<Player>>,
) {
    let (player_transform, player_sprite) = player_query.iter().next().unwrap();
    object_below_query.iter().next().and_then(|(ray, hits)| {
        hits.iter_sorted()
            .next()
            .map(|hit| (hit.entity, ray.origin + ray.direction * hit.time_of_impact))
            .and_then(|(entity, point_hit)| {
                let player_y =
                    player_transform.translation.y - player_sprite.custom_size.unwrap().y / 2.;
                let difference_between_y = player_y - point_hit.y;
                if difference_between_y < 1. {
                    commands.get_entity(entity).map(|mut entity| {
                        entity.insert(DeleteMe);
                    });
                }
                Some(())
            });
        Some(())
    });
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

fn startup(mut commands: Commands) {
    commands.spawn(Camera2dBundle { ..default() });
    commands.insert_resource(LastJumpTime(Timer::from_seconds(
        INITIAL_JUMP_TIME,
        TimerMode::Once,
    )));
    commands.insert_resource(Gravity(Vec2::NEG_Y * GRAVITY));
}

fn adjust_camera(mut camera_query: Query<&mut OrthographicProjection, With<Camera2d>>) {
    camera_query.iter_mut().next().and_then(|mut projection| {
        projection.scale /= 2.;
        Some(())
    });
}

pub struct StartupPlugin;
impl Plugin for StartupPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((DefaultPlugins, PhysicsPlugins::new(PreUpdate)))
            .add_systems(Startup, startup)
            .add_systems(
                Startup,
                (
                    map_spawner,
                    apply_deferred,
                    initialize_tiled_map,
                    apply_deferred,
                    initialize_player,
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
                    follow_player,
                ),
            )
            .add_systems(PostUpdate, (delete_me, apply_deferred).chain());
    }
}

fn main() {
    App::new().add_plugins(StartupPlugin).run();
}
