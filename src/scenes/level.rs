use std::time::Duration;

use bevy::{
    a11y::accesskit::Rect,
    app::{App, Plugin, Update},
    asset::AssetServer,
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        schedule::{
            common_conditions::in_state, IntoSystemConfigs, NextState, OnEnter, OnExit,
            ScheduleLabel, State, States,
        },
        system::{Commands, NonSend, Query, Res, ResMut, Resource},
    },
    input::{keyboard::KeyCode, Input},
    render::{color::Color, view::window},
    text::{Text, TextSection, TextStyle},
    time::{Stopwatch, Time, Timer},
    transform::{commands, components::Transform},
    ui::{node_bundles::TextBundle, AlignSelf, PositionType, Style, Val},
    utils::default,
    window::{PrimaryWindow, Window, WindowResolution},
    winit::WinitWindows,
};
use bevy_xpbd_2d::{
    components::Collider,
    plugins::spatial_query::{RayCaster, RayHits},
};
use serde::{Deserialize, Serialize};

use crate::{
    components_resources::{CheckpointCheck, Player, Score},
    models::BelongsToScene,
    plugins::{
        camera_controls::CameraControls, delete_manager::DeleteMe, level_loader::LevelLoader,
        physics::PhysicsManager, player_manager::PlayerManager, trick_manager::TrickManager,
    },
    service::{
        constants::Constants,
        user_stats::{self, LevelResult},
    },
};

use super::Scene;

fn update_level(
    time: Res<Time>,
    mut scene_state: ResMut<NextState<Scene>>,
    keyboard_input: Res<Input<KeyCode>>,
    mut level_stopwatch: ResMut<LevelStopwatch>,
) {
    level_stopwatch.0.tick(time.delta());
    if keyboard_input.just_pressed(KeyCode::Space) {
        println!("Space pressed");
        scene_state.set(Scene::Home);
    }
}

fn hit_checkmark(
    constants: Res<Constants>,
    player_query: Query<(&Transform, &Collider), With<Player>>,
    checkmark_query: Query<(&RayCaster, &RayHits), With<CheckpointCheck>>,
    mut scene_state: ResMut<NextState<LevelState>>,
) {
    if let Some((ray, hits)) = checkmark_query.iter().next() {
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
            println!("HIT CHECKMARK");
            scene_state.set(LevelState::Over);
        });
    }
}

fn cleanup(
    mut commands: Commands,
    belongs_to_scene_query: Query<(Entity, &BelongsToScene)>,
    mut level_state: ResMut<NextState<LevelState>>,
    current_scene: Res<State<Scene>>,
) {
    level_state.set(LevelState::PrePlay);
    belongs_to_scene_query
        .iter()
        .for_each(|(entity, owned_by_scene)| {
            if owned_by_scene.0 != **current_scene {
                commands.entity(entity).insert(DeleteMe {});
            }
        });
}

#[derive(States, Default, Clone, Copy, Eq, PartialEq, Hash, Debug, Serialize, Deserialize)]
enum LevelState {
    #[default]
    PrePlay,
    Playing,
    Over,
}

#[derive(Resource, Clone, Debug)]
pub struct LevelStateResource(Option<LevelState>);

#[derive(Resource, Clone, Debug)]
pub struct EndLevelTimer(Timer);

fn handle_enter_post_game(
    constants: Res<Constants>,
    mut commands: Commands,
    mut level_result: ResMut<LevelResult>,
    level_stopwatch: ResMut<LevelStopwatch>,
    player_query: Query<&Score, With<Score>>,
) {
    let player_score = player_query
        .iter()
        .next()
        .and_then(|score| Some(score.0))
        .unwrap_or(0);
    level_result.time = level_stopwatch.0.elapsed().as_millis() as usize;
    level_result.score = player_score;
    println!("Level Result: {:?}", level_result);
    user_stats::record_level_result_to_user_stats(level_result.clone());
    commands.insert_resource(EndLevelTimer(Timer::new(
        Duration::from_secs(constants.post_level_secs),
        bevy::time::TimerMode::Once,
    )));
}

fn handle_post_game_update(
    time: Res<Time>,
    level_result: ResMut<LevelResult>,
    mut scene_state: ResMut<NextState<Scene>>,
    mut level_state: ResMut<NextState<LevelState>>,
    mut end_level_timer: ResMut<EndLevelTimer>,
) {
    end_level_timer.0.tick(time.delta());
    if end_level_timer.0.finished() {
        println!("level result 2: {:?}", level_result);
        scene_state.set(Scene::Home);
    }
}

#[derive(Component)]
struct PointsText;

#[derive(Resource, Clone, Debug)]
pub struct LevelStopwatch(Stopwatch);

fn initialize_gui(mut commands: Commands, mut level_state: ResMut<NextState<LevelState>>,  asset_server: Res<AssetServer>) {
    // Text with multiple sections
    let font = asset_server.load("PixelifySans-VariableFont_wght.ttf");
    commands.spawn((
        BelongsToScene(Scene::Level),
        // Create a TextBundle that has a Text with a list of sections.
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle {
                    // This font is loaded and will be used instead of the default font.
                    font: font.clone(),
                    font_size: 60.0,
                    ..default()
                },
            ),
            TextSection::new(
                "0",
                TextStyle {
                    font: font,
                    font_size: 60.0,
                    color: Color::GOLD,
                },
            ),
        ]),
        PointsText,
    ));
}

/**
 * Create post game gui
 * Definition: this system will run once when the level enters its "Over" state
 * When this function runs, we want to create two pieces of text in the center of the screen
 * 1: The players score
 * 2: The players time in "Time: {MM}:{SS}" format
 */
fn create_post_game_gui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    level_result: Res<LevelResult>,
    windows: Query<&Window, With<PrimaryWindow>>,
    player_query: Query<&Score, With<Player>>,
) {
    let window = windows
        .get_single()
        .ok()
        .map(|window| (window.width(), window.height()))
        .unwrap_or((0., 0.));
    let font = asset_server.load("PixelifySans-VariableFont_wght.ttf");
    println!("mid height: {}", window.1);
    println!("mid width: {}", window.0);
    // score text
    commands.spawn((
        BelongsToScene(Scene::Level),
        // Create a TextBundle that has a Text with a list of sections.
        TextBundle {
            text: Text::from_sections([
                TextSection::new(
                    "Score: ",
                    TextStyle {
                        // This font is loaded and will be used instead of the default font.
                        font: font.clone(),
                        font_size: 60.0,
                        ..default()
                    },
                ),
                TextSection::new(
                    &level_result.score.to_string(),
                    TextStyle {
                        font: font.clone(),
                        font_size: 60.0,
                        color: Color::GOLD,
                    },
                ),
            ]),
            style: Style {
                position_type: PositionType::Absolute,
                right: Val::Px(window.0 / 1.63),
                top: Val::Px(window.1 / 1.63),
                ..Default::default()
            },
            ..Default::default()
        },
    ));
    // time text
    let seconds = level_result.time / 1000;
    let minutes = seconds / 60;
    let remainder_seconds = seconds - (minutes * 60);
    let mut seconds_text = remainder_seconds.to_string();
    if seconds_text.len() == 1 {
        seconds_text = format!("0{}", seconds_text);
    }
    let mut minutes_text = minutes.to_string();
    if minutes_text.len() == 1 {
        minutes_text = format!("0{}", minutes_text);
    }
    if minutes_text.len() < 1 {
        minutes_text = "00".to_string();
    }

    commands.spawn((
        BelongsToScene(Scene::Level),
        // Create a TextBundle that has a Text with a list of sections.
        TextBundle {
            text: Text::from_sections([
                TextSection::new(
                    "Time: ",
                    TextStyle {
                        // This font is loaded and will be used instead of the default font.
                        font: font.clone(),
                        font_size: 60.0,
                        ..default()
                    },
                ),
                TextSection::new(
                    (minutes_text + ":" + &seconds_text).as_str(),
                    TextStyle {
                        font: font.clone(),
                        font_size: 60.0,
                        color: Color::GOLD,
                    },
                ),
            ]),
            style: Style {
                position_type: PositionType::Absolute,
                right: Val::Px(window.0 / 1.63),
                top: Val::Px(window.1 / 1.63 - 100.),
                ..Default::default()
            },
            ..Default::default()
        },
    ));
}

fn update_gui(
    mut text_query: Query<(&mut Transform, &mut Text), With<PointsText>>,
    player_query: Query<&Score, With<Score>>,
) {
    player_query.iter().next().and_then(|score| {
        text_query
            .iter_mut()
            .next()
            .and_then(|(mut transform, mut text)| {
                text.sections[1].value = score.0.to_string();
                transform.translation.x = 0.0;
                transform.translation.y = 0.0;
                Some(())
            });
        Some(())
    });
}

pub struct LevelScene;
impl Plugin for LevelScene {
    fn build(&self, app: &mut App) {
        app.add_state::<LevelState>();
        app.insert_resource(LevelStopwatch(Stopwatch::new()));
        app.add_systems(OnEnter(Scene::Level), initialize_gui);
        app.insert_resource(LevelResult {
            level_id: "".to_string(),
            score: 0,
            time: 0,
        });
        app.add_plugins((
            CameraControls {
                startup: OnEnter(Scene::Level).intern(),
                scene: Scene::Level,
            },
            LevelLoader {
                startup: OnEnter(Scene::Level).intern(),
            },
            PhysicsManager {
                startup: OnEnter(Scene::Level).intern(),
            },
            TrickManager {
                scene: Scene::Level,
            },
            PlayerManager {
                scene: Scene::Level,
            },
        ));
        app.add_systems(
            Update,
            (update_level, update_gui).run_if(in_state(Scene::Level)),
        );
        app.add_systems(
            Update,
            (hit_checkmark)
                .run_if(in_state(Scene::Level))
                .run_if(in_state(LevelState::PrePlay)),
        );

        app.add_systems(
            OnEnter(LevelState::Over),
            (handle_enter_post_game, create_post_game_gui).chain(),
        );
        app.add_systems(
            Update,
            (handle_post_game_update)
                .run_if(in_state(Scene::Level))
                .run_if(in_state(LevelState::Over)),
        );
        app.add_systems(OnExit(Scene::Level), cleanup);
    }
}
