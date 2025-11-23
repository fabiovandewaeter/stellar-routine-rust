use crate::{
    UPS_TARGET,
    items::{
        inventory::{InputInventory, ItemStack, OutputInventory},
        recipe::{RecipeBook, RecipeId},
    },
    map::{
        MapManager, Structure, StructureLayerManager, TileCoordinates, absolute_coord_to_tile_coord,
    },
    units::Direction,
};
use bevy::{prelude::*, sprite_render::TilemapChunk};
use std::f32::consts::{FRAC_PI_2, PI};

pub const DEFAULT_ACTION_TIME_TICKS: u64 = UPS_TARGET as u64 * 1; // 1 second

pub struct MachinePlugin;

impl Plugin for MachinePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, orient_machines_system)
            .add_systems(
                FixedUpdate,
                (
                    (
                        process_crafting_machines_system,
                        process_belt_machines_system,
                        process_mining_machines_system,
                    ),
                    transfert_items_to_next_machine_system,
                    print_machine_inventories_system,
                ),
            );
    }
}

#[derive(Component)]
pub struct Machine {
    pub action_time_ticks: u64,
    pub action_speed: f32,
    pub action_progress_ticks: u64,
}
impl Default for Machine {
    fn default() -> Self {
        Self {
            action_time_ticks: DEFAULT_ACTION_TIME_TICKS,
            action_speed: 1.0,
            action_progress_ticks: 0,
        }
    }
}

#[derive(Bundle)]
pub struct MachineBaseBundle {
    pub name: Name,
    pub structure: Structure,
    pub direction: Direction,
    pub transform: Transform,
    pub machine: Machine,
}

#[derive(Component)]
pub struct BeltMachine;
#[derive(Bundle)]
pub struct BeltMachineBundle {
    pub base: MachineBaseBundle,
    pub input_inventory: InputInventory,
    pub output_inventory: OutputInventory,
    pub belt_machine: BeltMachine,
}

#[derive(Component)]
pub struct CraftingMachine {
    pub recipe_id: Option<RecipeId>,
}
#[derive(Bundle)]
pub struct CraftingMachineBundle {
    pub base: MachineBaseBundle,
    pub input_inventory: InputInventory,
    pub output_inventory: OutputInventory,
    pub crafting_machine: CraftingMachine,
}
impl CraftingMachine {
    pub fn new(recipe_id: RecipeId) -> Self {
        Self {
            recipe_id: Some(recipe_id),
        }
    }
}
impl Default for CraftingMachine {
    fn default() -> Self {
        Self { recipe_id: None }
    }
}

#[derive(Component)]
pub struct MiningMachine {
    pub mined_item: Option<ItemStack>,
}
#[derive(Bundle)]
pub struct MiningMachineBundle {
    pub base: MachineBaseBundle,
    pub output_inventory: OutputInventory,
    pub mining_machine: MiningMachine,
}
impl MiningMachine {
    pub fn new(mined_item: ItemStack) -> Self {
        Self {
            mined_item: Some(mined_item),
        }
    }
}
impl Default for MiningMachine {
    fn default() -> Self {
        Self { mined_item: None }
    }
}

pub fn process_belt_machines_system(
    mut machine_query: Query<
        (&mut Machine, &mut InputInventory, &mut OutputInventory),
        With<BeltMachine>,
    >,
) {
    for (mut machine, mut input_inventory, mut output_inventory) in machine_query.iter_mut() {
        if machine.action_progress_ticks >= machine.action_time_ticks {
            let item_stacks = input_inventory.0.remove_all_item_stack();
            for item_stack in item_stacks {
                output_inventory.0.add(item_stack).expect(
                    "process_belt_machines_system(): transfer to output_inventory didn't work",
                );
            }
            machine.action_progress_ticks = 0;
        }

        // start if previous action finised and there is items in input_inventory
        if machine.action_progress_ticks == 0 {
            if !input_inventory.0.slots.is_empty() {
                machine.action_time_ticks =
                    (DEFAULT_ACTION_TIME_TICKS as f32 / machine.action_speed) as u64;
                // TODO: see if need to change to 0
                machine.action_progress_ticks = 1;
            }
        } else if machine.action_progress_ticks > 0 {
            machine.action_progress_ticks += 1;
        }
    }
}

pub fn process_crafting_machines_system(
    mut machine_query: Query<(
        &mut Machine,
        &CraftingMachine,
        &mut InputInventory,
        &mut OutputInventory,
    )>,
    recipe_book: Res<RecipeBook>,
) {
    for (mut machine, crafting_machine, mut input_inventory, mut output_inventory) in
        machine_query.iter_mut()
    {
        let Some(recipe_id) = crafting_machine.recipe_id else {
            continue;
        };
        let Some(recipe) = recipe_book.0.get(&recipe_id) else {
            continue;
        };

        // use machine.action_time_ticks instead of recipe.base_craft_time_ticks because machine.action_time_ticks change because of machine.action_speed
        if machine.action_progress_ticks >= machine.action_time_ticks {
            for item_stack in &recipe.outputs {
                output_inventory
                    .0
                    .add(*item_stack)
                    .expect("add_item_stack() didn't work");
            }
            machine.action_progress_ticks = 0;
        }

        // start a new craft if possible
        if machine.action_progress_ticks == 0 {
            let mut items_present = true;
            for item_stack in &recipe.inputs {
                if !input_inventory.0.enough_quantity(*item_stack) {
                    items_present = false;
                    break;
                }
            }
            if !items_present {
                continue;
            }
            // consumes the input items
            for item_stack in &recipe.inputs {
                input_inventory.0.remove_quantity(*item_stack);
            }

            // reset the crafting machine
            machine.action_time_ticks =
                (recipe.base_craft_time_ticks as f32 / machine.action_speed) as u64;
            // TODO: see if need to change to 0
            machine.action_progress_ticks = 1;
        } else if machine.action_progress_ticks > 0 {
            machine.action_progress_ticks += 1;
        }
    }
}

pub fn process_mining_machines_system(
    mut machine_query: Query<(&mut Machine, &MiningMachine, &mut OutputInventory)>,
) {
    for (mut machine, mining_machine, mut output_inventory) in machine_query.iter_mut() {
        let Some(mined_item) = mining_machine.mined_item else {
            continue;
        };

        if machine.action_progress_ticks >= machine.action_time_ticks {
            let new_item_stack = mined_item.clone();
            output_inventory.0.add(new_item_stack).expect(
                "process_mining_machines_system(): transfer to output_inventory didn't work",
            );
            machine.action_progress_ticks = 0;
        }

        // start if previous action finised and if there is still room for more items
        if let Some(mined_item) = mining_machine.mined_item {
            if machine.action_progress_ticks == 0 && output_inventory.0.enough_room(mined_item) {
                machine.action_time_ticks =
                    (DEFAULT_ACTION_TIME_TICKS as f32 / machine.action_speed) as u64;
                // TODO: see if need to change to 0
                machine.action_progress_ticks = 1;
            } else if machine.action_progress_ticks > 0 {
                machine.action_progress_ticks += 1;
            }
        }
    }
}

pub fn transfert_items_to_next_machine_system(
    mut machine_query: Query<(
        Entity,
        &Transform,
        &mut Machine,
        &Direction,
        Option<&mut InputInventory>,
        &mut OutputInventory,
    )>,
    chunk_query: Query<&StructureLayerManager, With<TilemapChunk>>,
    map_manager: Res<MapManager>,
) {
    // we find all transfer pairs
    let mut transfer_pairs = Vec::new();
    for (
        source_machine_entity,
        transform,
        _,
        direction,
        mut input_inventory,
        mut output_inventory,
    ) in machine_query.iter()
    {
        let source_tile = absolute_coord_to_tile_coord((*transform).into());
        let delta = direction.direction_to_vec2();
        let target_tile = TileCoordinates {
            x: source_tile.x + delta.x,
            y: source_tile.y + delta.y,
        };

        if let Some(structure_entity) = map_manager.get_tile(target_tile, &chunk_query) {
            if let Ok((target_machine_entity, _, _, _, _, _)) = machine_query.get(structure_entity)
            {
                transfer_pairs.push((source_machine_entity, target_machine_entity))
            }
        }
    }

    // we do the transfers based on the transfer pairs
    for (source_entity, target_entity) in transfer_pairs {
        let Ok(
            [
                (_, _, _, _, _, mut source_output_inventory),
                (_, _, _, _, mut target_input_inventory, _),
            ],
        ) = machine_query.get_many_mut([source_entity, target_entity])
        else {
            continue;
        };

        let item_stacks = source_output_inventory.0.remove_all_item_stack();
        for item_stack in item_stacks {
            if let Some(target_input_inventory) = &mut target_input_inventory {
                if !target_input_inventory.0.add(item_stack).is_ok() {
                    source_output_inventory.0.add(item_stack).expect(
                        "transfer didn't work and couldn't add items back in source_machine",
                    );
                }
            }
        }
    }
}

pub fn print_machine_inventories_system(
    query: Query<(&Name, Option<&InputInventory>, &mut OutputInventory), With<Machine>>,
) {
    for (name, input_inventory, output_inventory) in query.iter() {
        if let Some(input_inventory) = &input_inventory {
            println!(
                "{:?}: {:?} | {:?}",
                name, input_inventory.0.slots, output_inventory.0.slots
            )
        } else {
            println!("{:?}: {:?}", name, output_inventory.0.slots)
        }
    }
}

pub fn orient_machines_system(mut query: Query<(&Direction, &mut Transform), With<Machine>>) {
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
