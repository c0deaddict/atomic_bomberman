use crate::asset_loaders::{Animation, AnimationBundle};
use bevy::prelude::*;

#[derive(Default)]
pub struct AnimatedSpritePlugin;

impl Plugin for AnimatedSpritePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(animate_sprite.system());
    }
}

pub struct AnimatedSprite {
    // TODO: Handle<Animation>?
    pub animation: Animation,
    pub index: usize,
    // TODO: playing: bool
    // TODO: speed: float (fps?)
    // TODO: fire event when done?
}

// TODO: hack because bundle can't be extended.
// Track: https://github.com/bevyengine/bevy/discussions/1217
impl AnimatedSprite {
    pub fn spawn<'a>(
        commands: &'a mut Commands,
        bundle: &AnimationBundle,
        animation: &Animation,
    ) -> &'a Commands {
        commands
            .spawn(SpriteSheetBundle {
                texture_atlas: bundle.texture_atlas.clone(),
                sprite: TextureAtlasSprite::new(animation.frames[0].index as u32),
                ..Default::default()
            })
            .with(AnimatedSprite {
                animation: animation.clone(),
                index: 0,
            })
            // TODO: only add timer if animation.frames().len > 1
            .with(Timer::from_seconds(1. / 25., true))
    }

    pub fn spawn_child<'a>(
        commands: &'a mut ChildBuilder,
        bundle: &AnimationBundle,
        animation: &Animation,
    ) -> &'a ChildBuilder<'a> {
        commands
            .spawn(SpriteSheetBundle {
                texture_atlas: bundle.texture_atlas.clone(),
                sprite: TextureAtlasSprite::new(animation.frames[0].index as u32),
                ..Default::default()
            })
            .with(AnimatedSprite {
                animation: animation.clone(),
                index: 0,
            })
            // TODO: only add timer if animation.frames().len > 1
            .with(Timer::from_seconds(1. / 25., true))
    }
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&mut Timer, &mut TextureAtlasSprite, &mut AnimatedSprite)>,
) {
    for (mut timer, mut sprite, mut animate_sprite) in query.iter_mut() {
        timer.tick(time.delta_seconds());
        if timer.finished() {
            animate_sprite.index =
                (animate_sprite.index + 1) % animate_sprite.animation.frames.len();
            sprite.index = animate_sprite.animation.frames[animate_sprite.index].index as u32;
        }
    }
}
