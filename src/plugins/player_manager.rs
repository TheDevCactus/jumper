use bevy::{
    app::{App, Plugin, PostStartup, Update},
    ecs::{
        query::With,
        system::{Commands, Query, Res, ResMut},
    },
    input::{keyboard::KeyCode, Input},
    math::{Vec2, Vec3},
    sprite::{SpriteSheetBundle, TextureAtlasSprite},
    time::Time,
    transform::components::Transform,
};
use bevy_xpbd_2d::{
    components::{Collider, CollisionLayers, LinearVelocity, LockedAxes, Restitution, RigidBody},
    plugins::spatial_query::{RayCaster, RayHits, SpatialQueryFilter},
};
use tiled::PropertyValue;

use crate::{
    models::Constants, BottomOfPlayerRayCast, CheckpointCheck, GroundedCheck, LastJumpTime,
    LastKeyPressed, Layers, Player, Score, SquishCheck, TextureAtlasHandle, TiledMap, Tileset,
    TilesetName,
};

use super::{
    animation_manager::SpriteAnimationController, delete_manager::DeleteMe, trick_manager::Trick,
};

fn initialize_player(
    mut commands: Commands,
    map: Res<TiledMap>,
    constants: Res<Constants>,
    other_atlases: Query<(&TextureAtlasHandle, &TilesetName, &Tileset)>,
) {
    let player_spawn = map.0.layers().find_map(|layer| {
        layer
            .as_object_layer()
            .and_then(|object_layer| {
                object_layer.objects().find(|object| {
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
    if player_spawn.is_none() {
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

fn update_bottom_of_player_raycasts(
    mut ray_query: Query<&mut RayCaster, With<BottomOfPlayerRayCast>>,
    player_query: Query<&Transform, With<Player>>,
) {
    let player_transform = player_query.iter().next().unwrap();
    ray_query.iter_mut().for_each(|mut ray| {
        ray.origin = player_transform.translation.truncate();
    });
}

fn if_enemy_directly_below_player_and_falling_kill_enemy(
    mut commands: Commands,
    constants: Res<Constants>,
    object_below_query: Query<(&RayCaster, &RayHits), With<SquishCheck>>,
    mut player_query: Query<(&Transform, &mut LinearVelocity, &Collider), With<Player>>,
) {
    let (player_transform, mut player_lin_vel, player_collider) =
        player_query.iter_mut().next().unwrap();
    object_below_query.iter().next().map(|(ray, hits)| {
        hits.iter_sorted()
            .next()
            .map(|hit| (hit.entity, ray.origin + ray.direction * hit.time_of_impact))
            .map(|(entity, point_hit)| {
                let player_y = player_transform.translation.y
                    - player_collider.shape().as_cuboid().unwrap().half_extents[1];
                let difference_between_y = player_y - point_hit.y;
                if difference_between_y < 1. {
                    commands.get_entity(entity).map(|mut entity| {
                        entity.insert(DeleteMe);
                        player_lin_vel.y += constants.squish_bounce_force;
                    });
                }
                ()
            });
        ()
    });
}

fn update_velocity_with_input(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut time_since_last_jump: ResMut<LastJumpTime>,
    constants: Res<Constants>,
    mut player_query: Query<(&mut LinearVelocity, &Transform, &Collider), With<Player>>,
    mut object_below_query: Query<(&mut RayCaster, &RayHits), With<GroundedCheck>>,
) {
    player_query
        .iter_mut()
        .next()
        .map(|(mut velocity, transform, collider)| {
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
            ()
        });
}

fn hit_checkmark(
    constants: Res<Constants>,
    player_query: Query<(&Transform, &Collider), With<Player>>,
    checkmark_query: Query<(&RayCaster, &RayHits), With<CheckpointCheck>>,
) {
    checkmark_query.iter().next().map(|(ray, hits)| {
        hits.iter_sorted().next().map(|hit| {
            let hit_point = ray.origin + ray.direction * hit.time_of_impact;
            let distance_to_hit = player_query
                .iter()
                .next()
                .map(|(transform, collider)| {
                    let player_y = transform.translation.y
                        - collider.shape().as_cuboid().unwrap().half_extents[1];
                    player_y - hit_point.y
                })
                .unwrap_or(99999.);
            if constants.grounded_threshold < distance_to_hit {
                return;
            }
            println!("hit checkmark");
        });
        ()
    });
}

pub struct PlayerManager;
impl Plugin for PlayerManager {
    fn build(&self, app: &mut App) {
        app.add_systems(PostStartup, initialize_player);
        app.add_systems(
            Update,
            (
                update_bottom_of_player_raycasts,
                if_enemy_directly_below_player_and_falling_kill_enemy,
                update_velocity_with_input,
                hit_checkmark,
            ),
        );
    }
}
