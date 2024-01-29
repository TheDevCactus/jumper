use std::time::Duration;

use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        component::Component,
        query::With,
        system::{Query, Res, ResMut, Resource},
    },
    input::{keyboard::KeyCode, Input},
    time::{Time, Timer, TimerMode},
    transform::components::Transform,
};
use bevy_xpbd_2d::{
    components::{Collider, LinearVelocity},
    plugins::spatial_query::{RayCaster, RayHits},
};

use crate::{
    models::{Constants, TrickDefinition, TrickListResource},
    GroundedCheck, LastKeyPressed, Player, Score,
};

#[derive(Component, Resource)]
pub struct Trick {
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

fn trick_manager(
    time: Res<Time>,
    constants: Res<Constants>,
    mut object_below_query: Query<(&RayCaster, &RayHits), With<GroundedCheck>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut trick_list: ResMut<TrickListResource>,
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
                        .0
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

pub struct TrickManager;
impl Plugin for TrickManager {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, trick_manager);
    }
}
