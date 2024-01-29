use bevy::{
    app::{App, Plugin, PostStartup, PreStartup, Update},
    core_pipeline::core_2d::{Camera2d, Camera2dBundle},
    ecs::{
        query::{With, Without},
        system::{Commands, Query},
    },
    render::camera::OrthographicProjection,
    transform::components::Transform,
    utils::default,
};

use crate::Player;

// inserts a camera bundle into our app
fn insert_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle { ..default() });
}

// initializes the cameras settings
fn adjust_camera(mut camera_query: Query<&mut OrthographicProjection, With<Camera2d>>) {
    camera_query.iter_mut().next().map(|mut projection| {
        projection.scale /= 2.5;
        ()
    });
}

// Updates the camera position to the players position
fn follow_player(
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    player_query: Query<&Transform, (With<Player>, Without<Camera2d>)>,
) {
    player_query.iter().next().and_then(|player_transform| {
        camera_query
            .iter_mut()
            .next()
            .map(|mut camera_transform| {
                camera_transform.translation.x = player_transform.translation.x;
                camera_transform.translation.y = player_transform.translation.y;
                ()
            })
    });
}

pub struct CameraControls;
impl Plugin for CameraControls {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, insert_camera);
        app.add_systems(PostStartup, adjust_camera);
        app.add_systems(Update, follow_player);
    }
}
