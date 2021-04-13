use crate::{animation::*, asset_loaders::*, grid::*, state::*};
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Default)]
pub struct BombPlugin;

impl Plugin for BombPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<PlaceBombEvent>().add_system_set(
            SystemSet::on_update(AppState::Game)
                .with_system(place_bomb.system())
                .with_system(trigger_bomb.system()),
        );
    }
}

pub struct Bomb;
pub struct PlaceBombEvent(pub Position);

fn place_bomb(
    mut commands: Commands,
    named_assets: Res<NamedAssets>,
    animation_assets: Res<Assets<Animation>>,
    mut place_bomb_events: EventReader<PlaceBombEvent>,
    query: Query<&Position, With<Bomb>>,
    audio: Res<Audio>,
) {
    for event in place_bomb_events.iter() {
        if !query.iter().any(|&p| p == event.0) {
            let animation = named_assets.animations.get("bomb regular green").unwrap();
            commands
                .spawn_bundle(AnimatedSpriteBundle::new(
                    animation.clone(),
                    &animation_assets,
                    Default::default(),
                ))
                .insert(Bomb)
                .insert(event.0)
                .insert(Offset(Vec3::new(24.0, -20.0, 25.0)))
                .insert(SnapToGrid)
                .insert(Timer::from_seconds(3.0, false));

            let handle = named_assets.sounds.get("bmdrop2").unwrap();
            audio.play(handle.clone());
        }
    }
}

fn trigger_bomb(
    mut commands: Commands,
    named_assets: Res<NamedAssets>,
    animation_assets: Res<Assets<Animation>>,
    time: Res<Time>,
    mut query: Query<(Entity, &Position, &mut Timer), With<Bomb>>,
    grid: Query<(&Position, Entity, &Cell)>,
    audio: Res<Audio>,
) {
    let grid: HashMap<&Position, (Entity, &Cell)> = grid
        .iter()
        .map(|(pos, entity, cell)| (pos, (entity, cell)))
        .collect();

    for (entity, pos, mut timer) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.finished() {
            commands.entity(entity).despawn_recursive();

            let handle = named_assets.sounds.get("explode2").unwrap();
            audio.play(handle.clone());

            // TODO: TriggerBombEvent
            // TODO: let flame plugin listen for this event?
        }
    }
}
