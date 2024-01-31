use std::{ops::Add, path::Display};

use bevy::{
    app::{App, Plugin, PostStartup, Update},
    ecs::{
        query::{With, Without},
        schedule::{common_conditions::in_state, IntoSystemConfigs, State},
        system::{Commands, Query, Res, ResMut},
    },
    input::{keyboard::KeyCode, Input},
    math::{Quat, Vec2, Vec3},
    sprite::{SpriteBundle, SpriteSheetBundle, TextureAtlasSprite},
    time::Time,
    transform::components::Transform,
};
use bevy_xpbd_2d::{
    components::{Collider, CollisionLayers, LinearVelocity, LockedAxes, Restitution, RigidBody},
    math::Scalar,
    parry::{either::Either::Left, na::RealField},
    plugins::spatial_query::{RayCaster, RayHits, ShapeCaster, ShapeHits, SpatialQueryFilter},
};
use tiled::PropertyValue;

use crate::{
    components_resources::{
        BottomOfPlayerRayCast, CheckpointCheck, GroundedCheck, LastJumpTime, LastKeyPressed,
        LeftSideOfPlayerCast, Player, RightSideOfPlayerCast, Score, SquishCheck,
        TextureAtlasHandle, TiledMap, Tileset, TilesetName,
    },
    models::BelongsToScene,
    scenes::Scene,
    service::constants::Constants,
};

use super::{
    animation_manager::SpriteAnimationController, delete_manager::DeleteMe, physics::Layers,
    trick_manager::Trick,
};

pub fn initialize_player(
    mut commands: Commands,
    map: Res<TiledMap>,
    constants: Res<Constants>,
    other_atlases: Query<(&TextureAtlasHandle, &TilesetName, &Tileset)>,
    scene: Res<State<Scene>>,
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
        BelongsToScene(scene.clone()),
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
        BelongsToScene(scene.clone()),
        ShapeCaster::new(
            Collider::cuboid(char_tileset.0.tile_width as f32, 5.),
            Vec2::ZERO,
            Scalar::default(),
            Vec2::NEG_Y,
        )
        .with_query_filter(SpatialQueryFilter::new().with_masks([Layers::Ground])),
        GroundedCheck,
        BottomOfPlayerRayCast,
    ));

    commands.spawn((
        BelongsToScene(scene.clone()),
        RightSideOfPlayerCast,
        RayCaster::new(Vec2::ZERO, Vec2::X)
            .with_query_filter(SpatialQueryFilter::new().with_masks([Layers::Ground])),
    ));
    commands.spawn((
        BelongsToScene(scene.clone()),
        LeftSideOfPlayerCast,
        RayCaster::new(Vec2::ZERO, Vec2::NEG_X)
            .with_query_filter(SpatialQueryFilter::new().with_masks([Layers::Ground])),
    ));

    commands.spawn((
        BelongsToScene(scene.clone()),
        RayCaster::new(Vec2::ZERO, Vec2::NEG_Y)
            .with_query_filter(SpatialQueryFilter::new().with_masks([Layers::Checkpoint])),
        CheckpointCheck,
        BottomOfPlayerRayCast,
    ));

    commands.spawn((
        BelongsToScene(scene.clone()),
        RayCaster::new(Vec2::ZERO, Vec2::NEG_Y)
            .with_query_filter(SpatialQueryFilter::new().with_masks([Layers::Enemy])),
        SquishCheck,
        BottomOfPlayerRayCast,
    ));
}

fn update_sides_of_player_raycasts(
    mut left_ray_query: Query<
        &mut RayCaster,
        (With<LeftSideOfPlayerCast>, Without<RightSideOfPlayerCast>),
    >,
    mut right_ray_query: Query<
        &mut RayCaster,
        (With<RightSideOfPlayerCast>, Without<LeftSideOfPlayerCast>),
    >,
    player_query: Query<&Transform, With<Player>>,
) {
    let player_transform = player_query.iter().next().unwrap();
    let new_pos = player_transform.translation.truncate();
    left_ray_query
        .iter_mut()
        .for_each(|mut ray| ray.origin = new_pos);
    right_ray_query
        .iter_mut()
        .for_each(|mut ray| ray.origin = new_pos);
}

fn update_bottom_of_player_raycasts(
    mut ray_query: Query<&mut RayCaster, With<BottomOfPlayerRayCast>>,
    mut shape_query: Query<&mut ShapeCaster, With<BottomOfPlayerRayCast>>,
    player_query: Query<&Transform, With<Player>>,
) {
    let player_transform = player_query.iter().next().unwrap();
    ray_query.iter_mut().for_each(|mut ray| {
        ray.origin = player_transform.translation.truncate();
    });
    shape_query.iter_mut().for_each(|mut shape| {
        shape.origin = player_transform
            .translation
            .truncate()
            .add(Vec2::new(0., -16.));
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
    if let Some((ray, hits)) = object_below_query.iter().next() {
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
            });
    }
}

fn update_velocity_with_input(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut time_since_last_jump: ResMut<LastJumpTime>,
    constants: Res<Constants>,
    mut player_query: Query<(&mut LinearVelocity, &Transform, &Collider), With<Player>>,
    mut object_below_query: Query<(&mut ShapeCaster, &ShapeHits), With<GroundedCheck>>,
    mut right_side_query: Query<
        (&mut RayCaster, &RayHits),
        (With<RightSideOfPlayerCast>, Without<LeftSideOfPlayerCast>),
    >,
    mut left_side_query: Query<
        (&mut RayCaster, &RayHits),
        (With<LeftSideOfPlayerCast>, Without<RightSideOfPlayerCast>),
    >,
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
                        hits.iter().next().map(|hit| {
                            (transform.translation.y
                                - collider.shape().as_cuboid().unwrap().half_extents[1])
                                - (ray.origin + ray.direction * hit.time_of_impact).y
                        })
                    });
            let distance_to_right = right_side_query
                .iter_mut()
                .map(|(ray, hits)| {
                    hits.iter().next().map(|hit| {
                        (transform.translation.x
                            - collider.shape().as_cuboid().unwrap().half_extents[0])
                            - (ray.origin + ray.direction * hit.time_of_impact).x
                    })
                })
                .next();
            if let Some(distance_to_right) = distance_to_right {
                if let Some(distance_to_right) = distance_to_right {
                    println!("{:?}", distance_to_right);
                    if distance_to_right.abs() < constants.wall_threshold + 32. {
                        if keyboard_input.pressed(KeyCode::Back) {
                            velocity.y += constants.jump_force * time.delta_seconds();
                            velocity.x -= constants.dash_force * time.delta_seconds();
                        }
                    }
                }
            }
            let distance_to_left = left_side_query
                .iter_mut()
                .map(|(ray, hits)| {
                    hits.iter().next().map(|hit| {
                        (transform.translation.x
                            + collider.shape().as_cuboid().unwrap().half_extents[0])
                            - (ray.origin + ray.direction * hit.time_of_impact).x
                    })
                })
                .next();
            if let Some(distance_to_left) = distance_to_left {
                if let Some(distance_to_left) = distance_to_left {
                    println!("{:?}", distance_to_left);
                    if distance_to_left.abs() < constants.wall_threshold + 32. {
                        if keyboard_input.pressed(KeyCode::Back) {
                            velocity.y += constants.jump_force * time.delta_seconds();
                            velocity.x += constants.dash_force * time.delta_seconds();
                        }
                    }
                }
            }
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
                            if velocity.x < 0. {
                                velocity.x = -constants.max_player_speed;
                            } else {
                                velocity.x = constants.max_player_speed;
                            }
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
        });
}

pub struct PlayerManager {
    pub scene: Scene,
}
impl Plugin for PlayerManager {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                update_sides_of_player_raycasts.run_if(in_state(self.scene)),
                update_bottom_of_player_raycasts.run_if(in_state(self.scene)),
                if_enemy_directly_below_player_and_falling_kill_enemy.run_if(in_state(self.scene)),
                update_velocity_with_input.run_if(in_state(self.scene)),
            ),
        );
    }
}
