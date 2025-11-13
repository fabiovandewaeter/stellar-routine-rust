use bevy::{
    prelude::*,
    sprite_render::{TileData, TilemapChunk, TilemapChunkTileData},
};
use rand::Rng;
use std::collections::HashMap;

pub const TILE_SIZE: Vec2 = Vec2 { x: 16.0, y: 16.0 };
pub const CHUNK_SIZE: UVec2 = UVec2 { x: 32, y: 32 };
pub const TILE_LAYER_LEVEL: f32 = -1.0;
pub const STRUCTURE_LAYER_LEVEL: f32 = 0.0;

pub struct MapPlugin;

impl Plugin for MapPlugin {
    fn build(&self, app: &mut bevy::app::App) {
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

/// absolute_pos = (5.5 * TILE_SIZE.X, 0.5 * TILE_SIZE.y) | pos = (5.5, 0.5) | grid_pos = (5, 0)
#[derive(Component, Default, Debug, Clone, Copy, PartialEq)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

/// absolute_pos = (5.5 * TILE_SIZE.X, 0.5 * TILE_SIZE.y) | pos = (5.5, 0.5) | grid_pos = (5, 0)
#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct AbsolutePosition {
    pub x: f32,
    pub y: f32,
}

impl From<AbsolutePosition> for Vec2 {
    fn from(p: AbsolutePosition) -> Vec2 {
        Vec2::new(p.x, p.y)
    }
}

/// absolute_pos = (5.5 * TILE_SIZE.X, 0.5 * TILE_SIZE.y) | pos = (5.5, 0.5) | grid_pos = (5, 0)
#[derive(Default, Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub struct GridPosition {
    pub x: i32,
    pub y: i32,
}

impl GridPosition {
    pub fn to_chunk_pos(self) -> ChunkPosition {
        ChunkPosition {
            x: self.x * CHUNK_SIZE.x as i32,
            y: self.y * CHUNK_SIZE.y as i32,
        }
    }
}

/// chunk_pos : (1,1) is 1 right and 1 down
/// ChunkPos {x: 2, y: 2} <=> GridPosition {x: 2*CHUNK_SIZE, y: 2*CHUNK_SIZE}
#[derive(Default, Debug, Hash, Clone, Copy, PartialEq, Eq)]
pub struct ChunkPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Component, Default, Debug)]
pub struct StructureManager {
    pub structures: HashMap<GridPosition, Entity>, // local GridPosition -> structure
}

/// Données spécifiques à chaque map
#[derive(Resource, Default)]
pub struct MapManager {
    pub chunks: HashMap<ChunkPosition, Entity>,
}

#[derive(Component)]
#[require(Position)]
pub struct Structure;

#[derive(Component)]
pub struct Wall;

pub fn spawn_one_chunk(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut map_manager: ResMut<MapManager>,
) -> () {
    let mut rng = rand::rng();
    let chunk_pos = ChunkPosition { x: 0, y: 0 };
    let mut structure_manager = StructureManager::default();
    for x in 0..CHUNK_SIZE.x {
        for y in 0..CHUNK_SIZE.y {
            let local_grid_pos = GridPosition {
                x: x as i32,
                y: y as i32,
            };

            let is_wall = rng.random_bool(0.2);
            if is_wall
            // && (chunk_pos.x > 0 || chunk_pos.x < 0)
            // && (chunk_pos.y > 0 || chunk_pos.y < 0)
            && (local_grid_pos.x > 2 )
            && (local_grid_pos.y > 2 )
            {
                let grid_pos = local_grid_pos_to_grid_pos(local_grid_pos, chunk_pos);
                let pos = grid_pos_to_pos(grid_pos);

                let target_pos = pos_to_absolute_pos(pos);
                let mut transform = Transform::default();
                transform.translation.x = target_pos.x;
                transform.translation.y = target_pos.y;
                let wall_entity = commands
                    .spawn((
                        Structure,
                        Wall,
                        Sprite::from_image(asset_server.load("structures/wall.png")),
                        pos,
                        transform,
                    ))
                    .id();
                structure_manager
                    .structures
                    .insert(local_grid_pos, wall_entity);
            }
        }
    }

    let tile_display_size = UVec2::splat(TILE_SIZE.x as u32);
    let chunk_center_x = (chunk_pos.x as f32 * CHUNK_SIZE.x as f32 + CHUNK_SIZE.x as f32 / 2.0)
        * tile_display_size.x as f32;
    let chunk_center_y = -(chunk_pos.y as f32 * CHUNK_SIZE.y as f32 + CHUNK_SIZE.y as f32 / 2.0)
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
    map_manager.chunks.insert(chunk_pos, chunk_entity);
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
/// absolute_pos = (5.5 * TILE_SIZE.X, 0.5 * TILE_SIZE.y) | pos = (5.5, 0.5) | grid_pos = (5, 0)
/// chunk_pos : (1,1) is 1 right and 1 down

pub fn local_grid_pos_to_grid_pos(
    local_grid_pos: GridPosition,
    chunk_pos: ChunkPosition,
) -> GridPosition {
    GridPosition {
        x: chunk_pos.x * (CHUNK_SIZE.x as i32) + local_grid_pos.x,
        y: chunk_pos.y * (CHUNK_SIZE.y as i32) + local_grid_pos.y,
    }
}

// Conversion coordonnées logiques -> monde ; (5.5, 0.5) => (5.5 * TILE_SIZE.x, 0.5 * TILE_SIZE.y)
pub fn pos_to_absolute_pos(pos: Position) -> AbsolutePosition {
    AbsolutePosition {
        x: (pos.x + 0.5) * TILE_SIZE.x as f32,
        y: -((pos.y + 0.5) * TILE_SIZE.y as f32),
    }
}

// // adds 0.5 to coordinates to make entities spawn based on the corner of there sprite and not the center
// pub fn grid_pos_to_absolute_pos(grid_pos: GridPosition) -> AbsolutePosition {
//     AbsolutePosition {
//         x: grid_pos.x as f32 * TILE_SIZE.x + TILE_SIZE.x * 0.5,
//         y: -(grid_pos.y as f32 * TILE_SIZE.y + TILE_SIZE.y * 0.5),
//         // x: grid_pos.x as f32 * TILE_SIZE.x,
//         // y: -(grid_pos.y as f32 * TILE_SIZE.y),
//     }
// }

pub fn grid_pos_to_pos(grid_pos: GridPosition) -> Position {
    Position {
        x: grid_pos.x as f32,
        y: grid_pos.y as f32,
    }
}

// (5.5, 0.5) => (5, 0)
pub fn pos_to_grid_pos(pos: Position) -> GridPosition {
    GridPosition {
        x: pos.x.floor() as i32,
        y: pos.y.floor() as i32,
    }
}

// Conversion monde -> coordonnées logiques
pub fn absolute_pos_to_pos(absolute_pos: AbsolutePosition) -> Position {
    Position {
        x: absolute_pos.x as f32 / TILE_SIZE.x,
        y: (-absolute_pos.y as f32) / TILE_SIZE.y,
    }
}

// Conversion monde -> coordonnées logiques
pub fn absolute_pos_to_grid_pos(absolute_pos: AbsolutePosition) -> GridPosition {
    GridPosition {
        x: (absolute_pos.x as f32 / TILE_SIZE.x).floor() as i32,
        y: ((-absolute_pos.y as f32) / TILE_SIZE.y).floor() as i32,
    }
}

/// Convertit une position monde (pixels) en position de chunk.
pub fn absolute_pos_to_chunk_pos(absolute_pos: AbsolutePosition) -> ChunkPosition {
    ChunkPosition {
        x: (absolute_pos.x as f32 / (CHUNK_SIZE.x as f32 * TILE_SIZE.x)).floor() as i32,
        y: ((-absolute_pos.y as f32) / (CHUNK_SIZE.y as f32 * TILE_SIZE.y)).floor() as i32,
    }
}

pub fn chunk_pos_to_grid_pos(chunk_pos: ChunkPosition) -> GridPosition {
    GridPosition {
        x: chunk_pos.x * CHUNK_SIZE.x as i32,
        y: chunk_pos.y * CHUNK_SIZE.y as i32,
    }
}

pub fn grid_pos_to_chunk_pos(grid_pos: GridPosition) -> ChunkPosition {
    ChunkPosition {
        x: grid_pos.x / CHUNK_SIZE.x as i32,
        y: grid_pos.y / CHUNK_SIZE.y as i32,
    }
}

pub fn pos_to_chunk_pos(pos: Position) -> ChunkPosition {
    ChunkPosition {
        x: (pos.x / CHUNK_SIZE.x as f32).floor() as i32,
        y: (pos.y / CHUNK_SIZE.y as f32).floor() as i32,
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
//         let camera_chunk_pos = world_pos_to_rounded_chunk(transform.translation.xy());
//         let active_map_id = camera_map.map_id;

//         // Récupérer les données de la map de la caméra
//         if let Some(map_data) = multi_map_manager.maps.get_mut(&active_map_id) {
//             for y in (camera_chunk_pos.y - SIZE)..(camera_chunk_pos.y + SIZE) {
//                 for x in (camera_chunk_pos.x - SIZE)..(camera_chunk_pos.x + SIZE) {
//                     let chunk_pos = ChunkPos { x, y };
//                     if !map_data
//                         .chunk_manager
//                         .spawned_chunks
//                         .contains_key(&chunk_pos)
//                     {
//                         let entity = spawn_chunk(
//                             &mut commands,
//                             &asset_server,
//                             &mut map_data.structure_manager,
//                             chunk_pos,
//                             active_map_id,
//                         );
//                         map_data
//                             .chunk_manager
//                             .spawned_chunks
//                             .insert(chunk_pos, entity);
//                     }
//                 }
//             }
//         }
//     }
// }

// fn spawn_chunks_around_units_system(
//     mut commands: Commands,
//     asset_server: Res<AssetServer>,
//     unit_query: Query<(&Position), With<Unit>>,
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
//     for (unit_grid_pos, current_map) in unit_query.iter() {
//         if current_map.map_id != active_map_id {
//             continue; // Ignore les unités sur d'autres maps
//         }

//         let unit_chunk_pos = rounded_tile_pos_to_rounded_chunk(*unit_grid_pos);

//         if let Some(map_data) = multi_map_manager.maps.get_mut(&current_map.map_id) {
//             for y in (unit_chunk_pos.y - SIZE)..(unit_chunk_pos.y + SIZE) {
//                 for x in (unit_chunk_pos.x - SIZE)..(unit_chunk_pos.x + SIZE) {
//                     let chunk_pos = ChunkPos { x, y };
//                     if !map_data
//                         .chunk_manager
//                         .spawned_chunks
//                         .contains_key(&chunk_pos)
//                     {
//                         let entity = spawn_chunk(
//                             &mut commands,
//                             &asset_server,
//                             &mut map_data.structure_manager,
//                             chunk_pos,
//                             current_map.map_id,
//                         );
//                         map_data
//                             .chunk_manager
//                             .spawned_chunks
//                             .insert(chunk_pos, entity);
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

    fn approx_pos(a: AbsolutePosition, b: AbsolutePosition) -> bool {
        (a.x - b.x).abs() < EPS && (a.y - b.y).abs() < EPS
    }

    fn approx_tile(a: Position, b: Position) -> bool {
        (a.x - b.x).abs() < EPS && (a.y - b.y).abs() < EPS
    }

    #[test]
    fn local_grid_pos_to_grid_pos_test() {
        let chunk_pos = ChunkPosition { x: 0, y: 0 };

        let grid_pos = local_grid_pos_to_grid_pos(GridPosition { x: 0, y: 0 }, chunk_pos);
        assert_eq!(grid_pos, GridPosition { x: 0, y: 0 });
        let grid_pos = local_grid_pos_to_grid_pos(GridPosition { x: 2, y: 2 }, chunk_pos);
        assert_eq!(grid_pos, GridPosition { x: 2, y: 2 });

        let chunk_pos = ChunkPosition { x: 2, y: 2 };

        let grid_pos = local_grid_pos_to_grid_pos(GridPosition { x: 0, y: 0 }, chunk_pos);
        assert_eq!(
            grid_pos,
            GridPosition {
                x: 2 * CHUNK_SIZE.x as i32,
                y: 2 * CHUNK_SIZE.y as i32
            }
        );
        let grid_pos = local_grid_pos_to_grid_pos(GridPosition { x: 2, y: 2 }, chunk_pos);
        assert_eq!(
            grid_pos,
            GridPosition {
                x: 2 * CHUNK_SIZE.x as i32 + 2,
                y: 2 * CHUNK_SIZE.y as i32 + 2
            }
        );
    }

    // #[test]
    // fn grid_pos_to_absolute_pos_test() {
    //     // TODO: does not work because now it's right/down instead of right/up for coordinates
    //     let absolute_pos = grid_pos_to_absolute_pos(GridPosition { x: 0, y: 0 });
    //     assert!(approx_pos(
    //         absolute_pos,
    //         AbsolutePosition { x: 0.0, y: 0.0 }
    //     ));
    //     let absolute_pos = grid_pos_to_absolute_pos(GridPosition { x: 2, y: 2 });
    //     assert!(approx_pos(
    //         absolute_pos,
    //         AbsolutePosition {
    //             x: 2.0 * TILE_SIZE.x,
    //             y: 2.0 * TILE_SIZE.y
    //         }
    //     ));
    // }
}
