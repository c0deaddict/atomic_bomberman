use super::Animation;
use bevy::prelude::*;
use bevy::ecs::bundle::Bundle;

#[derive(Default)]
pub struct AnimatedSpritePlugin;

impl Plugin for AnimatedSpritePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(animate_sprite.system());
    }
}

#[derive(Bundle)]
pub struct AnimatedSprite {
    #[bundle]
    pub sprite_sheet: SpriteSheetBundle,

    pub timer: Timer,
    pub animation: Handle<Animation>,
    pub index: usize,
    // TODO: playing: bool
    // TODO: speed: float (fps?)
    // TODO: fire event when done?
}

// TODO: option to attach multiple spritesheetbundles
impl AnimatedSprite {
    pub fn new(animation: Handle<Animation>, animation_assets: &Assets<Animation>) -> AnimatedSprite {
        let asset = animation_assets.get(&animation).unwrap();
        AnimatedSprite {
            sprite_sheet: SpriteSheetBundle {
                texture_atlas: asset.atlas.texture.clone(),
                sprite: TextureAtlasSprite::new(asset.frames[0].index as u32),
                ..Default::default()
            },
            // TODO: only add timer if animation.frames().len > 1
            timer: Timer::from_seconds(1. / 25., true),
            animation,
            index: 0,
        }
    }
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&mut Timer, &mut TextureAtlasSprite, &mut AnimatedSprite)>,
    animation_assets: Res<Assets<Animation>>,
) {
    for (mut timer, mut sprite, mut animate_sprite) in query.iter_mut() {
        let animation = animation_assets.get(&animate_sprite.animation).unwrap();
        timer.tick(time.delta());
        if timer.finished() {
            animate_sprite.index = (animate_sprite.index + 1) % animation.frames.len();
            sprite.index = animation.frames[animate_sprite.index].index as u32;
        }
    }
}
