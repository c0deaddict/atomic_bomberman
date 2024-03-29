use crate::{
    animation::*,
    asset_loaders::*,
    bomb::PlaceBombEvent,
    grid::{Direction, Position},
    state::*,
};
use bevy::{input::keyboard::KeyboardInput, prelude::*};
use std::collections::HashSet;
use std::fmt::Debug;

#[derive(Default)]
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::Game)
                .with_system(keyboard_handling)
                .with_system(player_movement)
                .with_system(change_sprite),
        );
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerSpeed{
    pub timer: Timer
}

#[derive(Component, Debug)]
pub struct PlayerDirection {
    pub direction: Direction,
    pub walking: bool,
}

#[derive(Component, Default)]
pub struct KeyboardMovement(Vec<Direction>);

impl PlayerDirection {
    pub fn animation(&self) -> String {
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
    mut query: Query<(&mut PlayerDirection, &mut KeyboardMovement, &Position)>,
    mut place_bomb_events: EventWriter<PlaceBombEvent>,
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
                    place_bomb_events.send(PlaceBombEvent(*pos));
                }
            }
        }
    }
}

fn player_movement(
    time: Res<Time>,
    mut queries: ParamSet<(
        Query<(&mut PlayerSpeed, &mut Transform, &mut Position, &PlayerDirection)>,
        Query<(&Cell, &Position)>,
    )>,
) {
    let occupied: HashSet<Position> = queries
        .p1()
        .iter()
        .filter(|(cell, _pos)| cell != &&Cell::Blank)
        .map(|(_cell, pos)| *pos)
        .collect();

    for (mut timer, mut transform, mut pos, player_direction) in queries.p0().iter_mut() {
        timer.timer.tick(time.delta());
        if timer.timer.finished() && player_direction.walking {
            let mut new_translation = transform.translation;
            match player_direction.direction {
                Direction::West => new_translation.x -= 2.,
                Direction::East => new_translation.x += 2.,
                Direction::South => new_translation.y -= 2.,
                Direction::North => new_translation.y += 2.,
            };

            let new_pos = Position::from(new_translation.truncate());

            let collision = occupied.contains(&new_pos);

            if !collision && new_pos.valid() {
                transform.translation.x = new_translation.x;
                transform.translation.y = new_translation.y;
                pos.x = new_pos.x;
                pos.y = new_pos.y;
            }
        }
    }
}

// NOTE: systems with Changed<> must be added AFTER the system that triggers the change.
fn change_sprite(
    mut commands: Commands,
    named_assets: Res<NamedAssets>,
    animation_assets: Res<Assets<Animation>>,
    mut query: Query<(Entity, &mut Children, &PlayerDirection), Changed<PlayerDirection>>,
) {
    for (entity, children, player_direction) in query.iter_mut() {
        if let Some(child) = children.last().copied() {
            commands.entity(child).despawn_recursive();
        }

        let animation = named_assets
            .animations
            .get(&player_direction.animation())
            .unwrap();

        let child = commands
            .spawn_bundle(AnimatedSpriteBundle::new(
                animation.clone(),
                &animation_assets,
                // 55 = 110 (height) / 2, 18 = 36 (tile height) / 2
                // when position is translated into vec2, it points to the center, hence the correction for -18.
                Transform::from_translation(Vec3::new(0.0, 55.0 - 18.0, 0.0)),
            ))
            .id();

        commands.entity(entity).push_children(&[child]);
    }
}
