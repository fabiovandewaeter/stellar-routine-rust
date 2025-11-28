use crate::items::{ItemType, Quality};
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::mem::replace;

// const DEFAULT_ITEM_STACK_LIMIT: u32 = 10000;
// const DEFAULT_INVENTORY_SLOTS_QUANTITY_LIMIT: u32 = 10;
const DEFAULT_ITEM_STACK_LIMIT: u32 = 10;
const DEFAULT_INVENTORY_SLOTS_QUANTITY_LIMIT: u32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
        self.item_type == other.item_type
            && self.quality == other.quality
            && (self.quantity + other.quantity) <= DEFAULT_ITEM_STACK_LIMIT
    }
}

#[derive(Debug, Clone, Component, Serialize, Deserialize)]
pub struct Inventory {
    pub slots: Vec<ItemStack>,
    pub slots_quantity_limit: u32,
}
impl Inventory {
    pub fn add(&mut self, item_stack: ItemStack) -> Result<(), ()> {
        // TODO: see if make that we fill existing slots as much as possible event if everything wont fit BUT we have to make sure it's ok to not add everything
        for slot in self.slots.iter_mut() {
            if slot.can_stack_with(&item_stack) {
                slot.quantity += item_stack.quantity;
                return Ok(());
            }
        }
        // if no compatible slot found, add to an empty slot
        if self.slots.len() < self.slots_quantity_limit as usize {
            self.slots.push(item_stack);
            return Ok(());
        }
        Err(())
    }

    pub fn remove_all_item_stack(&mut self) -> Vec<ItemStack> {
        replace(&mut self.slots, Vec::new())
    }

    /// checks all slots to see if there A UNIQ SLOT is enough quantity of specifil ItemType and Quality
    pub fn enough_quantity(&self, item_stack: ItemStack) -> bool {
        for slot in self.slots.iter() {
            if slot.item_type == item_stack.item_type
                && slot.quality == item_stack.quality
                && slot.quantity >= item_stack.quantity
            {
                return true;
            }
        }
        false
    }

    /// returns true if there is at least an empty slot or a slot of same type and quality with enough room for the desired quantity to add
    pub fn enough_room(&self, item_stack: ItemStack) -> bool {
        if self.slots.len() < self.slots_quantity_limit as usize {
            return true;
        }
        for slot in self.slots.iter() {
            if slot.can_stack_with(&item_stack) {
                return true;
            }
        }
        false
    }

    pub fn remove_quantity(&mut self, item_stack: ItemStack) {
        if let Some(pos) = self.slots.iter().position(|slot| {
            slot.item_type == item_stack.item_type
                && slot.quality == item_stack.quality
                && slot.quantity >= item_stack.quantity
        }) {
            self.slots[pos].quantity -= item_stack.quantity;

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
            slots_quantity_limit: DEFAULT_INVENTORY_SLOTS_QUANTITY_LIMIT,
        }
    }
}
#[derive(Component, Default)]
pub struct InputInventory(pub Inventory);
#[derive(Component, Default)]
pub struct OutputInventory(pub Inventory);

#[cfg(test)]
mod tests {
    use super::*;

    // ItemStack
    #[test]
    fn test_can_stack_with() {
        let item_stack = ItemStack {
            item_type: ItemType::IronOre,
            quality: Quality::Standard,
            quantity: DEFAULT_ITEM_STACK_LIMIT - 1,
        };

        let other = ItemStack {
            item_type: ItemType::IronOre,
            quality: Quality::Standard,
            quantity: 1,
        };
        assert!(item_stack.can_stack_with(&other));

        let other_too_much = ItemStack {
            item_type: ItemType::IronOre,
            quality: Quality::Standard,
            quantity: 2,
        };
        assert!(!item_stack.can_stack_with(&other_too_much));

        let other_different_item_type = ItemStack {
            item_type: ItemType::IronGear,
            quality: Quality::Standard,
            quantity: 1,
        };
        assert!(!item_stack.can_stack_with(&other_different_item_type));

        let other_different_quality = ItemStack {
            item_type: ItemType::IronGear,
            quality: Quality::Standard,
            quantity: 1,
        };
        assert!(!item_stack.can_stack_with(&other_different_quality));
    }

    // Inventory
    #[test]
    fn test_add() {
        let mut inventory = Inventory::default();

        assert_eq!(inventory.slots.len(), 0);
        let item_stack = ItemStack {
            item_type: ItemType::IronOre,
            quality: Quality::Standard,
            quantity: 1,
        };
        assert!(inventory.add(item_stack).is_ok());
        assert_eq!(inventory.slots.len(), 1);

        // tries to fill existing compatible slots first
        assert!(inventory.add(item_stack).is_ok());
        assert_eq!(inventory.slots.len(), 1);
        assert_eq!(inventory.slots.get(0).unwrap().quantity, 2);

        // add to a free slot if existing compatible slots are full
        let item_stack_fill_existing_slot = ItemStack {
            item_type: ItemType::IronOre,
            quality: Quality::Standard,
            quantity: DEFAULT_ITEM_STACK_LIMIT - 2,
        };
        assert!(inventory.add(item_stack_fill_existing_slot).is_ok());
        assert_eq!(inventory.slots.len(), 1);
        assert_eq!(
            inventory.slots.get(0).unwrap().quantity,
            DEFAULT_ITEM_STACK_LIMIT
        );
        assert!(inventory.add(item_stack).is_ok());
        assert_eq!(inventory.slots.len(), 2);
        let item_stack_different_type = ItemStack {
            item_type: ItemType::CopperOre,
            quality: Quality::Standard,
            quantity: 1,
        };
        assert!(inventory.add(item_stack_different_type).is_ok());
        assert_eq!(inventory.slots.len(), 3);
        let item_stack_different_quality = ItemStack {
            item_type: ItemType::CopperOre,
            quality: Quality::Perfect,
            quantity: 1,
        };
        assert!(inventory.add(item_stack_different_quality).is_ok());
        assert_eq!(inventory.slots.len(), 4);

        // can add up to inventory.slots_quantity_limit slots
        let item_stack_fill_empty_slot = ItemStack {
            item_type: ItemType::IronPlate,
            quality: Quality::Standard,
            quantity: DEFAULT_ITEM_STACK_LIMIT,
        };
        for _ in inventory.slots.len()..inventory.slots_quantity_limit as usize - 1 {
            assert!(inventory.add(item_stack_fill_empty_slot).is_ok());
        }
        assert_eq!(
            inventory.slots.len(),
            inventory.slots_quantity_limit as usize - 1
        );
        assert!(inventory.add(item_stack_fill_empty_slot).is_ok());
        assert_eq!(
            inventory.slots.len(),
            inventory.slots_quantity_limit as usize
        );
        let item_stack_too_much_quantity = ItemStack {
            item_type: ItemType::IronPlate,
            quality: Quality::Standard,
            quantity: DEFAULT_ITEM_STACK_LIMIT * 10,
        };
        assert!(inventory.add(item_stack_too_much_quantity).is_err());

        // don't add if there is no more free slots nor existing compatible slots
        let item_stack_new_category = ItemStack {
            item_type: ItemType::CopperWire,
            quality: Quality::Defective,
            quantity: 1,
        };
        assert!(inventory.add(item_stack_new_category).is_err());
    }

    #[test]
    fn test_enough_quantity() {
        let mut inventory = Inventory::default();

        let item_stack = ItemStack {
            item_type: ItemType::IronOre,
            quality: Quality::Standard,
            quantity: 1,
        };
        assert!(!inventory.enough_quantity(item_stack));

        inventory.slots.push(ItemStack {
            item_type: ItemType::IronOre,
            quality: Quality::Standard,
            quantity: 1,
        });
        assert!(inventory.enough_quantity(item_stack));

        let item_stack_wrong_type = ItemStack {
            item_type: ItemType::CopperOre,
            quality: Quality::Standard,
            quantity: 1,
        };
        assert!(!inventory.enough_quantity(item_stack_wrong_type));

        let item_stack_wrong_quality = ItemStack {
            item_type: ItemType::IronOre,
            quality: Quality::Perfect,
            quantity: 1,
        };
        assert!(!inventory.enough_quantity(item_stack_wrong_quality));
    }

    #[test]
    fn test_enough_room() {
        let mut inventory = Inventory::default();

        let item_stack = ItemStack {
            item_type: ItemType::IronOre,
            quality: Quality::Standard,
            quantity: 1,
        };
        assert!(inventory.enough_room(item_stack));

        // leave only 2 empty slots and add 2 special item stack to test ItemType and Quality
        let item_stack_fill_empty_slot = ItemStack {
            item_type: ItemType::IronPlate,
            quality: Quality::Standard,
            quantity: DEFAULT_ITEM_STACK_LIMIT,
        };
        for _ in inventory.slots.len()..inventory.slots_quantity_limit as usize - 2 {
            inventory.slots.push(item_stack_fill_empty_slot);
        }
        let item_stack_different_type = ItemStack {
            item_type: ItemType::CopperOre,
            quality: Quality::Standard,
            quantity: 1,
        };
        inventory.slots.push(item_stack_different_type);
        let item_stack_different_quality = ItemStack {
            item_type: ItemType::CopperOre,
            quality: Quality::Perfect,
            quantity: 1,
        };
        inventory.slots.push(item_stack_different_quality);
        assert_eq!(
            inventory.slots.len(),
            inventory.slots_quantity_limit as usize
        );

        // enough room when no empty slots BUT existing compatible slots
        assert!(inventory.enough_room(item_stack_different_type));
        assert!(inventory.enough_room(item_stack_different_quality));

        // not enough room when no empty slots or existing compatible slots
        let item_stack_different_type_and_quality = ItemStack {
            item_type: ItemType::CopperWire,
            quality: Quality::Defective,
            quantity: 1,
        };
        assert!(!inventory.enough_room(item_stack_different_type_and_quality));
    }

    #[test]
    fn test_remove_quantity() {
        let mut inventory = Inventory::default();

        let item_stack = ItemStack {
            item_type: ItemType::IronOre,
            quality: Quality::Standard,
            quantity: 2,
        };
        inventory.slots.push(item_stack);
        assert_eq!(inventory.slots.get(0).unwrap().quantity, 2);
        let item_stack_quantity_to_remove = ItemStack {
            item_type: ItemType::IronOre,
            quality: Quality::Standard,
            quantity: 1,
        };
        inventory.remove_quantity(item_stack_quantity_to_remove);
        assert_eq!(inventory.slots.get(0).unwrap().quantity, 1);
    }
}
// REGARDER pourquoi ça ajouter pas d'items dans l'output des mining machine
