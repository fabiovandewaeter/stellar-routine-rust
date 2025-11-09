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
#[derive(Component, Default, Debug, Hash, Clone, Copy, PartialEq, Eq)]
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

/// ChunkPos {x: 2, y: 2} <=> GridPosition {x: 2*CHUNK_SIZE, y: 2*CHUNK_SIZE}
#[derive(Component, Default, Debug, Hash, Clone, Copy, PartialEq, Eq)]
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
pub struct Structure;

#[derive(Component)]
pub struct Wall;

#[derive(Component)]
pub struct Chest;

pub fn spawn_one_chunk(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut map_manager: ResMut<MapManager>,
) -> () {
    let mut rng = rand::rng();

    let tile_display_size = UVec2::splat(TILE_SIZE.x as u32);
    let tile_data: Vec<Option<TileData>> = (0..CHUNK_SIZE.element_product())
        .map(|_| rng.random_range(0..5))
        .map(|i| {
            if i == 0 {
                None
            } else {
                Some(TileData::from_tileset_index(i - 1))
            }
        })
        .collect();

    let mut structure_manager = StructureManager::default();

    let chunk_pos = ChunkPosition { x: 0, y: 0 };
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
            {
                let grid_pos = local_tile_pos_to_rounded_tile(local_grid_pos, chunk_pos);
                let tilemap_world_pos = rounded_tile_pos_to_world(grid_pos);
                let transform = Transform::from_translation(Vec3::new(
                    tilemap_world_pos.x,
                    tilemap_world_pos.y,
                    // TILE_LAYER_LEVEL,
                    0.0,
                ));
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
                    .insert(local_grid_pos, wall_entity);
            }
        }
    }
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

// pub fn spawn_chunk(
//     commands: &mut Commands,
//     asset_server: &AssetServer,
//     mut structure_manager: &mut StructureManager,
//     chunk_pos: ChunkPos,
//     map_id: MapId,
// ) -> Entity {
//     let tilemap_entity = commands.spawn_empty().id();
//     let mut tile_storage = TileStorage::empty(CHUNK_SIZE.into());
//     let mut rng = rand::rng();

//     // Collecte les positions des structures à créer
//     let mut structures_to_spawn = Vec::new();

//     // Spawn the elements of the tilemap.
//     for x in 0..CHUNK_SIZE.x {
//         for y in 0..CHUNK_SIZE.y {
//             let local_tile_pos = TilePos { x, y };
//             let tile_entity = commands
//                 .spawn(TileBundle {
//                     position: local_tile_pos,
//                     tilemap_id: TilemapId(tilemap_entity),
//                     texture_index: TileTextureIndex(0),
//                     ..Default::default()
//                 })
//                 .id();

//             let is_wall = rng.random_bool(0.2);
//             if is_wall
//                 && (chunk_pos.x > 0 || chunk_pos.x < 0)
//                 && (chunk_pos.y > 0 || chunk_pos.y < 0)
//             {
//                 let local_tile_pos = GridPosition {
//                     x: local_tile_pos.x as i32,
//                     y: local_tile_pos.y as i32,
//                 };
//                 let rounded_tile_pos = local_tile_pos_to_rounded_tile(local_tile_pos, chunk_pos);
//                 structures_to_spawn.push(rounded_tile_pos);
//             }

//             match commands.get_entity(tilemap_entity) {
//                 Ok(mut entity_command) => entity_command.add_child(tile_entity),
//                 Err(_) => todo!(),
//             };

//             tile_storage.set(&local_tile_pos, tile_entity);
//         }
//     }

//     // Calcule la position du tilemap dans le monde
//     // let rounded_tile_pos = rounded_chunk_pos_to_rounded_tile(&chunk_pos);
//     let rounded_tile_pos = GridPosition::from(chunk_pos);
//     let tilemap_world_pos = rounded_tile_pos_to_world(rounded_tile_pos);
//     let tilemap_transform = Transform::from_translation(Vec3::new(
//         tilemap_world_pos.x,
//         tilemap_world_pos.y,
//         TILE_LAYER_LEVEL,
//     ));

//     let image_handles = vec![
//         asset_server.load("tiles/grass.png"),
//         asset_server.load("tiles/stone.png"),
//     ];

//     // Configure le tilemap
//     match commands.get_entity(tilemap_entity) {
//         Ok(mut entity_commands) => entity_commands.insert(TilemapBundle {
//             grid_size: TILE_SIZE.into(),
//             size: CHUNK_SIZE.into(),
//             storage: tile_storage,
//             texture: TilemapTexture::Vector(image_handles),
//             tile_size: TILE_SIZE,
//             transform: tilemap_transform,
//             render_settings: TilemapRenderSettings {
//                 render_chunk_size: RENDER_CHUNK_SIZE,
//                 ..Default::default()
//             },
//             ..Default::default()
//         }),
//         Err(_) => todo!(),
//     };

//     // Spawn les structures APRÈS avoir configuré le tilemap
//     // et les attache directement au tilemap
//     for rounded_tile_pos in structures_to_spawn {
//         let wall_entity = commands
//             .spawn((
//                 Structure,
//                 Wall,
//                 Sprite::from_image(asset_server.load("structures/wall.png")),
//                 CurrentMap { map_id },
//             ))
//             .id();

//         spawn_structure_in_chunk(
//             commands,
//             &wall_entity,
//             &mut structure_manager,
//             tilemap_entity,
//             rounded_tile_pos,
//             tilemap_world_pos,
//         );
//     }

//     // Ajoutez aussi CurrentMap au tilemap lui-même
//     commands
//         .entity(tilemap_entity)
//         .insert(CurrentMap { map_id });

//     tilemap_entity
// }

// fn spawn_structure_in_chunk(
//     commands: &mut Commands,
//     structure_entity: &Entity,
//     structure_manager: &mut StructureManager,
//     tilemap_entity: Entity,
//     rounded_tile_pos: GridPosition,
//     tilemap_world_pos: Vec2,
// ) {
//     // Calcule la position absolue de la structure
//     let structure_world_pos = rounded_tile_pos_to_world(rounded_tile_pos);

//     // Calcule la position RELATIVE au tilemap
//     let relative_pos = structure_world_pos - tilemap_world_pos;

//     let transform = Transform::from_translation(Vec3::new(
//         relative_pos.x,
//         relative_pos.y,
//         STRUCTURE_LAYER_LEVEL - TILE_LAYER_LEVEL, // Z relatif
//     ));

//     // TODO: use GridPosition instead of Transform there
//     let global_grid_pos = GridPosition {
//         x: rounded_tile_pos.x,
//         y: rounded_tile_pos.y,
//     };

//     match commands.get_entity(*structure_entity) {
//         Ok(mut entity_command) => {
//             entity_command.insert(transform);
//             entity_command.insert(global_grid_pos);
//         }
//         Err(_) => todo!(),
//     };

//     // Attache la structure au tilemap, pas à une tile individuelle
//     match commands.get_entity(tilemap_entity) {
//         Ok(mut entity_command) => entity_command.add_child(*structure_entity),
//         Err(_) => todo!(),
//     };

//     // Enregistre la structure dans le manager
//     structure_manager
//         .structures
//         .insert(rounded_tile_pos, *structure_entity);
// }

// // add transform to structure_entity and add it to structure_manager
// pub fn place_structure(
//     commands: &mut Commands,
//     asset_server: &Res<AssetServer>, // Ajouté pour pouvoir spawner le chunk
//     structure_entity: &Entity,
//     structure_manager: &mut StructureManager,
//     chunk_manager: &mut ChunkManager, // Maintenant mutable
//     rounded_tile_pos: GridPosition,
//     map_id: MapId,
// ) {
//     let rounded_chunk_pos = rounded_tile_pos_to_rounded_chunk(rounded_tile_pos);

//     // Charger le chunk s'il n'existe pas
//     if !chunk_manager
//         .spawned_chunks
//         .contains_key(&rounded_chunk_pos)
//     {
//         let entity = spawn_chunk(
//             commands,
//             asset_server,
//             structure_manager,
//             rounded_chunk_pos,
//             map_id,
//         );
//         chunk_manager
//             .spawned_chunks
//             .insert(rounded_chunk_pos, entity);
//     }

//     // Maintenant le chunk existe forcément
//     if let Some(&tilemap_entity) = chunk_manager.spawned_chunks.get(&rounded_chunk_pos) {
//         let tilemap_world_pos =
//             rounded_tile_pos_to_world(rounded_chunk_pos_to_rounded_tile(rounded_chunk_pos));

//         spawn_structure_in_chunk(
//             commands,
//             structure_entity,
//             structure_manager,
//             tilemap_entity,
//             rounded_tile_pos,
//             tilemap_world_pos,
//         );
//     } else {
//         panic!();
//     }
// }

// pub fn is_tile_passable(
//     rounded_tile_pos: GridPosition,
//     multi_map_manager: &Res<MultiMapManager>,
// ) -> bool {
//     if let Some(map_data) = multi_map_manager.get_map(map_id) {
//         if let Some(_structure_entity) =
//             map_data.structure_manager.structures.get(&rounded_tile_pos)
//         {
//             return false;
//         }
//     }
//     // Si le chunk n'existe pas, on suppose qu'il n'y a pas de mur.
//     // TODO: change that or spawn the chunk
//     true
// }

// ========= coordinates conversion =========
/// absolute_pos = (5.5 * TILE_SIZE.X, 0.5 * TILE_SIZE.y) | pos = (5.5, 0.5) | grid_pos = (5, 0)

// TODO: rename tile_pos to pos and rounded_tile to grid_pos and world_pos to absolute_pos
// TODO: rename tile_pos to pos and rounded_tile to grid_pos and world_pos to absolute_pos
// TODO: rename tile_pos to pos and rounded_tile to grid_pos and world_pos to absolute_pos
// TODO: rename tile_pos to pos and rounded_tile to grid_pos and world_pos to absolute_pos
// TODO: rename tile_pos to pos and rounded_tile to grid_pos and world_pos to absolute_pos
// TODO: rename tile_pos to pos and rounded_tile to grid_pos and world_pos to absolute_pos
// TODO: rename tile_pos to pos and rounded_tile to grid_pos and world_pos to absolute_pos
// TODO: rename tile_pos to pos and rounded_tile to grid_pos and world_pos to absolute_pos
// TODO: rename tile_pos to pos and rounded_tile to grid_pos and world_pos to absolute_pos
// TODO: rename tile_pos to pos and rounded_tile to grid_pos and world_pos to absolute_pos
// TODO: rename tile_pos to pos and rounded_tile to grid_pos and world_pos to absolute_pos
pub fn local_tile_pos_to_rounded_tile(
    local_tile_pos: GridPosition,
    rounded_chunk_pos: ChunkPosition,
) -> GridPosition {
    GridPosition {
        x: rounded_chunk_pos.x * CHUNK_SIZE.x as i32 + local_tile_pos.x,
        y: rounded_chunk_pos.y * CHUNK_SIZE.y as i32 + local_tile_pos.y,
    }
}

// Conversion coordonnées logiques -> monde ; (5.5, 0.5) => (5.5 * TILE_SIZE.x, 0.5 * TILE_SIZE.y)
pub fn tile_pos_to_world(tile_pos: Position) -> AbsolutePosition {
    AbsolutePosition {
        x: tile_pos.x * TILE_SIZE.x as f32,
        y: tile_pos.y * TILE_SIZE.y as f32,
    }
}

// adds 0.5 to coordinates to make entities spawn based on the corner of there sprite and not the center
pub fn rounded_tile_pos_to_world(rounded_tile_pos: GridPosition) -> AbsolutePosition {
    AbsolutePosition {
        x: rounded_tile_pos.x as f32 * TILE_SIZE.x + TILE_SIZE.x * 0.5,
        y: rounded_tile_pos.y as f32 * TILE_SIZE.y + TILE_SIZE.y * 0.5,
    }
}

// (5.5, 0.5) => (5, 0)
pub fn tile_pos_to_rounded_tile(pos: Position) -> GridPosition {
    GridPosition {
        x: pos.x.floor() as i32,
        y: pos.y.floor() as i32,
    }
}

// Conversion monde -> coordonnées logiques
pub fn world_pos_to_tile(world_pos: AbsolutePosition) -> Position {
    Position {
        x: world_pos.x as f32 / TILE_SIZE.x,
        y: world_pos.y as f32 / TILE_SIZE.y,
    }
}

// Conversion monde -> coordonnées logiques
pub fn world_pos_to_rounded_tile(world_pos: AbsolutePosition) -> GridPosition {
    GridPosition {
        x: (world_pos.x as f32 / TILE_SIZE.x).floor() as i32,
        y: (world_pos.y as f32 / TILE_SIZE.y).floor() as i32,
    }
}

/// Convertit une position monde (pixels) en position de chunk.
pub fn world_pos_to_rounded_chunk(world_pos: AbsolutePosition) -> ChunkPosition {
    ChunkPosition {
        x: (world_pos.x as f32 / (CHUNK_SIZE.x as f32 * TILE_SIZE.x)).floor() as i32,
        y: (world_pos.y as f32 / (CHUNK_SIZE.y as f32 * TILE_SIZE.y)).floor() as i32,
    }
}

pub fn rounded_chunk_pos_to_rounded_tile(rounded_chunk_pos: ChunkPosition) -> GridPosition {
    GridPosition {
        x: rounded_chunk_pos.x * CHUNK_SIZE.x as i32,
        y: rounded_chunk_pos.y * CHUNK_SIZE.y as i32,
    }
}

pub fn rounded_tile_pos_to_rounded_chunk(rounded_tile_pos: GridPosition) -> ChunkPosition {
    ChunkPosition {
        x: rounded_tile_pos.x / CHUNK_SIZE.x as i32,
        y: rounded_tile_pos.y / CHUNK_SIZE.y as i32,
    }
}

pub fn tile_pos_to_rounded_chunk(tile_pos: Position) -> ChunkPosition {
    ChunkPosition {
        x: (tile_pos.x / CHUNK_SIZE.x as f32).floor() as i32,
        y: (tile_pos.y / CHUNK_SIZE.y as f32).floor() as i32,
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
