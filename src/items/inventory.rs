use std::mem::replace;

use bevy::prelude::*;

const DEFAULT_INVENTORY_CAPACITY: u32 = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemType {
    IronPlate,
    CopperPlate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Quality {
    Perfect,
    Standard,
    Defective,
}

impl Default for Quality {
    fn default() -> Self {
        Quality::Standard
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ItemStack {
    pub item_type: ItemType,
    pub quality: Quality,
    pub quantity: u32,
}
impl ItemStack {
    pub fn new(item_type: ItemType, quality: Quality, quantity: u32) -> Self {
        Self {
            item_type,
            quality,
            quantity,
        }
    }

    pub fn can_stack_with(&self, other: &Self) -> bool {
        self.item_type == other.item_type && self.quality == other.quality
    }
}

#[derive(Component)]
pub struct Inventory {
    pub slots: Vec<ItemStack>,
    pub capacity: u32,
}
impl Inventory {
    pub fn add_item_stack(&mut self, item_stack: ItemStack) -> Result<(), ()> {
        for slot in self.slots.iter_mut() {
            if slot.can_stack_with(&item_stack) {
                slot.quantity += item_stack.quantity;
                return Ok(());
            }
        }

        // if no compatible slot found, add to an empty slot
        if self.slots.len() < self.capacity as usize {
            self.slots.push(item_stack);
            return Ok(());
        }

        Err(())
    }

    pub fn remove_all_item_stack(&mut self) -> Vec<ItemStack> {
        replace(&mut self.slots, Vec::new())
    }
}

impl Default for Inventory {
    fn default() -> Self {
        Self {
            slots: Vec::new(),
            capacity: DEFAULT_INVENTORY_CAPACITY,
        }
    }
}
