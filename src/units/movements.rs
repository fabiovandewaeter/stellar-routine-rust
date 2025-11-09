use crate::{
    UPS_TARGET,
    map::{MapManager, Position, Structure, pos_to_absolute_pos},
    units::{Unit, UnitUnitCollisions},
};
use bevy::prelude::*;

pub const UNIT_DEFAULT_MOVEMENT_SPEED: u32 = UPS_TARGET as u32; // ticks per tile ; smaller is faster (here its 1 tile per second at normal tickrate by default)

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct DesiredMovement {
    pub x: f32,
    pub y: f32,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Speed(pub u32);

impl Default for Speed {
    fn default() -> Self {
        Self(UNIT_DEFAULT_MOVEMENT_SPEED)
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    NorthWest,
    North,
    NorthEast,
    East,
    SouthEast,
    South,
    SouthWest,
    West,
}

impl Default for Direction {
    fn default() -> Self {
        Self::East
    }
}

pub fn move_and_collide_units_system(
    map_manager: Res<MapManager>,
    mut unit_query: Query<(Entity, &mut Position, Has<UnitUnitCollisions>), With<Unit>>,
) {
}

pub fn sync_transform_to_gridpos_system(
    mut query: Query<(&Position, &mut Transform), Without<Structure>>,
    time: Res<Time>,
) {
    for (pos, mut transform) in query.iter_mut() {
        // let target_pos = rounded_tile_pos_to_world(*grid_pos);
        let target_pos = pos_to_absolute_pos(*pos);
        let current_pos = transform.translation.xy();

        // Interpolation linéaire simple
        let new_pos = current_pos.lerp(target_pos.into(), time.delta_secs() * 10.0);
        transform.translation.x = new_pos.x;
        transform.translation.y = new_pos.y;
    }
}

pub fn update_sprite_facing_system(mut query: Query<(&Direction, &mut Transform)>) {
    for (facing_direction, mut transform) in query.iter_mut() {
        let is_moving_left = matches!(
            facing_direction,
            Direction::West | Direction::NorthWest | Direction::SouthWest
        );

        let is_moving_right = matches!(
            facing_direction,
            Direction::East | Direction::NorthEast | Direction::SouthEast
        );

        if is_moving_left {
            transform.scale.x = -transform.scale.x.abs();
        } else if is_moving_right {
            transform.scale.x = transform.scale.x.abs();
        }
    }
}
