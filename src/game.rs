use crate::{animation::*, asset_loaders::*, state::*};
use bevy::{input::keyboard::KeyboardInput, prelude::*};
use std::cmp::{Eq, PartialEq};
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Default)]
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_plugin(AnimatedSpritePlugin)
            .add_event::<PlaceBombEvent>()
            .on_state_enter(STAGE, AppState::Game, setup.system())
            .on_state_update(STAGE, AppState::Game, keyboard_handling.system())
            .on_state_update(STAGE, AppState::Game, player_movement.system())
            .on_state_update(STAGE, AppState::Game, place_bomb.system())
            .on_state_update(STAGE, AppState::Game, trigger_bomb.system())
            .on_state_update(STAGE, AppState::Game, flame_out.system())
            .on_state_update(STAGE, AppState::Game, change_sprite.system())
            .on_state_exit(STAGE, AppState::Game, cleanup.system());
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Direction {
    North,
    South,
    East,
    West,
}

struct Player;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
struct Pos {
    x: i8,
    y: i8,
}

struct Bomb;
struct Flame;

#[derive(Debug)]
struct PlayerDirection {
    direction: Direction,
    walking: bool,
}

#[derive(Default)]
struct KeyboardMovement(Vec<Direction>);

impl PlayerDirection {
    fn animation(&self) -> String {
        let dir = match self.direction {
            Direction::North => "north",
            Direction::East => "east",
            Direction::South => "south",
            Direction::West => "west",
        };
        let ani = match self.walking {
            true => "walk",
            false => "stand",
        };
        format!("{} {}", ani, dir)
    }
}

struct PlaceBombEvent(Pos);

fn pos_to_vec(pos: &Pos) -> Vec3 {
    Vec3::new(20.0 + pos.x as f32 * 40.0, -64.0 - pos.y as f32 * 36.0, 0.0)
}

fn setup(
    commands: &mut Commands,
    asset_server: Res<AssetServer>,
    // audio: Res<Audio>,
    named_assets: Res<NamedAssets>,
    animation_assets: Res<Assets<Animation>>,
    scheme_assets: Res<Assets<Scheme>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // player: STAND.ANI (four perspectives)
    //   WALK.ANI for walking

    let background_handle = asset_server.load("data/RES/FIELD0.PCX");
    commands.spawn(SpriteBundle {
        material: materials.add(background_handle.into()),
        transform: Transform::from_translation(Vec3::new(320., -240., 0.)),
        ..Default::default()
    });

    // let music = asset_server.load("data/SOUND/MENU.RSS");
    // audio.play(music);

    let player_direction = PlayerDirection {
        direction: Direction::North,
        walking: false,
    };
    let animation = named_assets
        .animations
        .get(&player_direction.animation())
        .unwrap();

    commands
        .spawn((
            Transform::from_translation(Vec3::new(320., -240., 1.0)),
            GlobalTransform::default(),
        ))
        .with(Player)
        .with(player_direction)
        .with(KeyboardMovement::default())
        .with(Pos { x: 0, y: 0 })
        .with(Timer::from_seconds(1. / 60., true))
        .with_children(|parent| {
            let shadow = named_assets.animations.get("shadow").unwrap();
            // TODO: make a bundle or something for this. this is ugly.
            parent
                .spawn((
                    Transform::from_translation(Vec3::new(0.0, -35.0, 0.0)),
                    GlobalTransform::default(),
                ))
                .with_children(|parent| {
                    AnimatedSprite::spawn_child(parent, shadow.clone(), &animation_assets);
                });
            AnimatedSprite::spawn_child(parent, animation.clone(), &animation_assets);
        });

    let scheme = scheme_assets
        .get(named_assets.schemes.get("X MARKS THE SPOT (10)").unwrap())
        .unwrap();

    let brick = named_assets.animations.get("tile 0 brick").unwrap();
    let solid = named_assets.animations.get("tile 0 solid").unwrap();
    let blank = named_assets.animations.get("tile 0 blank").unwrap();

    for x in 0..15 {
        for y in 0..11 {
            let cell = scheme.grid[y][x];
            let animation = match cell {
                Cell::Solid => solid,
                Cell::Brick => brick,
                Cell::Blank => blank,
            };

            let pos = Pos {
                x: x as i8,
                y: y as i8,
            };

            commands
                .spawn((
                    Transform::from_translation(pos_to_vec(&pos) + Vec3::new(20.0, -18.0, 0.0)),
                    GlobalTransform::default(),
                ))
                .with(cell)
                .with(pos)
                .with_children(|parent| {
                    AnimatedSprite::spawn_child(parent, animation.clone(), &animation_assets);
                });
        }
    }
}

fn key_to_direction(key_code: &KeyCode) -> Option<Direction> {
    match key_code {
        KeyCode::A => Some(Direction::West),
        KeyCode::D => Some(Direction::East),
        KeyCode::S => Some(Direction::South),
        KeyCode::W => Some(Direction::North),
        _ => None,
    }
}

fn keyboard_handling(
    mut keyboard_events: EventReader<KeyboardInput>,
    mut query: Query<(&mut PlayerDirection, &mut KeyboardMovement, &Pos)>,
    mut place_bomb_events: ResMut<Events<PlaceBombEvent>>,
) {
    // TODO: need to remember which keys are pressed.
    // maybe the naive input handling is enough?
    for (mut player_direction, mut keyboard_movement, pos) in query.iter_mut() {
        for event in keyboard_events.iter() {
            if let Some(key_code) = event.key_code {
                if let Some(direction) = key_to_direction(&key_code) {
                    if event.state.is_pressed() {
                        keyboard_movement.0.push(direction);
                    } else {
                        keyboard_movement.0.retain(|&x| x != direction);
                    }

                    if let Some(new_direction) = keyboard_movement.0.last() {
                        if &player_direction.direction != new_direction {
                            player_direction.direction = *new_direction;
                        }
                        if !player_direction.walking {
                            player_direction.walking = true;
                        }
                    } else if player_direction.walking {
                        player_direction.walking = false;
                    }
                } else if event.state.is_pressed() && key_code == KeyCode::Space {
                    place_bomb_events.send(PlaceBombEvent(pos.clone()));
                }
            }
        }
    }
}

fn player_movement(
    time: Res<Time>,
    mut query: Query<(&mut Timer, &mut Transform, &mut Pos, &PlayerDirection)>,
    grid: Query<(&Cell, &Pos)>,
) {
    for (mut timer, mut transform, mut pos, player_direction) in query.iter_mut() {
        timer.tick(time.delta_seconds());
        if timer.finished() && player_direction.walking {
            let mut new_translation = transform.translation;
            match player_direction.direction {
                Direction::West => new_translation.x -= 2.,
                Direction::East => new_translation.x += 2.,
                Direction::South => new_translation.y -= 2.,
                Direction::North => new_translation.y += 2.,
            };

            let new_pos = Pos {
                x: ((new_translation.x - 20.) / 40.) as i8,
                y: ((new_translation.y - 32.0 - -64.0) / -36.) as i8,
            };

            let collision = grid
                .iter()
                .any(|(cell, pos)| cell != &Cell::Blank && pos == &new_pos);

            if !collision {
                transform.translation.x = new_translation.x;
                transform.translation.y = new_translation.y;
                pos.x = new_pos.x;
                pos.y = new_pos.y;
            }
        }
    }
}

fn place_bomb(
    commands: &mut Commands,
    named_assets: Res<NamedAssets>,
    animation_assets: Res<Assets<Animation>>,
    mut place_bomb_events: EventReader<PlaceBombEvent>,
    query: Query<&Pos, With<Bomb>>,
    audio: Res<Audio>,
) {
    for event in place_bomb_events.iter() {
        if !query.iter().any(|&p| p == event.0) {
            let animation = named_assets.animations.get("bomb regular green").unwrap();
            commands
                .spawn((
                    Transform::from_translation(pos_to_vec(&event.0) + Vec3::new(24., -20., 0.)),
                    GlobalTransform::default(),
                ))
                .with(Bomb)
                .with(event.0)
                .with(Timer::from_seconds(3.0, false))
                .with_children(|parent| {
                    AnimatedSprite::spawn_child(parent, animation.clone(), &animation_assets);
                });

            let handle = named_assets.sounds.get("bmdrop2").unwrap();
            audio.play(handle.clone());
        }
    }
}

fn trigger_bomb(
    commands: &mut Commands,
    named_assets: Res<NamedAssets>,
    animation_assets: Res<Assets<Animation>>,
    time: Res<Time>,
    mut query: Query<(Entity, &Pos, &mut Timer), With<Bomb>>,
    grid: Query<(&Pos, Entity, &Cell)>,
    audio: Res<Audio>,
) {
    let grid: HashMap<&Pos, (Entity, &Cell)> = grid
        .iter()
        .map(|(pos, entity, cell)| (pos, (entity, cell)))
        .collect();

    for (entity, pos, mut timer) in query.iter_mut() {
        timer.tick(time.delta_seconds());
        if timer.finished() {
            commands.despawn_recursive(entity);

            let handle = named_assets.sounds.get("explode2").unwrap();
            audio.play(handle.clone());

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
                .spawn((
                    Transform::from_translation(pos_to_vec(&pos) + Vec3::new(21., -19., 0.)),
                    GlobalTransform::default(),
                ))
                .with(Flame)
                .with(Timer::from_seconds(0.5, false))
                .with_children(|parent| {
                    AnimatedSprite::spawn_child(parent, center.clone(), &animation_assets);
                })
                .current_entity()
                .unwrap();

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
                        commands.despawn_recursive(*entity);
                        break;
                    }
                    _ => {}
                }

                let part = commands
                    .spawn((
                        Transform::from_translation(Vec3::new((x - pos.x) as f32 * 40., -6., 0.)),
                        GlobalTransform::default(),
                    ))
                    .with_children(|parent| {
                        AnimatedSprite::spawn_child(parent, midwest.clone(), &animation_assets);
                    })
                    .current_entity()
                    .unwrap();

                commands.push_children(flame, &[part]);
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
                        commands.despawn_recursive(*entity);
                        break;
                    }
                    _ => {}
                }

                let part = commands
                    .spawn((
                        Transform::from_translation(Vec3::new((x - pos.x) as f32 * 40.0, -4., 0.)),
                        GlobalTransform::default(),
                    ))
                    .with_children(|parent| {
                        AnimatedSprite::spawn_child(parent, mideast.clone(), &animation_assets);
                    })
                    .current_entity()
                    .unwrap();

                commands.push_children(flame, &[part]);
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
                        commands.despawn_recursive(*entity);
                        break;
                    }
                    _ => {}
                }

                let part = commands
                    .spawn((
                        Transform::from_translation(Vec3::new(6., (y - pos.y) as f32 * 32., 0.)),
                        GlobalTransform::default(),
                    ))
                    .with_children(|parent| {
                        AnimatedSprite::spawn_child(parent, midnorth.clone(), &animation_assets);
                    })
                    .current_entity()
                    .unwrap();

                commands.push_children(flame, &[part]);
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
                        commands.despawn_recursive(*entity);
                        break;
                    }
                    _ => {}
                }

                let part = commands
                    .spawn((
                        Transform::from_translation(Vec3::new(6., (y - pos.y) as f32 * 32.0, 0.)),
                        GlobalTransform::default(),
                    ))
                    .with_children(|parent| {
                        AnimatedSprite::spawn_child(parent, midsouth.clone(), &animation_assets);
                    })
                    .current_entity()
                    .unwrap();

                commands.push_children(flame, &[part]);
            }
        }
    }
}

fn flame_out(
    commands: &mut Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut Timer), With<Flame>>,
) {
    for (entity, mut timer) in query.iter_mut() {
        timer.tick(time.delta_seconds());
        if timer.finished() {
            commands.despawn_recursive(entity);
        }
    }
}

// NOTE: systems with Changed<> must be added AFTER the system that triggers the change.
fn change_sprite(
    commands: &mut Commands,
    named_assets: Res<NamedAssets>,
    animation_assets: Res<Assets<Animation>>,
    mut query: Query<(Entity, &mut Children, &PlayerDirection), Changed<PlayerDirection>>,
) {
    for (entity, children, player_direction) in query.iter_mut() {
        if let Some(child) = children.last().copied() {
            commands.despawn_recursive(child);
        }

        let animation = named_assets
            .animations
            .get(&player_direction.animation())
            .unwrap();

        let child = AnimatedSprite::spawn(commands, animation.clone(), &animation_assets)
            .current_entity()
            .unwrap();

        commands.push_children(entity, &[child]);
    }
}

fn cleanup() {
    info!("cleanup");
}
