use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ItemType {
    IronOre,
    CopperOre,

    IronPlate,
    CopperPlate,

    IronGear,
    CopperWire,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
