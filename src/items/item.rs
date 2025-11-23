#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemType {
    IronOre,
    CopperOre,

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
