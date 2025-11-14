use crate::map::TILE_SIZE;
use avian2d::prelude::{
    CoefficientCombine, Collider, Friction, LinearVelocity, LockedAxes, RigidBody,
};
use bevy::prelude::*;

pub const UNIT_REACH: f32 = 1.0;
pub const UNIT_DEFAULT_SIZE: f32 = 1.0;

pub struct UnitsPlugin;

pub const UNIT_DEFAULT_MOVEMENT_SPEED: f32 = 50.0;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(FixedUpdate, (player_control_system,));
    }
}

#[derive(Component, Debug, Default)]
#[require(
    Sprite,
    // Transform,
    // Coordinates,
    // Direction,
    // DesiredMovement,
    Speed,
    // Size
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
    mut unit_query: Query<(&mut LinearVelocity, &Speed), With<Player>>,
    input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
) {
    let Ok((mut velocity, speed)) = unit_query.single_mut() else {
        return;
    };

    let mut direction = Vec2::ZERO;

    if input.pressed(KeyCode::KeyW) || input.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if input.pressed(KeyCode::KeyS) || input.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if input.pressed(KeyCode::KeyA) || input.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if input.pressed(KeyCode::KeyD) || input.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    // Normaliser le vecteur pour éviter que le mouvement diagonal
    // soit plus rapide (racine(1²+1²) = 1.414)
    if direction.length_squared() > 0.0 {
        direction = direction.normalize();
    }

    // Appliquer la vitesse
    velocity.x = direction.x * speed.0;
    velocity.y = direction.y * speed.0;
}
