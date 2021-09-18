use crate::state::*;
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
            SystemSet::new()
                .with_system(position_translation.system())
        );
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
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
            x: v.x as i32,
            y: v.y as i32,
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
    fn valid(&self) -> bool {
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
        PositionIterator{pos: Some(*self), dir}
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
}
