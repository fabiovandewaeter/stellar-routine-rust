use crate::{
    map::Position,
    units::movements::{
        DesiredMovement, Direction, Speed, move_and_collide_units_system,
        sync_transform_to_gridpos_system, update_sprite_facing_system,
    },
};
use bevy::prelude::*;

pub const UNIT_REACH: f32 = 1.0;
pub const UNIT_DEFAULT_SIZE: f32 = 1.0;

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
    DesiredMovement,
    Speed,
    Size
)]
pub struct Unit {
    pub name: String,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Size(pub f32);

impl Default for Size {
    fn default() -> Self {
        Self(UNIT_DEFAULT_SIZE)
    }
}

#[derive(Component)]
pub struct Player;

/// add if the unit should checks its collisions with other units (collisions with walls are not affected by this component)
#[derive(Component)]
pub struct UnitUnitCollisions;

pub fn player_control_system(
    mut unit_query: Query<(&mut DesiredMovement, &Speed), (With<Unit>, With<Player>)>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    if let Ok((mut desired_movement, speed)) = unit_query.single_mut() {
        let mut delta = IVec2::new(0, 0);
        if input.pressed(KeyCode::KeyW) {
            delta.y -= 1;
        }
        if input.pressed(KeyCode::KeyA) {
            delta.x -= 1;
        }
        if input.pressed(KeyCode::KeyD) {
            delta.x += 1;
        }
        if input.pressed(KeyCode::KeyS) {
            delta.y += 1;
        }

        desired_movement.0.x = (delta.x as f32) * speed.0 * time.delta_secs();
        desired_movement.0.y = (delta.y as f32) * speed.0 * time.delta_secs();
    }
}
