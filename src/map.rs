use avian2d::prelude::{CoefficientCombine, Collider, Friction, RigidBody};
use bevy::{
    prelude::*,
    sprite_render::{TileData, TilemapChunk, TilemapChunkTileData},
};
use rand::Rng;
use std::collections::HashMap;

use crate::{
    items::{ItemStack, ItemType, Quality, recipe::RecipeId},
    machine::{BeltMachine, CraftingMachine, Machine},
    units::{Direction, Unit, pathfinding::RecalculateFlowField},
};

pub const TILE_SIZE: Vec2 = Vec2 { x: 16.0, y: 16.0 };
pub const CHUNK_SIZE: UVec2 = UVec2 { x: 32, y: 32 };
pub const TILE_LAYER: f32 = -1.0;
pub const STRUCTURE_LAYER: f32 = 0.0;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapManager::default()).add_systems(
            PostStartup,
            spawn_one_chunk
        )
        .add_systems(FixedUpdate, 
        (
            // spawn_chunks_around_camera_system,
            spawn_chunks_around_units_system,
        )
            .chain())
        .add_systems(Update, update_tileset_image)
        // .add_systems(Update, ())
        ;
    }
}

/// absolute_coord = (5.5 * TILE_SIZE.X, 0.5 * TILE_SIZE.y) | coord = (5.5, 0.5) | tile_coord = (5, 0)
// #[derive(Component, Default, Debug, Clone, Copy, PartialEq)]
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct Coordinates {
    pub x: f32,
    pub y: f32,
}

/// absolute_coord = (5.5 * TILE_SIZE.X, 0.5 * TILE_SIZE.y) | coord = (5.5, 0.5) | tile_coord = (5, 0)
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct AbsoluteCoordinates {
    pub x: f32,
    pub y: f32,
}

impl From<Transform> for AbsoluteCoordinates {
    fn from(p: Transform) -> AbsoluteCoordinates {
        AbsoluteCoordinates {
            x: p.translation.x,
            y: p.translation.y,
        }
    }
}

impl From<AbsoluteCoordinates> for Vec2 {
    fn from(p: AbsoluteCoordinates) -> Vec2 {
        Vec2::new(p.x, p.y)
    }
}

/// absolute_coord = (5.5 * TILE_SIZE.X, 0.5 * TILE_SIZE.y) | coord = (5.5, 0.5) | tile_coord = (5, 0)
#[derive(Default, Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub struct TileCoordinates {
    pub x: i32,
    pub y: i32,
}

#[derive(Default, Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub struct LocalTileCoordinates {
    pub x: i32,
    pub y: i32,
}

/// chunk_coord : (1,1) is 1 right and 1 down
/// Chunkcoord {x: 2, y: 2} <=> TileCoordinates {x: 2*CHUNK_SIZE, y: 2*CHUNK_SIZE}
#[derive(Default, Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub struct ChunkCoordinates {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Default, Debug)]
pub struct StructureManager {
    pub structures: HashMap<LocalTileCoordinates, Entity>, // local TileCoordinates -> structure
}

/// Données spécifiques à chaque map
#[derive(Resource, Default)]
pub struct MapManager {
    pub chunks: HashMap<ChunkCoordinates, Entity>,
}
impl MapManager {
    pub fn get_tile(
        &self,
        tile: TileCoordinates,
        chunk_query: &Query<&StructureManager, With<TilemapChunk>>,
    ) -> Option<Entity> {
        let chunk_coord = tile_coord_to_chunk_coord(tile);
        if let Some(chunk_entity) = self.chunks.get(&chunk_coord) {
            if let Ok(structure_manager) = chunk_query.get(*chunk_entity) {
                let local_tile = tile_coord_to_local_tile_coord(tile, chunk_coord);
                return structure_manager.structures.get(&local_tile).copied();
            }
        }
        None
    }

    pub fn is_tile_walkable(
        &self,
        tile: TileCoordinates,
        chunk_query: &Query<&StructureManager, With<TilemapChunk>>,
    ) -> bool {
        self.get_tile(tile, chunk_query).is_none()
    }
}

#[derive(Component, Default)]
#[require(
    RigidBody::Static,
    Collider::rectangle(TILE_SIZE.x, TILE_SIZE.y),
    Friction {
        dynamic_coefficient: 0.0,
        static_coefficient: 0.0,
        combine_rule: CoefficientCombine::Multiply,
    },
)]
pub struct Structure;

#[derive(Component)]
pub struct Wall;

pub fn spawn_one_chunk(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut map_manager: ResMut<MapManager>,
    mut message_recalculate: MessageWriter<RecalculateFlowField>,
) -> () {
    let mut rng = rand::rng();
    let chunk_coord = ChunkCoordinates { x: 0, y: 0 };
    let mut structure_manager = StructureManager::default();
    for x in 0..CHUNK_SIZE.x {
        for y in 0..CHUNK_SIZE.y {
            let local_tile_coord = LocalTileCoordinates {
                x: x as i32,
                y: y as i32,
            };

            let is_wall = rng.random_bool(0.2);
            if is_wall && (local_tile_coord.x > 2) && (local_tile_coord.y > 2) {
                let tile_coord = local_tile_coord_to_tile_coord(local_tile_coord, chunk_coord);
                let target_coord = tile_coord_to_absolute_coord(tile_coord);
                let transform =
                    Transform::from_xyz(target_coord.x, target_coord.y, STRUCTURE_LAYER);
                let wall_entity = commands
                    .spawn((
                        Structure,
                        Wall,
                        Sprite::from_image(asset_server.load("structures/wall.png")),
                        transform,
                    ))
                    .id();
                structure_manager
                    .structures
                    .insert(local_tile_coord, wall_entity);
            }
        }
    }

    let local_tile_coord = LocalTileCoordinates { x: 1, y: 1 };
    let tile_coord = local_tile_coord_to_tile_coord(local_tile_coord, chunk_coord);
    let target_coord = tile_coord_to_absolute_coord(tile_coord);
    let transform = Transform::from_xyz(target_coord.x, target_coord.y, STRUCTURE_LAYER);
    let mut machine = Machine::default();
    let item_stack = ItemStack::new(ItemType::IronPlate, Quality::Perfect, 10);
    machine.input_inventory.add_item_stack(item_stack);
    let machine_entity = commands
        .spawn((
            Name::new("Belt machine"),
            Structure,
            // ProductionMachine::default(),
            machine,
            BeltMachine,
            Sprite::from_image(asset_server.load("structures/machine.png")),
            Direction::North,
            transform,
        ))
        .id();
    structure_manager
        .structures
        .insert(local_tile_coord, machine_entity);
    let local_tile_coord = LocalTileCoordinates { x: 1, y: 0 };
    let tile_coord = local_tile_coord_to_tile_coord(local_tile_coord, chunk_coord);
    let target_coord = tile_coord_to_absolute_coord(tile_coord);
    let transform = Transform::from_xyz(target_coord.x, target_coord.y, STRUCTURE_LAYER);
    let machine_entity = commands
        .spawn((
            Name::new("Crafting machine"),
            Structure,
            // ProductionMachine::default(),
            Machine::default(),
            CraftingMachine::new(RecipeId::IronPlateToIronGear),
            Sprite::from_image(asset_server.load("structures/machine.png")),
            Direction::South,
            transform,
        ))
        .id();
    structure_manager
        .structures
        .insert(local_tile_coord, machine_entity);

    message_recalculate.write_default();

    let tile_display_size = UVec2::splat(TILE_SIZE.x as u32);
    let chunk_center_x = (chunk_coord.x as f32 * CHUNK_SIZE.x as f32 + CHUNK_SIZE.x as f32 / 2.0)
        * tile_display_size.x as f32;
    let chunk_center_y = -(chunk_coord.y as f32 * CHUNK_SIZE.y as f32 + CHUNK_SIZE.y as f32 / 2.0)
        * tile_display_size.y as f32;

    let chunk_transform =
        Transform::from_translation(Vec3::new(chunk_center_x, chunk_center_y, TILE_LAYER));

    let tile_data: Vec<Option<TileData>> = (0..CHUNK_SIZE.element_product())
        // .map(|_| rng.random_range(0..5))
        .map(|_| rng.random_range(1..2))
        .map(|i| {
            if i == 0 {
                None
            } else {
                Some(TileData::from_tileset_index(i - 1))
            }
        })
        .collect();
    let chunk_entity = commands
        .spawn((
            TilemapChunk {
                chunk_size: CHUNK_SIZE,
                tile_display_size,
                tileset: asset_server.load("textures/array_texture.png"),
                ..default()
            },
            TilemapChunkTileData(tile_data),
            structure_manager,
            chunk_transform,
        ))
        .id();
    map_manager.chunks.insert(chunk_coord, chunk_entity);
}

fn update_tileset_image(
    chunk_query: Single<&TilemapChunk>,
    mut events: MessageReader<AssetEvent<Image>>,
    mut images: ResMut<Assets<Image>>,
) {
    let chunk = *chunk_query;
    for event in events.read() {
        if event.is_loaded_with_dependencies(chunk.tileset.id()) {
            let image = images.get_mut(&chunk.tileset).unwrap();
            image.reinterpret_stacked_2d_as_array(4);
        }
    }
}

// ========= coordinates conversion =========
/// absolute_coord = (5.5 * TILE_SIZE.X, 0.5 * TILE_SIZE.y) | coord = (5.5, 0.5) | tile_coord = (5, 0)
/// chunk_coord : (1,1) is 1 right and 1 down

pub fn local_tile_coord_to_tile_coord(
    local_tile_coord: LocalTileCoordinates,
    chunk_coord: ChunkCoordinates,
) -> TileCoordinates {
    TileCoordinates {
        x: local_tile_coord.x + chunk_coord.x * (CHUNK_SIZE.x as i32),
        y: local_tile_coord.y + chunk_coord.y * (CHUNK_SIZE.y as i32),
    }
}

// Conversion coordonnées logiques -> monde ; (5.5, 0.5) => (5.5 * TILE_SIZE.x, 0.5 * TILE_SIZE.y)
pub fn coord_to_absolute_coord(coord: Coordinates) -> AbsoluteCoordinates {
    AbsoluteCoordinates {
        x: (coord.x + 0.5) * TILE_SIZE.x as f32,
        y: -((coord.y + 0.5) * TILE_SIZE.y as f32),
        // x: (coord.x) * TILE_SIZE.x as f32,
        // y: -((coord.y) * TILE_SIZE.y as f32),
    }
}

pub fn tile_coord_to_local_tile_coord(
    tile_coord: TileCoordinates,
    chunk_coord: ChunkCoordinates,
) -> LocalTileCoordinates {
    LocalTileCoordinates {
        x: tile_coord.x - chunk_coord.x * (CHUNK_SIZE.x as i32),
        y: tile_coord.y - chunk_coord.y * (CHUNK_SIZE.y as i32),
    }
}

// // adds 0.5 to coordinates to make entities spawn based on the corner of there sprite and not the center
pub fn tile_coord_to_absolute_coord(tile_coord: TileCoordinates) -> AbsoluteCoordinates {
    AbsoluteCoordinates {
        x: tile_coord.x as f32 * TILE_SIZE.x + TILE_SIZE.x * 0.5,
        y: -(tile_coord.y as f32 * TILE_SIZE.y + TILE_SIZE.y * 0.5),
        // x: tile_coord.x as f32 * TILE_SIZE.x,
        // y: -(tile_coord.y as f32 * TILE_SIZE.y),
    }
}

pub fn tile_coord_to_coord(tile_coord: TileCoordinates) -> Coordinates {
    Coordinates {
        x: tile_coord.x as f32,
        y: tile_coord.y as f32,
    }
}

// (5.5, 0.5) => (5, 0)
pub fn coord_to_tile_coord(coord: Coordinates) -> TileCoordinates {
    TileCoordinates {
        x: coord.x.floor() as i32,
        y: coord.y.floor() as i32,
    }
}

// Conversion monde -> coordonnées logiques
pub fn absolute_coord_to_coord(absolute_coord: AbsoluteCoordinates) -> Coordinates {
    Coordinates {
        // x: absolute_coord.x as f32 / TILE_SIZE.x,
        // y: (-absolute_coord.y as f32) / TILE_SIZE.y,
        x: absolute_coord.x as f32 / TILE_SIZE.x - 0.5,
        y: (-absolute_coord.y as f32) / TILE_SIZE.y - 0.5,
    }
}

// Conversion monde -> coordonnées logiques
pub fn absolute_coord_to_tile_coord(absolute_coord: AbsoluteCoordinates) -> TileCoordinates {
    TileCoordinates {
        // x: ((absolute_coord.x as f32 / TILE_SIZE.x) - 0.5).floor() as i32,
        // y: (((-absolute_coord.y as f32) / TILE_SIZE.y) - 0.5).floor() as i32,
        x: ((absolute_coord.x as f32 / TILE_SIZE.x) - 0.5).round() as i32,
        y: (((-absolute_coord.y as f32) / TILE_SIZE.y) - 0.5).round() as i32,
    }
}

/// Convertit une coordition monde (pixels) en coordition de chunk.
pub fn absolute_coord_to_chunk_coord(absolute_coord: AbsoluteCoordinates) -> ChunkCoordinates {
    ChunkCoordinates {
        x: (absolute_coord.x as f32 / (CHUNK_SIZE.x as f32 * TILE_SIZE.x)).floor() as i32,
        y: ((-absolute_coord.y as f32) / (CHUNK_SIZE.y as f32 * TILE_SIZE.y)).floor() as i32,
    }
}

pub fn chunk_coord_to_tile_coord(chunk_coord: ChunkCoordinates) -> TileCoordinates {
    TileCoordinates {
        x: chunk_coord.x * CHUNK_SIZE.x as i32,
        y: chunk_coord.y * CHUNK_SIZE.y as i32,
    }
}

pub fn tile_coord_to_chunk_coord(tile_coord: TileCoordinates) -> ChunkCoordinates {
    ChunkCoordinates {
        // x: tile_coord.x / CHUNK_SIZE.x as i32,
        // y: tile_coord.y / CHUNK_SIZE.y as i32,
        x: tile_coord.x.div_euclid(CHUNK_SIZE.x as i32),
        y: tile_coord.y.div_euclid(CHUNK_SIZE.y as i32),
    }
}

pub fn coord_to_chunk_coord(coord: Coordinates) -> ChunkCoordinates {
    ChunkCoordinates {
        x: (coord.x / CHUNK_SIZE.x as f32).floor() as i32,
        y: (coord.y / CHUNK_SIZE.y as f32).floor() as i32,
    }
}
// ==========================================

// fn spawn_chunks_around_camera_system(
//     mut commands: Commands,
//     asset_server: Res<AssetServer>,
//     camera_query: Query<(&Transform), With<Camera>>,
//     mut multi_map_manager: ResMut<MultiMapManager>,
// ) {
//     const SIZE: i32 = 4;
//     if let Ok((transform, camera_map)) = camera_query.single() {
//         let camera_chunk_coord = world_coord_to_rounded_chunk(transform.translation.xy());
//         let active_map_id = camera_map.map_id;

//         // Récupérer les données de la map de la caméra
//         if let Some(map_data) = multi_map_manager.maps.get_mut(&active_map_id) {
//             for y in (camera_chunk_coord.y - SIZE)..(camera_chunk_coord.y + SIZE) {
//                 for x in (camera_chunk_coord.x - SIZE)..(camera_chunk_coord.x + SIZE) {
//                     let chunk_coord = Chunkcoord { x, y };
//                     if !map_data
//                         .chunk_manager
//                         .spawned_chunks
//                         .contains_key(&chunk_coord)
//                     {
//                         let entity = spawn_chunk(
//                             &mut commands,
//                             &asset_server,
//                             &mut map_data.structure_manager,
//                             chunk_coord,
//                             active_map_id,
//                         );
//                         map_data
//                             .chunk_manager
//                             .spawned_chunks
//                             .insert(chunk_coord, entity);
//                     }
//                 }
//             }
//         }
//     }
// }

fn spawn_chunks_around_units_system(
    unit_query: Query<&Transform, With<Unit>>,
    chunk_query: Query<&StructureManager, With<TilemapChunk>>,
    mut map_manager: ResMut<MapManager>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut message_recalculate: MessageWriter<RecalculateFlowField>,
) {
    const SIZE: i32 = 2;

    for unit_transform in unit_query.iter() {
        let unit_chunk_coord = absolute_coord_to_chunk_coord((*unit_transform).into());
        for y in (unit_chunk_coord.y - SIZE)..(unit_chunk_coord.y + SIZE) {
            for x in (unit_chunk_coord.x - SIZE)..(unit_chunk_coord.x + SIZE) {
                let chunk_coord = ChunkCoordinates { x, y };
                if map_manager.chunks.contains_key(&chunk_coord) {
                    continue;
                }

                let mut rng = rand::rng();
                let mut structure_manager = StructureManager::default();
                for x in 0..CHUNK_SIZE.x {
                    for y in 0..CHUNK_SIZE.y {
                        let local_tile_coord = LocalTileCoordinates {
                            x: x as i32,
                            y: y as i32,
                        };

                        let is_wall = rng.random_bool(0.2);
                        if is_wall && (local_tile_coord.x > 2) && (local_tile_coord.y > 2) {
                            let tile_coord =
                                local_tile_coord_to_tile_coord(local_tile_coord, chunk_coord);
                            let target_coord = tile_coord_to_absolute_coord(tile_coord);
                            let transform = Transform::from_xyz(
                                target_coord.x,
                                target_coord.y,
                                STRUCTURE_LAYER,
                            );
                            let wall_entity = commands
                                .spawn((
                                    Structure,
                                    Wall,
                                    Sprite::from_image(asset_server.load("structures/wall.png")),
                                    transform,
                                ))
                                .id();
                            structure_manager
                                .structures
                                .insert(local_tile_coord, wall_entity);
                        }
                    }
                }

                message_recalculate.write_default();

                let tile_display_size = UVec2::splat(TILE_SIZE.x as u32);
                let chunk_center_x = (chunk_coord.x as f32 * CHUNK_SIZE.x as f32
                    + CHUNK_SIZE.x as f32 / 2.0)
                    * tile_display_size.x as f32;
                let chunk_center_y = -(chunk_coord.y as f32 * CHUNK_SIZE.y as f32
                    + CHUNK_SIZE.y as f32 / 2.0)
                    * tile_display_size.y as f32;

                let chunk_transform = Transform::from_translation(Vec3::new(
                    chunk_center_x,
                    chunk_center_y,
                    TILE_LAYER,
                ));

                let tile_data: Vec<Option<TileData>> = (0..CHUNK_SIZE.element_product())
                    // .map(|_| rng.random_range(0..5))
                    .map(|_| rng.random_range(1..2))
                    .map(|i| {
                        if i == 0 {
                            None
                        } else {
                            Some(TileData::from_tileset_index(i - 1))
                        }
                    })
                    .collect();
                let chunk_entity = commands
                    .spawn((
                        TilemapChunk {
                            chunk_size: CHUNK_SIZE,
                            tile_display_size,
                            tileset: asset_server.load("textures/array_texture.png"),
                            ..default()
                        },
                        TilemapChunkTileData(tile_data),
                        structure_manager,
                        chunk_transform,
                    ))
                    .id();
                map_manager.chunks.insert(chunk_coord, chunk_entity);
            }
        }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     const EPS: f32 = 1e-6;

//     fn approx_coord(a: AbsoluteCoordinates, b: AbsoluteCoordinates) -> bool {
//         (a.x - b.x).abs() < EPS && (a.y - b.y).abs() < EPS
//     }

//     fn approx_tile(a: Coordinates, b: Coordinates) -> bool {
//         (a.x - b.x).abs() < EPS && (a.y - b.y).abs() < EPS
//     }

//     #[test]
//     fn local_tile_coord_to_tile_coord_test() {
//         let chunk_coord = ChunkCoordinates { x: 0, y: 0 };

//         let tile_coord =
//             local_tile_coord_to_tile_coord(TileCoordinates { x: 0, y: 0 }, chunk_coord);
//         assert_eq!(tile_coord, TileCoordinates { x: 0, y: 0 });
//         let tile_coord =
//             local_tile_coord_to_tile_coord(TileCoordinates { x: 2, y: 2 }, chunk_coord);
//         assert_eq!(tile_coord, TileCoordinates { x: 2, y: 2 });

//         let chunk_coord = ChunkCoordinates { x: 2, y: 2 };

//         let tile_coord =
//             local_tile_coord_to_tile_coord(TileCoordinates { x: 0, y: 0 }, chunk_coord);
//         assert_eq!(
//             tile_coord,
//             TileCoordinates {
//                 x: 2 * CHUNK_SIZE.x as i32,
//                 y: 2 * CHUNK_SIZE.y as i32
//             }
//         );
//         let tile_coord =
//             local_tile_coord_to_tile_coord(TileCoordinates { x: 2, y: 2 }, chunk_coord);
//         assert_eq!(
//             tile_coord,
//             TileCoordinates {
//                 x: 2 * CHUNK_SIZE.x as i32 + 2,
//                 y: 2 * CHUNK_SIZE.y as i32 + 2
//             }
//         );
//     }
// }
