use crate::{
    map::{
        AbsoluteCoordinates, Coordinates, TILE_SIZE, absolute_coord_to_coord, coord_to_tile_coord,
    },
    units::pathfinding::{FlowField, RecalculateFlowField},
};
use avian2d::prelude::{
    CoefficientCombine, Collider, Friction, LinearVelocity, LockedAxes, RigidBody,
};
use bevy::prelude::*;

pub const UNIT_REACH: f32 = 1.0;
pub const UNIT_DEFAULT_SIZE: f32 = 1.0;

pub struct UnitsPlugin;

pub const UNIT_DEFAULT_MOVEMENT_SPEED: f32 = 1000.0;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(
            FixedUpdate,
            (
                (
                    player_control_system,
                    units_follow_field_system,
                    update_sprite_facing_system,
                    apply_floor_friction_system,
                ),
                sync_coords_system,
            )
                .chain(),
        );
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

#[derive(Component, Debug, Default)]
#[require(
    Sprite,
    Transform,
    Coordinates,
    Direction,
    Speed,
    RigidBody::Dynamic,
    Collider::circle(TILE_SIZE.x / 2.0),
    LinearVelocity::ZERO,
    LockedAxes::ROTATION_LOCKED,
    Friction {
        dynamic_coefficient: 0.0,
        static_coefficient: 0.0,
        combine_rule: CoefficientCombine::Multiply,
    },
)]
pub struct Unit {
    pub name: String,
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Speed(pub f32);

impl Default for Speed {
    fn default() -> Self {
        Self(UNIT_DEFAULT_MOVEMENT_SPEED)
    }
}

#[derive(Component)]
pub struct Player;

pub fn player_control_system(
    mut unit_query: Query<(&mut LinearVelocity, &mut Direction, &Speed), With<Player>>,
    input: Res<ButtonInput<KeyCode>>,
    mut message_recalculate: MessageWriter<RecalculateFlowField>,
    time: Res<Time>,
) {
    let Ok((mut velocity, mut direction, speed)) = unit_query.single_mut() else {
        return;
    };

    let mut delta = Vec2::ZERO;
    let mut has_moved = false;

    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        delta.y += 1.0;
        *direction = Direction::North;
    }
    if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
        delta.y -= 1.0;
        *direction = Direction::South;
    }
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        delta.x -= 1.0;
        *direction = Direction::West;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        delta.x += 1.0;
        *direction = Direction::East;
    }

    // Normaliser le vecteur pour éviter que le mouvement diagonal
    // soit plus rapide (racine(1²+1²) = 1.414)
    if delta.length_squared() > 0.0 {
        has_moved = true;
        delta = delta.normalize();
    }

    // Appliquer la vitesse
    let delta_time = time.delta_secs();
    velocity.x = delta.x * speed.0 * delta_time;
    velocity.y = delta.y * speed.0 * delta_time;

    // TODO: change to put that after the collisions check
    if has_moved {
        message_recalculate.write_default();
    }
}

fn sync_coords_system(mut query: Query<(&mut Coordinates, &Transform), With<Unit>>) {
    for (mut coords, transform) in query.iter_mut() {
        let abs = AbsoluteCoordinates {
            x: transform.translation.x,
            y: transform.translation.y,
        };

        let new_coords = absolute_coord_to_coord(abs);

        // Si les coords ont effectivement changé, updater le composant
        if (new_coords.x - coords.x).abs() > f32::EPSILON
            || (new_coords.y - coords.y).abs() > f32::EPSILON
        {
            *coords = new_coords;
        }
    }
}

pub fn units_follow_field_system(
    mut unit_query: Query<
        (&mut LinearVelocity, &mut Direction, &Coordinates, &Speed),
        (With<Unit>, Without<Player>),
    >,
    flow_field: Res<FlowField>,
    time: Res<Time>,
) {
    for (mut velocity, mut direction, coord, speed) in unit_query.iter_mut() {
        let tile = coord_to_tile_coord(*coord);

        if let Some(&delta) = flow_field.0.get(&tile) {
            let delta_time = time.delta_secs();
            velocity.x = delta.x * speed.0 * delta_time;
            velocity.y = -delta.y * speed.0 * delta_time;

            if delta.y < 0.0 {
                *direction = Direction::North;
            }
            if delta.y > 0.0 {
                *direction = Direction::South;
            }
            if delta.x < 0.0 {
                *direction = Direction::West;
            }
            if delta.x > 0.0 {
                *direction = Direction::East;
            }
        } else {
            velocity.x = 0.0;
            velocity.y = 0.0;
        }
    }
}

pub fn apply_floor_friction_system(
    mut unit_query: Query<&mut LinearVelocity, (With<Unit>, Without<Player>)>,
    time: Res<Time>,
) {
    const FRICTION_COEFF: f32 = 2.0;
    const CLAMP_LIMIT: f32 = 1e-4;
    for mut velocity in unit_query.iter_mut() {
        let factor = (1.0 - FRICTION_COEFF * time.delta_secs()).max(0.0);
        velocity.x *= factor;
        velocity.y *= factor;

        // tiny clamp pour éviter valeurs très petites qui trainent
        if velocity.length_squared() < CLAMP_LIMIT {
            velocity.x = 0.0;
            velocity.y = 0.0;
        }
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
