use avian2d::prelude::*;
use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*};
use stellar_routine_rust::{
    UPS_TARGET,
    camera::{
        CameraMovement, CameraMovementKind, UpsCounter, display_fps_ups_system,
        handle_camera_inputs_system,
    },
    map::{Coordinates, MapPlugin, coord_to_absolute_coord},
    units::{
        Player, Speed, UNIT_DEFAULT_MOVEMENT_SPEED, UNIT_LAYER, Unit, UnitsPlugin,
        pathfinding::PathfindingPlugin,
    },
};

const LENGTH_UNIT: f32 = 16.0;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Stellar Routine".to_string(),
                        present_mode: bevy::window::PresentMode::AutoVsync,
                        ..default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(PhysicsPlugins::default().with_length_unit(LENGTH_UNIT))
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(UnitsPlugin)
        .add_plugins(MapPlugin)
        .add_plugins(PathfindingPlugin)
        .insert_resource(Gravity(Vec2::ZERO))
        // .insert_resource(TimeState::default())
        .insert_resource(UpsCounter {
            ticks: 0,
            last_second: 0.0,
            ups: 0,
        })
        .insert_resource(Time::<Fixed>::from_hz(UPS_TARGET as f64))
        .add_systems(Startup, setup_system)
        .add_systems(
            Update,
            (
                handle_camera_inputs_system,
                display_fps_ups_system,
                // control_time_system,
            ),
        )
        .add_systems(FixedUpdate, update_logic_system)
        .run();
}

fn setup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    let mut orthographic_projection = OrthographicProjection::default_2d();
    orthographic_projection.scale *= 0.8;
    let projection = Projection::Orthographic(orthographic_projection);
    commands.spawn((
        Camera2d,
        Camera { ..default() },
        projection,
        CameraMovement(CameraMovementKind::SmoothFollowPlayer),
    ));

    let player_texture_handle = asset_server.load("default.png");
    let speed = Speed(UNIT_DEFAULT_MOVEMENT_SPEED * 2.0);
    let coordinates = Coordinates { x: 0.0, y: 0.0 };
    let absolute_coordinates = coord_to_absolute_coord(coordinates);
    let mut transform =
        Transform::from_xyz(absolute_coordinates.x, absolute_coordinates.y, UNIT_LAYER);
    transform.scale *= 0.8;
    commands.spawn((
        Unit {
            name: "Player".into(),
        },
        Sprite::from_image(player_texture_handle.clone()),
        transform,
        Player,
        speed,
    ));

    let coordinates = Coordinates { x: 5.0, y: 5.0 };
    let absolute_coordinates = coord_to_absolute_coord(coordinates);
    let mut transform =
        Transform::from_xyz(absolute_coordinates.x, absolute_coordinates.y, UNIT_LAYER);
    transform.scale *= 0.8;
    commands.spawn((
        Unit {
            name: "Monstre".into(),
        },
        Sprite::from_image(player_texture_handle.clone()),
        transform,
    ));
}

pub fn update_logic_system(mut counter: ResMut<UpsCounter>) {
    counter.ticks += 1;
}

// #[derive(Resource, Default)]
// struct TimeState {
//     is_paused: bool,
// }

// fn control_time_system(
//     mut fixed_time: ResMut<Time<Fixed>>,
//     input: Res<ButtonInput<KeyCode>>,
//     mut time_state: ResMut<TimeState>,
// ) {
//     // P pour Pause, pour alterner entre l'état de pause
//     if input.just_pressed(KeyCode::Space) {
//         if time_state.is_paused {
//             println!("Temps de la simulation repris.");
//             fixed_time.set_timestep_hz(UPS_TARGET as f64);
//             time_state.is_paused = false;
//         } else {
//             println!("Temps de la simulation mis en pause.");
//             fixed_time.set_timestep_hz(0.0);
//             time_state.is_paused = true;
//         }
//     }

//     // Si le jeu est en pause, on ne gère pas les autres commandes de vitesse
//     if time_state.is_paused {
//         return;
//     }

//     // Accélérer (x2)
//     if input.just_pressed(KeyCode::KeyY) {
//         let current_hz = fixed_time.timestep().as_secs_f64().recip();
//         let new_hz = current_hz * 2.0;
//         println!("Temps de la simulation accéléré à {} Hz.", new_hz);
//         fixed_time.set_timestep_hz(new_hz);
//     }

//     // Ralentir (/2)
//     if input.just_pressed(KeyCode::KeyU) {
//         let current_hz = fixed_time.timestep().as_secs_f64().recip();
//         let new_hz = current_hz / 2.0;
//         println!("Temps de la simulation ralenti à {} Hz.", new_hz);
//         fixed_time.set_timestep_hz(new_hz);
//     }

//     // Normal (retour à la vitesse initiale)
//     if input.just_pressed(KeyCode::KeyI) {
//         println!("Temps de la simulation réinitialisé à {} Hz.", UPS_TARGET);
//         fixed_time.set_timestep_hz(UPS_TARGET as f64);
//     }
// }
