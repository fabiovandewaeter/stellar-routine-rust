use avian2d::prelude::LinearVelocity;
use bevy::prelude::*;

use crate::{
    items::inventory::Inventory,
    map::DEFAULT_CURRENT_MAP,
    units::{
        Direction, Player, Speed, Unit, load_units_from_file_system, save_units_to_file_system,
    },
};

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (save_on_key_system, load_on_key_system));
    }
}

pub fn save_on_key_system(
    input: Res<ButtonInput<KeyCode>>,
    units_query: Query<
        (
            &Name,
            &Transform,
            &Direction,
            &Speed,
            &LinearVelocity,
            Has<Player>,
            Option<&Inventory>,
        ),
        With<Unit>,
    >,
) {
    if input.just_pressed(KeyCode::F5) {
        save_units_to_file_system(units_query, DEFAULT_CURRENT_MAP.to_owned());
    }
}

pub fn load_on_key_system(
    input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    if input.just_pressed(KeyCode::F9) {
        load_units_from_file_system(&mut commands, &asset_server, DEFAULT_CURRENT_MAP.to_owned());
    }
}
