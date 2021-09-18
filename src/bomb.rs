use crate::{
    animation::*,
    asset_loaders::*,
    grid::{Direction, *},
    state::*,
};
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Default)]
pub struct BombPlugin;

impl Plugin for BombPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_event::<PlaceBombEvent>().add_system_set(
            SystemSet::on_update(AppState::Game)
                .with_system(place_bomb.system())
                .with_system(trigger_bomb.system())
                .with_system(flame_out.system()),
        );
    }
}

pub struct Flame;
pub struct Bomb;
pub struct PlaceBombEvent(pub Position);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FlameCell {
    North,
    South,
    East,
    West,
    Center,
}

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
            let asset = animation_assets.get(animation).unwrap();
            let offset = Vec2::new(
                ((40 - asset.width as i32) / 2) as f32,
                ((36 - asset.height as i32) / 2) as f32 * -1.0,
            );
            println!("bomb offset = {:?}", offset);

            commands
                .spawn_bundle(AnimatedSpriteBundle::new(
                    animation.clone(),
                    &animation_assets,
                    Default::default(),
                ))
                .insert(Bomb)
                .insert(event.0)
                .insert(Offset(Vec3::from((offset, 25.0))))
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

    let mut bomb: Option<Position> = None;
    for (entity, pos, mut timer) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.finished() {
            // TODO: move despawn to after flame calc below.
            commands.entity(entity).despawn_recursive();

            bomb = Some(pos.clone());
            break;
        }
    }

    if let Some(pos) = bomb {
        println!("trigger_bomb {:?}", pos);
        let handle = named_assets.sounds.get("explode2").unwrap();
        audio.play(handle.clone());

        let mut flame: HashMap<Position, FlameCell> = HashMap::new();
        let strength = 3;
        trace_flame(pos, strength, grid, &mut flame);

        // TODO: trigger other bombs in flame
        // TODO: keep list of bombs that have already been processed.
        for (_entity, pos, _timer) in query.iter_mut() {
            if flame.contains_key(pos) {
                println!("trigger another bomb at {:?}", pos);
            }
        }

        let center = named_assets.animations.get("flame center green").unwrap();
        let midwest = named_assets.animations.get("flame midwest green").unwrap();
        let tipwest = named_assets.animations.get("flame tipwest green").unwrap();
        let mideast = named_assets.animations.get("flame mideast green").unwrap();
        let tipeast = named_assets.animations.get("flame tipeast green").unwrap();
        let midnorth = named_assets.animations.get("flame midnorth green").unwrap();
        let tipnorth = named_assets.animations.get("flame tipnorth green").unwrap();
        let midsouth = named_assets.animations.get("flame midsouth green").unwrap();
        let tipsouth = named_assets.animations.get("flame tipsouth green").unwrap();

        let asset = animation_assets.get(midwest).unwrap();
        println!("{:?}", asset);

        // TODO: is it possible to spawn an "empty" parent?
        let flame_entity = commands
            .spawn_bundle((Transform::default(), GlobalTransform::default()))
            .insert(Flame)
            .insert(Timer::from_seconds(5.0, false))
            .id();

        for (pos, cell) in flame.iter() {
            let (animation, offset) = match cell {
                FlameCell::North => (midnorth, Vec2::new(7.0, 0.0)),
                FlameCell::East => (midwest, Vec2::new(0.0, -7.0)),
                FlameCell::South => (midsouth, Vec2::new(7.0, 0.0)),
                FlameCell::West => (mideast, Vec2::new(0.0, -7.0)),
                FlameCell::Center => (center, Vec2::new(0.0, 0.0)),
            };

            let part = commands
                .spawn_bundle(AnimatedSpriteBundle::new(
                    animation.clone(),
                    &animation_assets,
                    Default::default(),
                ))
                .insert(*pos)
                .insert(Offset(Vec3::from((offset, 25.0))))
                .insert(SnapToGrid)
                .id();

            commands.entity(flame_entity).push_children(&[part]);
        }
    }
}

fn trace_flame(
    bomb: Position,
    strength: usize,
    grid: HashMap<&Position, (Entity, &Cell)>,
    flame: &mut HashMap<Position, FlameCell>,
) {
    flame.insert(bomb, FlameCell::Center);

    for dir in Direction::iter() {
        for pos in bomb.iter(dir).take(strength) {
            match grid.get(&pos) {
                Some((_entity, Cell::Solid)) => break,
                Some((_entity, Cell::Brick)) => {
                    println!("destroying {:?}", pos);
                    // commands.entity(*entity).despawn_recursive();
                    break;
                }
                _ => {}
            }

            // TODO: could insert dir here into flame as hashmap
            // if pos already contains a dir, then create a "cross" at the point.
            if let Some(cell) = flame.get(&pos) {
                // TODO: should determine if axis is different, eg [south, north] != [east, west]
                if cell.dir() != Some(dir) {
                    flame.insert(pos, FlameCell::Center);
                }
            } else {
                flame.insert(pos, FlameCell::from(dir));
            }
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

impl From<Direction> for FlameCell {
    fn from(dir: Direction) -> Self {
        match dir {
            Direction::North => FlameCell::North,
            Direction::East => FlameCell::East,
            Direction::South => FlameCell::South,
            Direction::West => FlameCell::West,
        }
    }
}

impl FlameCell {
    fn dir(&self) -> Option<Direction> {
        match self {
            FlameCell::North => Some(Direction::North),
            FlameCell::East => Some(Direction::East),
            FlameCell::South => Some(Direction::South),
            FlameCell::West => Some(Direction::West),
            FlameCell::Center => None,
        }
    }
}
