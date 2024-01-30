use bevy::{
    app::{App, Plugin, PostStartup, PreStartup, Update},
    core_pipeline::core_2d::{Camera2d, Camera2dBundle},
    ecs::{
        query::{With, Without},
        schedule::{
            apply_deferred, common_conditions::in_state, IntoSystemConfigs, ScheduleLabel, State,
            SystemConfigs,
        },
        system::{Commands, Query, Res, System},
    },
    render::camera::OrthographicProjection,
    transform::components::Transform,
    utils::{default, intern::Interned},
};

use crate::{components_resources::Player, models::BelongsToScene, scenes::Scene};

// inserts a camera bundle into our app
fn insert_camera(mut commands: Commands, scene: Res<State<Scene>>) {
    commands.spawn((
        Camera2dBundle { ..default() },
        BelongsToScene(scene.clone()),
    ));
}

// initializes the cameras settings
fn adjust_camera(mut camera_query: Query<&mut OrthographicProjection, With<Camera2d>>) {
    if let Some(mut projection) = camera_query.iter_mut().next() {
        projection.scale /= 2.5;
    }
}

// Updates the camera position to the players position
fn follow_player(
    mut camera_query: Query<&mut Transform, With<Camera2d>>,
    player_query: Query<&Transform, (With<Player>, Without<Camera2d>)>,
) {
    player_query.iter().next().and_then(|player_transform| {
        camera_query.iter_mut().next().map(|mut camera_transform| {
            camera_transform.translation.x = player_transform.translation.x;
            camera_transform.translation.y = player_transform.translation.y;
        })
    });
}

pub struct CameraControls {
    pub startup: Interned<dyn ScheduleLabel>,
    pub scene: Scene,
}
impl Plugin for CameraControls {
    fn build(&self, app: &mut App) {
        app.add_systems(
            self.startup,
            (insert_camera, apply_deferred, adjust_camera).chain(),
        );
        app.add_systems(Update, follow_player.run_if(in_state(self.scene)));
    }
}
