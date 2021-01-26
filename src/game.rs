use crate::{animation::*, asset_loaders::*, state::*};
use bevy::{input::keyboard::KeyboardInput, prelude::*};
use std::cmp::{Eq, PartialEq};
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

struct Player {}

#[derive(Clone, PartialEq, Eq, Debug)]
struct Pos {
    x: u8,
    y: u8,
}

struct Bombs(Vec<Pos>);

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
        .with(Player {})
        .with(player_direction)
        .with(KeyboardMovement::default())
        .with(Pos { x: 0, y: 0 })
        .with(Bombs(vec![]))
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
            let tx = 20.0 + x as f32 * 40.0;
            let ty = -64.0 - y as f32 * 36.0;

            let cell = scheme.grid[y][x];
            let animation = match cell {
                Cell::Solid => solid,
                Cell::Brick => brick,
                Cell::Blank => blank,
            };

            commands
                .spawn((
                    Transform::from_translation(Vec3::new(tx + 20.0, ty - 18.0, 0.0)),
                    GlobalTransform::default(),
                ))
                .with(cell)
                .with(Pos {
                    x: x as u8,
                    y: y as u8,
                })
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
                x: ((new_translation.x - 20.) / 40.) as u8,
                y: ((new_translation.y - 32.0 - -64.0) / -36.) as u8,
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
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&Pos, &mut Bombs)>,
    audio: Res<Audio>,
) {
    for (pos, mut bombs) in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::Space) {
            if !bombs.0.contains(&pos) {
                let animation = named_assets.animations.get("bomb regular green").unwrap();
                let x = 20.0 + pos.x as f32 * 40.0;
                let y = -64.0 + pos.y as f32 * -36.0;
                commands
                    .spawn((
                        Transform::from_translation(Vec3::new(x + 24., y - 20., 0.0)),
                        GlobalTransform::default(),
                    ))
                    .with_children(|parent| {
                        AnimatedSprite::spawn_child(parent, animation.clone(), &animation_assets);
                    });

                let handle = named_assets.sounds.get("bmdrop2").unwrap();
                audio.play(handle.clone());

                bombs.0.push(pos.clone());
            }
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
