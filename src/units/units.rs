use crate::{
    map::Position,
    units::movements::{
        DesiredMovement, Direction, Speed, move_and_collide_units_system,
        sync_transform_to_gridpos_system, update_sprite_facing_system,
    },
};
use bevy::prelude::*;

pub const UNIT_REACH: u8 = 1;

pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(
            Update,
            sync_transform_to_gridpos_system.after(move_and_collide_units_system),
        )
        .add_systems(
            FixedUpdate,
            (
                move_and_collide_units_system,
                player_control_system,
                update_sprite_facing_system.after(move_and_collide_units_system),
            ),
        );
    }
}

#[derive(Component, Debug, Default)]
#[require(
    Sprite,
    // Transform,
    Position,
    Direction,
    Speed,
)]
pub struct Unit {
    pub name: String,
}

#[derive(Component)]
pub struct Player;

/// add if the unit should checks its collisions with other units (collisions with walls are not affected by this component)
#[derive(Component)]
pub struct UnitUnitCollisions;

pub fn player_control_system(
    mut unit_query: Query<(&Position, &mut DesiredMovement, &Speed), (With<Unit>, With<Player>)>,
    input: Res<ButtonInput<KeyCode>>,
) {
    if let Ok((position, mut desired_movement, speed)) = unit_query.single_mut() {
        let mut delta = IVec2::new(0, 0);
        if input.pressed(KeyCode::KeyW) {
            delta.y += 1;
        }
        if input.pressed(KeyCode::KeyA) {
            delta.x -= 1;
        }
        if input.pressed(KeyCode::KeyD) {
            delta.x += 1;
        }
        if input.pressed(KeyCode::KeyS) {
            delta.y -= 1;
        }

        // if tile_movement.direction != new_direction {
        //     tile_movement.direction = new_direction;
        // }
        desired_movement.x = position.x + (delta.x * speed.0 as i32) as f32;
        desired_movement.y = position.y + (delta.y * speed.0 as i32) as f32;
    }
}
