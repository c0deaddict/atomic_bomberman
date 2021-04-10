use super::Animation;
use bevy::prelude::*;

#[derive(Default)]
pub struct AnimatedSpritePlugin;

impl Plugin for AnimatedSpritePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system(animate_sprite.system());
    }
}

pub struct AnimatedSprite {
    pub animation: Handle<Animation>,
    pub index: usize,
    // TODO: playing: bool
    // TODO: speed: float (fps?)
    // TODO: fire event when done?
}

// TODO: option to attach multiple spritesheetbundles
// TODO: hack because bundle can't be extended.
// Track: https://github.com/bevyengine/bevy/discussions/1217
// TODO: need to be able to override Transform
impl AnimatedSprite {
    pub fn spawn<'a>(
        commands: &'a mut Commands,
        animation: Handle<Animation>,
        animation_assets: &Assets<Animation>,
    ) -> &'a Commands {
        let asset = animation_assets.get(&animation).unwrap();
        commands
            .spawn(SpriteSheetBundle {
                texture_atlas: asset.atlas.texture.clone(),
                sprite: TextureAtlasSprite::new(asset.frames[0].index as u32),
                ..Default::default()
            })
            .with(AnimatedSprite {
                animation,
                index: 0,
            })
            // TODO: only add timer if animation.frames().len > 1
            .with(Timer::from_seconds(1. / 25., true))
    }

    pub fn spawn_child<'a>(
        commands: &'a mut ChildBuilder,
        animation: Handle<Animation>,
        animation_assets: &Assets<Animation>,
    ) -> &'a ChildBuilder<'a> {
        let asset = animation_assets.get(&animation).unwrap();
        commands
            .spawn(SpriteSheetBundle {
                texture_atlas: asset.atlas.texture.clone(),
                sprite: TextureAtlasSprite::new(asset.frames[0].index as u32),
                ..Default::default()
            })
            .with(AnimatedSprite {
                animation,
                index: 0,
            })
            // TODO: only add timer if animation.frames().len > 1
            .with(Timer::from_seconds(1. / 25., true))
    }
}

fn animate_sprite(
    time: Res<Time>,
    mut query: Query<(&mut Timer, &mut TextureAtlasSprite, &mut AnimatedSprite)>,
    animation_assets: Res<Assets<Animation>>,
) {
    for (mut timer, mut sprite, mut animate_sprite) in query.iter_mut() {
        let animation = animation_assets.get(&animate_sprite.animation).unwrap();
        timer.tick(time.delta_seconds());
        if timer.finished() {
            animate_sprite.index = (animate_sprite.index + 1) % animation.frames.len();
            sprite.index = animation.frames[animate_sprite.index].index as u32;
        }
    }
}
