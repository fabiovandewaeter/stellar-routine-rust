use crate::items::{ItemStack, ItemType};
use bevy::prelude::*;
use std::mem::replace;

const DEFAULT_INVENTORY_CAPACITY: u32 = 10;

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

    pub fn enough_quantity(&self, item_type: ItemType, quantity: u32) -> bool {
        for slot in self.slots.iter() {
            if slot.item_type == item_type && slot.quantity >= quantity {
                return true;
            }
        }
        false
    }

    pub fn remove_quantity(&mut self, item_type: ItemType, quantity: u32) {
        if let Some(pos) = self
            .slots
            .iter()
            .position(|slot| slot.item_type == item_type && slot.quantity >= quantity)
        {
            self.slots[pos].quantity -= quantity;

            if self.slots[pos].quantity <= 0 {
                self.slots.remove(pos);
            }
        }
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
