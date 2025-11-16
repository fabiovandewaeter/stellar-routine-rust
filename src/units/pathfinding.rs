use crate::{
    map::{MapManager, StructureManager, TileCoordinates, absolute_coord_to_tile_coord},
    units::Player,
};
use bevy::{prelude::*, sprite_render::TilemapChunk};
use pathfinding::prelude::dijkstra_all;
use std::collections::HashMap;

const FLOWFIELD_RADIUS: i32 = 50; // radius in tile

pub struct PathfindingPlugin;

#[derive(Resource, Default)]
// pub struct FlowField(pub HashMap<TileCoordinates, Vec2>);
pub struct FlowField(pub HashMap<TileCoordinates, TileCoordinates>);

#[derive(Message, Default)]
pub struct RecalculateFlowField;

impl Plugin for PathfindingPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(FlowField::default())
            .add_message::<RecalculateFlowField>()
            .add_systems(FixedUpdate, calculate_flow_field_system);
    }
}

pub fn calculate_flow_field_system(
    mut message_recalculate: MessageReader<RecalculateFlowField>,
    mut flow_field: ResMut<FlowField>,
    map_manager: Res<MapManager>,
    player_query: Query<&Transform, With<Player>>,
    chunk_query: Query<&StructureManager, With<TilemapChunk>>,
) {
    if message_recalculate.is_empty() {
        return;
    }
    message_recalculate.clear();

    let Ok(transform) = player_query.single() else {
        return;
    };
    let goal = absolute_coord_to_tile_coord((*transform).into());

    // TODO: regarder si on devrait utiliser dijkstra_partial ou dijkstra_reach
    let cost_map = dijkstra_all(&goal, |&tile| {
        let mut neighbors = Vec::with_capacity(8);
        for y in -1..=1 {
            for x in -1..=1 {
                if x == 0 && y == 0 {
                    continue;
                }

                let neighbor_tile = TileCoordinates {
                    x: tile.x + x,
                    y: tile.y + y,
                };

                // Vérifier que le voisin est dans le rayon ET praticable
                let dx = (neighbor_tile.x - goal.x).abs();
                let dy = (neighbor_tile.y - goal.y).abs();

                if dx > FLOWFIELD_RADIUS || dy > FLOWFIELD_RADIUS {
                    continue;
                }

                // 1. Vérifier si la tuile de destination est marchable
                if !map_manager.is_tile_walkable(neighbor_tile, &chunk_query) {
                    continue;
                }

                // 2. NOUVELLE VÉRIFICATION : Empêcher de couper les coins
                if x != 0 && y != 0 {
                    // C'est un mouvement diagonal
                    let adjacent_1 = TileCoordinates {
                        x: tile.x + x,
                        y: tile.y,
                    };
                    let adjacent_2 = TileCoordinates {
                        x: tile.x,
                        y: tile.y + y,
                    };

                    if !map_manager.is_tile_walkable(adjacent_1, &chunk_query)
                        || !map_manager.is_tile_walkable(adjacent_2, &chunk_query)
                    {
                        // L'un des coins est un mur, on ne peut pas passer
                        continue;
                    }
                }

                // Si on arrive ici, le mouvement est valide
                let cost = if x == 0 || y == 0 { 10 } else { 14 };
                neighbors.push((neighbor_tile, cost));
            }
        }

        neighbors
    });

    flow_field.0.clear();
    for y in (goal.y - FLOWFIELD_RADIUS)..=(goal.y + FLOWFIELD_RADIUS) {
        for x in (goal.x - FLOWFIELD_RADIUS)..=(goal.x + FLOWFIELD_RADIUS) {
            let tile = TileCoordinates { x, y };

            if tile == goal {
                continue;
            }

            if !map_manager.is_tile_walkable(tile, &chunk_query) {
                continue;
            }

            let mut best_neighbor = tile;
            let mut min_cost = cost_map.get(&tile).map_or(u32::MAX, |&(_, cost)| cost);

            for dy in -1..=1 {
                for dx in -1..=1 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }

                    let neighbor_tile = TileCoordinates {
                        x: tile.x + dx,
                        y: tile.y + dy,
                    };

                    if let Some((_, neighbor_cost)) = cost_map.get(&neighbor_tile) {
                        if *neighbor_cost < min_cost {
                            min_cost = *neighbor_cost;
                            best_neighbor = neighbor_tile;
                        }
                    }
                }
            }

            // si on a trouvé un chemin vers le player
            if best_neighbor != tile {
                // let direction = Vec2::new(
                //     (best_neighbor.x - tile.x) as f32,
                //     (best_neighbor.y - tile.y) as f32,
                // )
                // .normalize_or_zero();
                // flow_field.0.insert(tile, direction);
                flow_field.0.insert(tile, best_neighbor);
            }
        }
    }
}
