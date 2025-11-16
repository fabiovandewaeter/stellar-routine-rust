#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemType {
    IronPlate,
    CopperPlate,

    IronGear,
    CopperWire,
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
