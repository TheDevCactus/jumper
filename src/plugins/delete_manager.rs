use bevy::{
    app::{App, Plugin, PostUpdate},
    ecs::{
        component::Component,
        entity::Entity,
        query::With,
        system::{Commands, Query},
    },
};

#[derive(Component)]
pub struct DeleteMe;

fn delete_me(mut commands: Commands, query: Query<Entity, With<DeleteMe>>) {
    query.iter().for_each(|entity| {
        commands.entity(entity).despawn();
    });
}
pub struct DeleteManager;
impl Plugin for DeleteManager {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, delete_me);
    }
}
