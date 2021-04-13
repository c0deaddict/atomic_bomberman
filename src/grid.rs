use crate::state::*;
use bevy::{math::const_vec2, prelude::*};
use std::cmp::{Eq, PartialEq};
use std::fmt::Debug;

#[derive(Default)]
pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_system_set(
            SystemSet::on_update(AppState::Game).with_system(position_translation.system()),
        );
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

pub struct Offset(pub Vec3);

pub struct SnapToGrid;

const GRID_OFFSET: Vec2 = const_vec2!([20.0, -64.0]);
const CELL_DIMENSION: Vec2 = const_vec2!([40.0, -36.0]);

impl From<Position> for Vec2 {
    fn from(pos: Position) -> Self {
        let v = Vec2::new(pos.x as f32, pos.y as f32);
        GRID_OFFSET + (v * CELL_DIMENSION)
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
