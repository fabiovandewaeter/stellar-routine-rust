use std::f32::consts::{FRAC_2_PI, FRAC_PI_2, PI};

use bevy::{prelude::*, sprite_render::TilemapChunk};

use crate::{
    UPS_TARGET,
    items::inventory::Inventory,
    map::{MapManager, Structure, StructureManager, TileCoordinates, absolute_coord_to_tile_coord},
    units::Direction,
};

const DEFAULT_CRAFT_TIME_TICKS: u64 = UPS_TARGET as u64 * 1; // 1 second

pub struct MachinePlugin;

impl Plugin for MachinePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, orient_machines_system)
            .add_systems(
                FixedUpdate,
                (
                    process_production_in_machines_system,
                    transfert_items_to_next_machine_system,
                    print_machine_inventories_system,
                ),
            );
    }
}

#[derive(Component)]
#[require(Name, Structure, Direction)]
pub struct ProductionMachine {
    pub craft_time_ticks: u64,
    pub progress_ticks: u64,
    pub input_inventory: Inventory,
    pub output_inventory: Inventory,
}

impl Default for ProductionMachine {
    fn default() -> Self {
        Self {
            craft_time_ticks: DEFAULT_CRAFT_TIME_TICKS,
            progress_ticks: 0,
            input_inventory: Inventory::default(),
            output_inventory: Inventory::default(),
        }
    }
}

pub fn process_production_in_machines_system(mut machine_query: Query<&mut ProductionMachine>) {
    for mut production_machine in machine_query.iter_mut() {
        if production_machine.progress_ticks >= production_machine.craft_time_ticks {
            let item_stacks = production_machine.input_inventory.remove_all_item_stack();
            for item_stack in item_stacks {
                production_machine
                    .output_inventory
                    .add_item_stack(item_stack).expect("process_production_in_machines_system(): transfer to output_inventory didn't work");
            }
            production_machine.progress_ticks = 0;
        }
        production_machine.progress_ticks += 1;
    }
}

pub fn transfert_items_to_next_machine_system(
    mut machine_query: Query<(Entity, &Transform, &mut ProductionMachine, &Direction)>,
    chunk_query: Query<&StructureManager, With<TilemapChunk>>,
    map_manager: Res<MapManager>,
) {
    // we find all transfer pairs
    let mut transfer_pairs = Vec::new();
    for (source_machine_entity, transform, _, direction) in machine_query.iter() {
        let source_tile = absolute_coord_to_tile_coord((*transform).into());
        let delta = direction.direction_to_vec2();
        let target_tile = TileCoordinates {
            x: source_tile.x + delta.x,
            y: source_tile.y + delta.y,
        };

        if let Some(structure_entity) = map_manager.get_tile(target_tile, &chunk_query) {
            if let Ok((target_machine_entity, _, _, _)) = machine_query.get(structure_entity) {
                transfer_pairs.push((source_machine_entity, target_machine_entity))
            }
        }
    }

    // we do the transfers based on the transfer pairs
    for (source_entity, target_entity) in transfer_pairs {
        let Ok([(_, _, mut source_machine, _), (_, _, mut target_machine, _)]) =
            machine_query.get_many_mut([source_entity, target_entity])
        else {
            continue;
        };

        let item_stacks = source_machine.output_inventory.remove_all_item_stack();
        for item_stack in item_stacks {
            if !target_machine
                .input_inventory
                .add_item_stack(item_stack)
                .is_ok()
            {
                source_machine
                    .output_inventory
                    .add_item_stack(item_stack)
                    .expect("transfer didn't work and couldn't add items back in source_machine");
            }
        }
    }
}

pub fn print_machine_inventories_system(query: Query<(&Name, &ProductionMachine)>) {
    for (name, production_machine) in query.iter() {
        println!(
            "{:?}: {:?} | {:?}",
            name,
            production_machine.input_inventory.slots,
            production_machine.output_inventory.slots
        )
    }
}

pub fn orient_machines_system(
    mut query: Query<(&Direction, &mut Transform), With<ProductionMachine>>,
) {
    for (direction, mut transform) in query.iter_mut() {
        let angle = match direction {
            Direction::North => 0.0,       // up = sprite par défaut
            Direction::East => -FRAC_PI_2, // right = -90°
            Direction::South => PI,        // down = 180°
            Direction::West => FRAC_PI_2,  // left = +90°
        };

        transform.rotation = Quat::from_rotation_z(angle);
    }
}
