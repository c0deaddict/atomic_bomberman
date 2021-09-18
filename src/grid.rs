use crate::state::*;
use bevy::input::keyboard::KeyboardInput;
use bevy::{math::const_vec2, prelude::*};
use std::cmp::{Eq, PartialEq};
use std::fmt::Debug;

use self::Direction::*;

#[derive(Default)]
pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new().with_system(position_translation.system()),
        )
        .add_startup_system(setup.system())
        .add_system(keyboard_handling.system());
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Direction {
    North,
    South,
    East,
    West,
}

pub struct Offset(pub Vec3);

pub struct SnapToGrid;

pub struct DebugGrid;

pub const WIDTH: i32 = 15;
pub const HEIGHT: i32 = 11;

const GRID_OFFSET: Vec2 = const_vec2!([20.0, -64.0]);
const CELL_DIMENSION: Vec2 = const_vec2!([40.0, -36.0]);

impl From<Position> for Vec2 {
    // Convert position to center of cell.
    fn from(pos: Position) -> Self {
        let v = Vec2::new(pos.x as f32, pos.y as f32);
        GRID_OFFSET + (v * CELL_DIMENSION) + (0.5 * CELL_DIMENSION)
    }
}

impl From<Vec2> for Position {
    fn from(v: Vec2) -> Self {
        let v = (v - GRID_OFFSET) / CELL_DIMENSION;
        Position {
            x: if v.x < 0.0 { -1 } else { v.x as i32 },
            y: if v.y < 0.0 { -1 } else { v.y as i32},
        }
    }
}

fn position_translation(mut query: Query<(&Position, &Offset, &mut Transform), With<SnapToGrid>>) {
    for (pos, offset, mut transform) in query.iter_mut() {
        transform.translation = offset.0 + Vec3::from((Vec2::from(*pos), 0.));
    }
}

pub struct PositionIterator {
    pos: Option<Position>,
    dir: Direction,
}

impl Position {
    pub fn valid(&self) -> bool {
        self.x >= 0 && self.y >= 0 && self.x < WIDTH && self.y < HEIGHT
    }

    pub fn move_to(&mut self, dir: Direction) {
        match dir {
            Direction::West => self.x -= 1,
            Direction::East => self.x += 1,
            Direction::South => self.y -= 1,
            Direction::North => self.y += 1,
        };
    }

    fn neighbour(&self, dir: Direction) -> Option<Position> {
        let mut res = *self;
        res.move_to(dir);
        if res.valid() {
            Some(res)
        } else {
            None
        }
    }

    pub fn iter(&self, dir: Direction) -> PositionIterator {
        PositionIterator {
            pos: Some(*self),
            dir,
        }
    }
}

impl Iterator for PositionIterator {
    type Item = Position;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(pos) = self.pos {
            self.pos = pos.neighbour(self.dir);
            self.pos
        } else {
            None
        }
    }
}

impl Direction {
    pub fn iter() -> impl Iterator<Item = Direction> {
        [North, South, East, West].iter().copied()
    }

    pub fn axis(&self) -> Axis {
        match self {
            North => Axis::Vertical,
            South => Axis::Vertical,
            East => Axis::Horizontal,
            West => Axis::Horizontal,
        }
    }
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    // Draw a debug grid over the screen.
    let blue = materials.add(Color::rgb(0.0, 0.0, 1.0).into());

    for x in 0..16 {
        commands
            .spawn_bundle(SpriteBundle {
                material: blue.clone(),
                sprite: Sprite::new(Vec2::new(1.0, 480.0)),
                transform: Transform::from_translation(Vec3::new(
                    20.0 + (x as f32) * 40.0,
                    -240.0,
                    50.0,
                )),
                visible: Visible {
                    is_transparent: true,
                    is_visible: false,
                },
                ..Default::default()
            })
            .insert(DebugGrid);
    }

    for y in 0..12 {
        commands
            .spawn_bundle(SpriteBundle {
                material: blue.clone(),
                sprite: Sprite::new(Vec2::new(640.0, 1.0)),
                transform: Transform::from_translation(Vec3::new(
                    320.0,
                    -64.0 + (y as f32) * -36.0,
                    50.0,
                )),
                visible: Visible {
                    is_transparent: true,
                    is_visible: false,
                },
                ..Default::default()
            })
            .insert(DebugGrid);
    }
}

fn keyboard_handling(
    mut keyboard_events: EventReader<KeyboardInput>,
    mut query: Query<&mut Visible, With<DebugGrid>>,
) {
    let mut visible: Option<bool> = None;

    for event in keyboard_events.iter() {
        if event.state.is_pressed() {
            if let Some(key_code) = event.key_code {
                if key_code == KeyCode::G {
                    println!("showing debug grid");
                    visible = Some(true);
                } else if key_code == KeyCode::H {
                    visible = Some(false);
                    println!("hiding debug grid");
                }
            }
        }
    }

    if let Some(state) = visible {
        for mut sprite in query.iter_mut() {
            sprite.is_visible = state;
        }
    }
}
