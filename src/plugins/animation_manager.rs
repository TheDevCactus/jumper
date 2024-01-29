use bevy::{
    app::{App, Plugin, Update},
    ecs::{
        component::Component,
        system::{Query, Res},
    },
    sprite::TextureAtlasSprite,
    time::{Time, Timer, TimerMode},
};

#[derive(Component)]
pub struct LoopingIncrementer {
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
pub struct AnimationTimer(Timer);

pub struct SpriteAnimationController;
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

pub struct AnimationManager;
impl Plugin for AnimationManager {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, update_animated_sprites);
    }
}
