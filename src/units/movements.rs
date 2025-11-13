use crate::{
    UPS_TARGET,
    map::{
        MapManager, Position, Structure, StructureManager, TILE_SIZE, grid_pos_to_chunk_pos,
        pos_to_absolute_pos, pos_to_chunk_pos, pos_to_grid_pos,
    },
    units::{Size, Unit, UnitUnitCollisions},
};
use bevy::{prelude::*, sprite_render::TilemapChunk};

pub const UNIT_DEFAULT_MOVEMENT_SPEED: f32 = 5.0;

#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct DesiredMovement(pub Position);

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Speed(pub f32);

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
    mut unit_query: Query<
        (
            &mut Position,
            &DesiredMovement,
            &Size,
            Has<UnitUnitCollisions>,
        ),
        With<Unit>,
    >,
    chunk_query: Query<&StructureManager, With<TilemapChunk>>,
) {
    for (mut position, desired_movement, unit_size, has_collision_with_units) in
        unit_query.iter_mut()
    {
        if desired_movement.0.x == 0.0 && desired_movement.0.y == 0.0 {
            continue;
        }

        let size = unit_size.0 - 0.01;
        let corners: Vec<Position> = vec![
            *position,
            Position {
                x: position.x + size,
                y: position.y,
            },
            Position {
                x: position.x,
                y: position.y + size,
            },
            Position {
                x: position.x + size,
                y: position.y + size,
            },
        ];

        // x axis
        if desired_movement.0.x != 0.0 {
            let mut can_move = true;
            for corner in &corners {
                let desired_pos = Position {
                    x: corner.x + desired_movement.0.x,
                    y: corner.y,
                };
                let desired_grid_pos = pos_to_grid_pos(desired_pos);
                let desired_chunk_pos = pos_to_chunk_pos(desired_pos);
                println!(
                    "{:?} {:?} {:?}",
                    desired_movement.0.x, desired_pos, desired_grid_pos,
                );
                if let Some(chunk_entity) = map_manager.chunks.get(&desired_chunk_pos) {
                    if let Ok(structure_manager) = chunk_query.get(*chunk_entity) {
                        if structure_manager
                            .structures
                            .get(&desired_grid_pos)
                            .is_some()
                        {
                            can_move = false;
                            let mut limited_movement = desired_grid_pos.x as f32 - 1.0 - position.x;
                            if desired_movement.0.x < 0.0 {
                                limited_movement = desired_grid_pos.x as f32 + 1.0 - position.x;
                            }
                            *position = Position {
                                x: position.x + limited_movement,
                                y: position.y,
                            };
                            break;
                        }
                    } else {
                        can_move = false;
                        break;
                    }
                } else {
                    can_move = false;
                    break;
                }
            }

            if can_move {
                *position = Position {
                    x: position.x + desired_movement.0.x,
                    y: position.y,
                };
            }
        }

        // y axis
        if desired_movement.0.y != 0.0 {
            let mut can_move = true;
            for corner in &corners {
                let desired_pos = Position {
                    x: corner.x,
                    y: corner.y + desired_movement.0.y,
                };
                let desired_grid_pos = pos_to_grid_pos(desired_pos);
                let desired_chunk_pos = pos_to_chunk_pos(desired_pos);
                if let Some(chunk_entity) = map_manager.chunks.get(&desired_chunk_pos) {
                    if let Ok(structure_manager) = chunk_query.get(*chunk_entity) {
                        if structure_manager
                            .structures
                            .get(&desired_grid_pos)
                            .is_some()
                        {
                            can_move = false;
                            let mut limited_movement = desired_grid_pos.y as f32 - 1.0 - position.y;
                            if desired_movement.0.y < 0.0 {
                                limited_movement = desired_grid_pos.y as f32 + 1.0 - position.y;
                            }
                            *position = Position {
                                x: position.x,
                                y: position.y + limited_movement,
                            };
                            break;
                        }
                    } else {
                        can_move = false;
                        break;
                    }
                } else {
                    can_move = false;
                    break;
                }
            }

            if can_move {
                *position = Position {
                    x: position.x,
                    y: position.y + desired_movement.0.y,
                };
            }
        }
    }
}

pub fn sync_transform_to_gridpos_system(
    mut query: Query<(&Position, &mut Transform), Without<Structure>>,
    // mut query: Query<(&Position, &mut Transform)>,
    time: Res<Time>,
) {
    for (pos, mut transform) in query.iter_mut() {
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
