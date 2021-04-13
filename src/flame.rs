use crate::{animation::*, asset_loaders::*, grid::Position, state::*};
use bevy::{input::keyboard::KeyboardInput, prelude::*};
use std::cmp::{Eq, PartialEq};
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;

#[derive(Default)]
pub struct FlamePlugin;

impl Plugin for FlamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<StartFlameEvent>().add_system_set(
            SystemSet::on_update(AppState::Game)
                .with_system(trigger_flame.system())
                .with_system(flame_out.system()),
        );
    }
}

pub struct Flame;
pub struct StartFlameEvent(Position);

fn trigger_flame(
    mut commands: Commands,
    named_assets: Res<NamedAssets>,
    animation_assets: Res<Assets<Animation>>,
    mut start_flame_events: EventReader<StartFlameEvent>,
    grid: Query<(&Position, Entity, &Cell)>,
) {
    let grid: HashMap<&Position, (Entity, &Cell)> = grid
        .iter()
        .map(|(pos, entity, cell)| (pos, (entity, cell)))
        .collect();

    for event in start_flame_events.iter() {
        let center = named_assets.animations.get("flame center green").unwrap();
        let midwest = named_assets.animations.get("flame midwest green").unwrap();
        let tipwest = named_assets.animations.get("flame tipwest green").unwrap();
        let mideast = named_assets.animations.get("flame mideast green").unwrap();
        let tipeast = named_assets.animations.get("flame tipeast green").unwrap();
        let midnorth = named_assets.animations.get("flame midnorth green").unwrap();
        let tipnorth = named_assets.animations.get("flame tipnorth green").unwrap();
        let midsouth = named_assets.animations.get("flame midsouth green").unwrap();
        let tipsouth = named_assets.animations.get("flame tipsouth green").unwrap();

        let flame = commands
            .spawn_bundle((
                GlobalTransform::default(),
                Transform::from_translation(pos_to_vec(&pos) + Vec3::new(21., -19., 0.)),
            ))
            .insert(Flame)
            .insert(Timer::from_seconds(5.0, false))
            .with_children(|parent| {
                parent.spawn_bundle(AnimatedSpriteBundle::new(
                    center.clone(),
                    &animation_assets,
                    Default::default(),
                ));
            })
            .id();

        let strength = 3;

        // TODO: sync animation of entire flame.
        // TODO: first plot path of flame, then turn into animations
        // TODO: brick becomes blank.
        // TODO: trigger other bombs in path
        // TODO: kill in path

        for x in ((pos.x - (strength - 1))..pos.x).rev() {
            if x < 0 {
                break;
            }

            let p = Pos { x, y: pos.y };
            match grid.get(&p) {
                Some((entity, Cell::Solid)) => {
                    break;
                }
                Some((entity, Cell::Brick)) => {
                    commands.entity(*entity).despawn_recursive();
                    break;
                }
                _ => {}
            }

            let part = commands
                .spawn_bundle(AnimatedSpriteBundle::new(
                    midwest.clone(),
                    &animation_assets,
                    Transform::from_translation(Vec3::new((x - pos.x) as f32 * 40., -6., 0.)),
                ))
                .id();

            commands.entity(flame).push_children(&[part]);
        }

        for x in (pos.x + 1)..(pos.x + strength) {
            if x >= 15 {
                break;
            }

            let p = Pos { x, y: pos.y };
            match grid.get(&p) {
                Some((entity, Cell::Solid)) => {
                    break;
                }
                Some((entity, Cell::Brick)) => {
                    commands.entity(*entity).despawn_recursive();
                    break;
                }
                _ => {}
            }

            let part = commands
                .spawn_bundle(AnimatedSpriteBundle::new(
                    mideast.clone(),
                    &animation_assets,
                    Transform::from_translation(Vec3::new((x - pos.x) as f32 * 40.0, -4., 0.)),
                ))
                .id();

            commands.entity(flame).push_children(&[part]);
        }

        for y in ((pos.y - (strength - 1))..pos.y).rev() {
            if y < 0 {
                break;
            }

            let p = Pos { x: pos.x, y };
            match grid.get(&p) {
                Some((entity, Cell::Solid)) => {
                    break;
                }
                Some((entity, Cell::Brick)) => {
                    commands.entity(*entity).despawn_recursive();
                    break;
                }
                _ => {}
            }

            let part = commands
                .spawn_bundle((
                    Transform::from_translation(Vec3::new(6., (y - pos.y) as f32 * 32., 0.)),
                    GlobalTransform::default(),
                ))
                .with_children(|parent| {
                    parent.spawn_bundle(AnimatedSpriteBundle::new(
                        midnorth.clone(),
                        &animation_assets,
                        Default::default(),
                    ));
                })
                .id();

            commands.entity(flame).push_children(&[part]);
        }

        for y in (pos.y + 1)..(pos.y + strength) {
            if y >= 11 {
                break;
            }

            let p = Pos { x: pos.x, y };
            match grid.get(&p) {
                Some((entity, Cell::Solid)) => {
                    break;
                }
                Some((entity, Cell::Brick)) => {
                    commands.entity(*entity).despawn_recursive();
                    break;
                }
                _ => {}
            }

            let part = commands
                .spawn_bundle((
                    Transform::from_translation(Vec3::new(6., (y - pos.y) as f32 * 32.0, 0.)),
                    GlobalTransform::default(),
                ))
                .with_children(|parent| {
                    parent.spawn_bundle(AnimatedSpriteBundle::new(
                        midsouth.clone(),
                        &animation_assets,
                        Default::default(),
                    ));
                })
                .id();

            commands.entity(flame).push_children(&[part]);
        }
    }
}

fn flame_out(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Timer), With<Flame>>,
) {
    for (entity, mut timer) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.finished() {
            commands.entity(entity).despawn_recursive();
        }
    }
}
