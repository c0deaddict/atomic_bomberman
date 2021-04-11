use super::Animation;
use bevy::ecs::bundle::Bundle;
use bevy::prelude::*;

#[derive(Default)]
pub struct AnimatedSpritePlugin;

struct AnimationTimer(Timer);

impl Plugin for AnimatedSpritePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app
            .insert_resource(AnimationTimer(Timer::from_seconds(1. / 25., true)))
            .add_system(animate_sprite.system());
    }
}

pub struct AnimatedSprite {
    pub animation: Handle<Animation>,
    pub index: usize,
}

#[derive(Bundle)]
pub struct AnimatedSpriteBundle {
    #[bundle]
    pub sprite_sheet: SpriteSheetBundle,

    pub animated_sprite: AnimatedSprite,
    // TODO: playing: bool
    // TODO: speed: float (fps?)
    // TODO: fire event when done?
}

// TODO: option to attach multiple spritesheetbundles
impl AnimatedSpriteBundle {
    pub fn new(
        animation: Handle<Animation>,
        animation_assets: &Assets<Animation>,
    ) -> AnimatedSpriteBundle {
        let asset = animation_assets.get(&animation).unwrap();
        AnimatedSpriteBundle {
            sprite_sheet: SpriteSheetBundle {
                texture_atlas: asset.atlas.texture.clone(),
                sprite: TextureAtlasSprite::new(asset.frames[0].index as u32),
                ..Default::default()
            },
            animated_sprite: AnimatedSprite {
                animation,
                index: 0,
            },
        }
    }
}

fn animate_sprite(
    time: Res<Time>,
    animation_assets: Res<Assets<Animation>>,
    mut timer: ResMut<AnimationTimer>,
    mut query: Query<(&mut TextureAtlasSprite, &mut AnimatedSprite)>,
) {
    timer.0.tick(time.delta());
    if timer.0.finished() {
        for (mut sprite, mut animate_sprite) in query.iter_mut() {
            let animation = animation_assets.get(&animate_sprite.animation).unwrap();
            if animation.frames.len() > 1 {
                animate_sprite.index = (animate_sprite.index + 1) % animation.frames.len();
                sprite.index = animation.frames[animate_sprite.index].index as u32;
            }
        }
    }
}
