use crate::{
    items::{ItemStack, ItemType, Quality},
    map::machine::DEFAULT_ACTION_TIME_TICKS,
};
use bevy::ecs::resource::Resource;
use std::collections::HashMap;

const DEFAULT_CRAFT_TIME_TICKS: u64 = DEFAULT_ACTION_TIME_TICKS;

// #[derive(Debug, Clone, PartialEq, Eq)]
// pub struct RecipeItemStack {
//     pub item_type: ItemType,
//     pub quantity: u32,
// }

#[derive(Debug, Clone)]
pub struct Recipe {
    pub inputs: Vec<ItemStack>,
    pub outputs: Vec<ItemStack>,
    pub base_craft_time_ticks: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RecipeId {
    IronPlateToIronGear,
    CopperPlateToCopperWire,
}

#[derive(Resource)]
pub struct RecipeBook(pub HashMap<RecipeId, Recipe>);
impl Default for RecipeBook {
    fn default() -> Self {
        let mut recipes = HashMap::new();

        recipes.insert(
            RecipeId::IronPlateToIronGear,
            Recipe {
                inputs: vec![ItemStack {
                    item_type: ItemType::IronPlate,
                    quantity: 2,
                    quality: Quality::Standard,
                }],
                outputs: vec![ItemStack {
                    item_type: ItemType::IronGear,
                    quantity: 1,
                    quality: Quality::Standard,
                }],
                base_craft_time_ticks: DEFAULT_CRAFT_TIME_TICKS,
            },
        );

        RecipeBook(recipes)
    }
}
