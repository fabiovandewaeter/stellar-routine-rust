use avian2d::prelude::{CoefficientCombine, Collider, Friction, RigidBody};
use bevy::{
    prelude::*,
    sprite_render::{TileData, TilemapChunk, TilemapChunkTileData},
};
use rand::Rng;
use std::collections::HashMap;

use crate::units::pathfinding::RecalculateFlowField;

pub const TILE_SIZE: Vec2 = Vec2 { x: 16.0, y: 16.0 };
pub const CHUNK_SIZE: UVec2 = UVec2 { x: 32, y: 32 };
pub const TILE_LAYER_LEVEL: f32 = -1.0;
pub const STRUCTURE_LAYER_LEVEL: f32 = 0.0;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(MapManager::default()).add_systems(
            PostStartup,
            spawn_one_chunk
            // FixedUpdate,
            // (
            //     spawn_chunks_around_camera_system,
            //     spawn_chunks_around_units_system,
            // )
            // .chain()
        )
        .add_systems(Update, update_tileset_image)
        // .add_systems(Update, ())
        ;
    }
}

/// absolute_coord = (5.5 * TILE_SIZE.X, 0.5 * TILE_SIZE.y) | coord = (5.5, 0.5) | tile_coord = (5, 0)
#[derive(Component, Default, Debug, Clone, Copy, PartialEq)]
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

impl TileCoordinates {
    pub fn to_chunk_coord(self) -> ChunkCoordinates {
        ChunkCoordinates {
            x: self.x * CHUNK_SIZE.x as i32,
            y: self.y * CHUNK_SIZE.y as i32,
        }
    }
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
    pub structures: HashMap<TileCoordinates, Entity>, // local TileCoordinates -> structure
}

/// Données spécifiques à chaque map
#[derive(Resource, Default)]
pub struct MapManager {
    pub chunks: HashMap<ChunkCoordinates, Entity>,
}

#[derive(Component)]
#[require(Coordinates)]
pub struct Structure;

#[derive(Component)]
#[require(
    RigidBody::Static,
    Collider::rectangle(TILE_SIZE.x, TILE_SIZE.y),
    Friction {
        dynamic_coefficient: 0.0,
        static_coefficient: 0.0,
        combine_rule: CoefficientCombine::Multiply,
    },
)]
pub struct Wall;

pub fn is_tile_walkable(
    tile: TileCoordinates,
    map_manager: &MapManager,
    chunks_query: &Query<&StructureManager, With<TilemapChunk>>,
) -> bool {
    if let Some(chunk_entity) = map_manager.chunks.get(&tile_coord_to_chunk_coord(tile)) {
        if let Ok(structure_manager) = chunks_query.get(*chunk_entity) {
            return !structure_manager.structures.contains_key(&tile);
        }
    }
    false
}

pub fn spawn_one_chunk(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut map_manager: ResMut<MapManager>,
    mut message_recalculate: MessageWriter<RecalculateFlowField>,
) -> () {
    let mut rng = rand::rng();
    let chunk_coord = ChunkCoordinates { x: 0, y: 0 };
    let mut chunk_was_modified = false;
    let mut structure_manager = StructureManager::default();
    for x in 0..CHUNK_SIZE.x {
        for y in 0..CHUNK_SIZE.y {
            let local_tile_coord = TileCoordinates {
                x: x as i32,
                y: y as i32,
            };

            let is_wall = rng.random_bool(0.2);
            if is_wall && (local_tile_coord.x > 2) && (local_tile_coord.y > 2) {
                let tile_coord = local_tile_coord_to_tile_coord(local_tile_coord, chunk_coord);
                let coord = tile_coord_to_coord(tile_coord);

                let target_coord = coord_to_absolute_coord(coord);
                let mut transform = Transform::default();
                transform.translation.x = target_coord.x;
                transform.translation.y = target_coord.y;
                let wall_entity = commands
                    .spawn((
                        Structure,
                        Wall,
                        Sprite::from_image(asset_server.load("structures/wall.png")),
                        coord,
                        transform,
                    ))
                    .id();
                structure_manager
                    .structures
                    .insert(local_tile_coord, wall_entity);

                chunk_was_modified = true;
            }
        }
    }

    if chunk_was_modified {
        message_recalculate.write_default();
    }

    let tile_display_size = UVec2::splat(TILE_SIZE.x as u32);
    let chunk_center_x = (chunk_coord.x as f32 * CHUNK_SIZE.x as f32 + CHUNK_SIZE.x as f32 / 2.0)
        * tile_display_size.x as f32;
    let chunk_center_y = -(chunk_coord.y as f32 * CHUNK_SIZE.y as f32 + CHUNK_SIZE.y as f32 / 2.0)
        * tile_display_size.y as f32;

    let chunk_transform = Transform::from_translation(Vec3::new(
        chunk_center_x,
        chunk_center_y,
        //STRUCTURE_LAYER_LEVEL, // ou TILE_LAYER_LEVEL si tu veux que les tiles soient derrière/avant
        -1.0,
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
    local_tile_coord: TileCoordinates,
    chunk_coord: ChunkCoordinates,
) -> TileCoordinates {
    TileCoordinates {
        x: chunk_coord.x * (CHUNK_SIZE.x as i32) + local_tile_coord.x,
        y: chunk_coord.y * (CHUNK_SIZE.y as i32) + local_tile_coord.y,
    }
}

// Conversion coordonnées logiques -> monde ; (5.5, 0.5) => (5.5 * TILE_SIZE.x, 0.5 * TILE_SIZE.y)
pub fn coord_to_absolute_coord(coord: Coordinates) -> AbsoluteCoordinates {
    AbsoluteCoordinates {
        x: (coord.x + 0.5) * TILE_SIZE.x as f32,
        y: -((coord.y + 0.5) * TILE_SIZE.y as f32),
    }
}

// // adds 0.5 to coordinates to make entities spawn based on the corner of there sprite and not the center
// pub fn tile_coord_to_absolute_coord(tile_coord: TileCoordinates) -> AbsoluteCoordinates {
//     AbsoluteCoordinates {
//         x: tile_coord.x as f32 * TILE_SIZE.x + TILE_SIZE.x * 0.5,
//         y: -(tile_coord.y as f32 * TILE_SIZE.y + TILE_SIZE.y * 0.5),
//         // x: tile_coord.x as f32 * TILE_SIZE.x,
//         // y: -(tile_coord.y as f32 * TILE_SIZE.y),
//     }
// }

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
        x: absolute_coord.x as f32 / TILE_SIZE.x,
        y: (-absolute_coord.y as f32) / TILE_SIZE.y,
    }
}

// Conversion monde -> coordonnées logiques
pub fn absolute_coord_to_tile_coord(absolute_coord: AbsoluteCoordinates) -> TileCoordinates {
    TileCoordinates {
        x: (absolute_coord.x as f32 / TILE_SIZE.x).floor() as i32,
        y: ((-absolute_coord.y as f32) / TILE_SIZE.y).floor() as i32,
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
        x: tile_coord.x / CHUNK_SIZE.x as i32,
        y: tile_coord.y / CHUNK_SIZE.y as i32,
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

// fn spawn_chunks_around_units_system(
//     mut commands: Commands,
//     asset_server: Res<AssetServer>,
//     unit_query: Query<(&Coordinates), With<Unit>>,
//     mut multi_map_manager: ResMut<MultiMapManager>,
//     camera_query: Query<With<Camera>>,
// ) {
//     const SIZE: i32 = 2;

//     // Récupérer la map active (celle de la caméra)
//     let active_map_id = if let Ok(camera_map) = camera_query.get_single() {
//         camera_map.map_id
//     } else {
//         MapId(0) // Fallback vers la map principale
//     };

//     // Ne spawner des chunks que pour les unités sur la map active
//     for (unit_tile_coord, current_map) in unit_query.iter() {
//         if current_map.map_id != active_map_id {
//             continue; // Ignore les unités sur d'autres maps
//         }

//         let unit_chunk_coord = rounded_tile_coord_to_rounded_chunk(*unit_tile_coord);

//         if let Some(map_data) = multi_map_manager.maps.get_mut(&current_map.map_id) {
//             for y in (unit_chunk_coord.y - SIZE)..(unit_chunk_coord.y + SIZE) {
//                 for x in (unit_chunk_coord.x - SIZE)..(unit_chunk_coord.x + SIZE) {
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
//                             current_map.map_id,
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

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f32 = 1e-6;

    fn approx_coord(a: AbsoluteCoordinates, b: AbsoluteCoordinates) -> bool {
        (a.x - b.x).abs() < EPS && (a.y - b.y).abs() < EPS
    }

    fn approx_tile(a: Coordinates, b: Coordinates) -> bool {
        (a.x - b.x).abs() < EPS && (a.y - b.y).abs() < EPS
    }

    #[test]
    fn local_tile_coord_to_tile_coord_test() {
        let chunk_coord = ChunkCoordinates { x: 0, y: 0 };

        let tile_coord =
            local_tile_coord_to_tile_coord(TileCoordinates { x: 0, y: 0 }, chunk_coord);
        assert_eq!(tile_coord, TileCoordinates { x: 0, y: 0 });
        let tile_coord =
            local_tile_coord_to_tile_coord(TileCoordinates { x: 2, y: 2 }, chunk_coord);
        assert_eq!(tile_coord, TileCoordinates { x: 2, y: 2 });

        let chunk_coord = ChunkCoordinates { x: 2, y: 2 };

        let tile_coord =
            local_tile_coord_to_tile_coord(TileCoordinates { x: 0, y: 0 }, chunk_coord);
        assert_eq!(
            tile_coord,
            TileCoordinates {
                x: 2 * CHUNK_SIZE.x as i32,
                y: 2 * CHUNK_SIZE.y as i32
            }
        );
        let tile_coord =
            local_tile_coord_to_tile_coord(TileCoordinates { x: 2, y: 2 }, chunk_coord);
        assert_eq!(
            tile_coord,
            TileCoordinates {
                x: 2 * CHUNK_SIZE.x as i32 + 2,
                y: 2 * CHUNK_SIZE.y as i32 + 2
            }
        );
    }
}
