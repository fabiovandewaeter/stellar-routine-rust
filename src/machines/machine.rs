use bevy::prelude::*;

use crate::map::Structure;

#[derive(Component)]
#[require(Structure)]
pub struct ProductionMachine {
    pub craft_time_ticks: u64,
    pub progress_ticks: u64,
}
