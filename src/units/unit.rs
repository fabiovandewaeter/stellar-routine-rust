use crate::{
    map::{
        AbsoluteCoordinates, TILE_SIZE, absolute_coord_to_tile_coord, tile_coord_to_absolute_coord,
    },
    units::pathfinding::{FlowField, RecalculateFlowField},
};
use avian2d::prelude::{
    CoefficientCombine, Collider, Forces, Friction, LinearDamping, LinearVelocity, LockedAxes,
    RigidBody, RigidBodyForces, TranslationInterpolation,
};
use bevy::prelude::*;

pub const UNIT_REACH: f32 = 1.0;
pub const UNIT_DEFAULT_SIZE: f32 = TILE_SIZE.x * 0.8;
// pub const UNIT_DEFAULT_MOVEMENT_SPEED: f32 = 2000.0;
pub const UNIT_DEFAULT_MOVEMENT_SPEED: f32 = 5000.0;
pub const UNIT_LAYER: f32 = 1.0;

pub struct UnitsPlugin;

impl Plugin for UnitsPlugin {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(
            FixedUpdate,
            (
                player_control_system,
                units_follow_field_system,
                update_sprite_facing_system,
                // apply_floor_friction_system,
            ),
        );
    }
}

#[derive(Component, Debug, Default)]
// #[require(
//     Name,
//     Sprite,
//     Transform,
//     Direction,
//     Speed,
//     RigidBody::Dynamic,
//     Collider::circle(UNIT_DEFAULT_SIZE / 2.0),
//     LinearVelocity::ZERO,
//     LockedAxes::ROTATION_LOCKED,
//     Friction {
//         dynamic_coefficient: 0.0,
//         static_coefficient: 0.0,
//         combine_rule: CoefficientCombine::Multiply,
//     },
//     TranslationInterpolation,
//     LinearDamping(20.0)
// )]
pub struct Unit;
#[derive(Bundle)]
pub struct UnitBundle {
    pub name: Name,
    pub transform: Transform,
    pub direction: Direction,
    pub speed: Speed,
    pub rigid_body: RigidBody,
    pub collider: Collider,
    pub linear_velocity: LinearVelocity,
    pub locked_axes: LockedAxes,
    pub friction: Friction,
    pub translation_interpolation: TranslationInterpolation,
    pub linear_damping: LinearDamping,
    pub unit: Unit,
}
impl UnitBundle {
    pub fn new(name: Name, transform: Transform, speed: Speed) -> Self {
        Self {
            name,
            transform,
            direction: Direction::East,
            speed,
            rigid_body: RigidBody::Dynamic,
            collider: Collider::circle(UNIT_DEFAULT_SIZE / 2.0),
            linear_velocity: LinearVelocity::ZERO,
            locked_axes: LockedAxes::ROTATION_LOCKED,
            friction: Friction {
                dynamic_coefficient: 0.0,
                static_coefficient: 0.0,
                combine_rule: CoefficientCombine::Multiply,
            },
            translation_interpolation: TranslationInterpolation,
            linear_damping: LinearDamping(20.0),
            unit: Unit,
        }
    }
}
#[derive(Component)]
pub struct Player;

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    East,
    South,
    West,
}
impl Direction {
    pub fn direction_to_vec2(&self) -> IVec2 {
        match self {
            Direction::North => IVec2 { x: 0, y: -1 },
            Direction::East => IVec2 { x: 1, y: 0 },
            Direction::South => IVec2 { x: 0, y: 1 },
            Direction::West => IVec2 { x: -1, y: 0 },
        }
    }
}

impl Default for Direction {
    fn default() -> Self {
        Self::East
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Speed(pub f32);
impl Default for Speed {
    fn default() -> Self {
        Self(UNIT_DEFAULT_MOVEMENT_SPEED)
    }
}

pub fn player_control_system(
    mut unit_query: Query<(&mut LinearVelocity, &mut Direction, &Speed), With<Player>>,
    input: Res<ButtonInput<KeyCode>>,
    mut message_recalculate: MessageWriter<RecalculateFlowField>,
    time: Res<Time<Fixed>>,
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

// fn sync_coords_system(mut query: Query<(&mut Transform), With<Unit>>) {
//     for (mut transform) in query.iter_mut() {
//         let abs = AbsoluteCoordinates {
//             x: transform.translation.x,
//             y: transform.translation.y,
//         };

//         let new_coords = absolute_coord_to_coord(abs);

//         let coords = absolute_coord_to_coord(abs);
//         if (new_coords.x - coords.x).abs() > f32::EPSILON
//             || (new_coords.y - coords.y).abs() > f32::EPSILON
//         {
//             let new_absolute = coord_to_absolute_coord(new_coords);
//             transform.translation.x = new_absolute.x;
//             transform.translation.y = new_absolute.y;
//         }
//     }
// }

pub fn units_follow_field_system(
    // mut unit_query: Query<
    //     (&mut LinearVelocity, &mut Direction, &Transform, &Speed),
    //     (With<Unit>, Without<Player>),
    // >,
    mut unit_query: Query<
        (
            // &mut LinearVelocity,
            &mut Direction,
            Forces,
            &Transform,
            &Speed,
        ),
        (With<Unit>, Without<Player>),
    >,
    flow_field: Res<FlowField>,
    time: Res<Time<Fixed>>,
) {
    // const MAX_SPEED: f32 = 30.0;
    const MAX_SPEED: f32 = 35.0;
    const ARRIVAL_DISTANCE: f32 = TILE_SIZE.x * 0.25;

    // for (mut velocity, mut direction, mut forces, transform, speed) in unit_query.iter_mut() {
    for (mut direction, mut forces, transform, speed) in unit_query.iter_mut() {
        // 1. Position actuelle de l'unité
        let current_pos_world = transform.translation.xy();
        let current_pos_abs = AbsoluteCoordinates {
            x: current_pos_world.x,
            y: current_pos_world.y,
        };
        let current_tile = absolute_coord_to_tile_coord(current_pos_abs);

        // 2. Trouver la prochaine tuile cible depuis le flow field
        // ASSUREZ-VOUS d'avoir aussi fait la modification du FlowField
        // pour qu'il contienne des TileCoordinates (waypoints)
        if let Some(&next_tile) = flow_field.0.get(&current_tile) {
            // 3. Calculer la position CIBLE (le centre de la prochaine tuile)
            let target_pos_abs = tile_coord_to_absolute_coord(next_tile);
            let target_pos_world: Vec2 = target_pos_abs.into();

            // 4. Calculer la direction et la force vers la cible
            let to_target_vec = target_pos_world - current_pos_world;
            let distance = to_target_vec.length();

            // if distance < ARRIVAL_DISTANCE {
            //     // On freine activement l'unité pour la stabiliser
            //     // MODIFIÉ : L'appel de fonction est identique, mais
            //     // c'est sur l'objet `Forces`
            //     forces.apply_force(-forces.linear_velocity() * 5.0); // Simple "damping"
            //     continue;
            // }

            let direction_to_target = to_target_vec.normalize_or_zero();

            // 5. Appliquer la FORCE
            // MODIFIÉ : L'appel de fonction est identique
            forces.apply_force(direction_to_target * speed.0);

            // 6. Mettre à jour la direction du sprite (logique inchangée)
            let abs_x = direction_to_target.x.abs();
            let abs_y = direction_to_target.y.abs();

            if abs_x > abs_y {
                *direction = if direction_to_target.x > 0.0 {
                    Direction::East
                } else {
                    Direction::West
                };
            } else {
                // Note : J'ai inversé North/South car votre système de
                // coordonnées Y semble inversé (le joueur va "vers le haut"
                // avec `delta.y += 1.0` mais `absolute_coord_to_coord`
                // utilise `-absolute_coord.y`). À VÉRIFIER.
                // Si la direction du sprite est fausse, inversez North/South ici.
                *direction = if direction_to_target.y > 0.0 {
                    Direction::North
                } else {
                    Direction::South
                };
            }

            // 7. BRIDER la vitesse (logique inchangée)
            // if forces.linear_velocity().length_squared() > MAX_SPEED * MAX_SPEED {
            //     *forces.linear_velocity_mut() =
            //         forces.linear_velocity().normalize_or_zero() * MAX_SPEED;
            // }

            // } else {
            //     // Pas de chemin dans le flow field (on est sur la cible ou bloqué)
            //     // On freine activement
            //     // MODIFIÉ : L'appel de fonction est identique
            //     forces.apply_force(-forces.linear_velocity() * 10.0);

            //     if forces.linear_velocity().length_squared() < 0.1 {
            //         *forces.linear_velocity_mut() = Vec2::ZERO;
            //     }
        }
    }
}

pub fn apply_floor_friction_system(
    mut unit_query: Query<&mut LinearVelocity, (With<Unit>, Without<Player>)>,
    time: Res<Time<Fixed>>,
) {
    const FRICTION_COEFF: f32 = 2.0;
    const CLAMP_LIMIT: f32 = 1e-4;
    let delta_time = time.delta_secs();
    for mut velocity in unit_query.iter_mut() {
        velocity.y *= 1.0 / (1.0 + FRICTION_COEFF * delta_time);
        velocity.x *= 1.0 / (1.0 + FRICTION_COEFF * delta_time);

        // tiny clamp pour éviter valeurs très petites qui trainent
        if velocity.length_squared() < CLAMP_LIMIT {
            velocity.x = 0.0;
            velocity.y = 0.0;
        }
    }
}

pub fn update_sprite_facing_system(mut query: Query<(&Direction, &mut Transform)>) {
    for (facing_direction, mut transform) in query.iter_mut() {
        let is_moving_left = matches!(facing_direction, Direction::West);

        let is_moving_right = matches!(facing_direction, Direction::East);

        if is_moving_left {
            transform.scale.x = -transform.scale.x.abs();
        } else if is_moving_right {
            transform.scale.x = transform.scale.x.abs();
        }
    }
}
