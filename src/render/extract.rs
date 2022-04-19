use bevy::{
    math::Vec4,
    prelude::{Added, Bundle, Changed, Commands, Component, Entity, Or, Query, Transform},
    utils::HashMap,
};

use crate::{
    map::{
        Tilemap2dSize, Tilemap2dSpacing, Tilemap2dTextureSize, Tilemap2dTileSize, TilemapId,
        TilemapMeshType, TilemapTexture,
    },
    tiles::{TilePos2d, TileTexture},
};

use super::chunk::PackedTileData;

#[derive(Component)]
pub struct ExtractedTile {
    pub position: TilePos2d,
    pub tile: PackedTileData,
    pub tilemap_id: TilemapId,
}

#[derive(Bundle)]
pub struct ExtractedTileBundle {
    tile: ExtractedTile,
}

#[derive(Bundle)]
pub struct ExtractedTilemapBundle {
    transform: Transform,
    size: Tilemap2dTileSize,
    texture_size: Tilemap2dTextureSize,
    spacing: Tilemap2dSpacing,
    mesh_type: TilemapMeshType,
    texture: TilemapTexture,
    map_size: Tilemap2dSize,
}

pub fn extract(
    mut commands: Commands,
    changed_tiles_query: Query<
        (Entity, &TilePos2d, &TilemapId, &TileTexture),
        Or<(
            Added<TilePos2d>,
            Added<TileTexture>,
            Changed<TilePos2d>,
            Changed<TileTexture>,
        )>,
    >,
    tilemap_query: Query<(
        Entity,
        &Transform,
        &Tilemap2dTileSize,
        &Tilemap2dTextureSize,
        &Tilemap2dSpacing,
        &TilemapMeshType,
        &TilemapTexture,
        &Tilemap2dSize,
    )>,
) {
    let mut extracted_tiles = Vec::new();
    let mut extracted_tilemaps = HashMap::default();
    // Process all tiles
    for (entity, tile_pos, tilemap_id, tile_texture) in changed_tiles_query.iter() {
        let tile = PackedTileData {
            position: Vec4::new(tile_pos.x as f32, tile_pos.y as f32, 0.0, 0.0),
            texture: Vec4::new(tile_texture.0 as f32, 0.0, 0.0, 0.0),
            color: Vec4::new(1.0, 1.0, 1.0, 1.0),
        };

        let data = tilemap_query.get(tilemap_id.0).unwrap();
        extracted_tilemaps.insert(
            data.0,
            (
                data.0,
                ExtractedTilemapBundle {
                    transform: *data.1,
                    size: *data.2,
                    texture_size: *data.3,
                    spacing: *data.4,
                    mesh_type: *data.5,
                    texture: data.6.clone(),
                    map_size: *data.7,
                },
            ),
        );

        extracted_tiles.push((
            entity,
            ExtractedTileBundle {
                tile: ExtractedTile {
                    position: *tile_pos,
                    tile,
                    tilemap_id: *tilemap_id,
                },
            },
        ));
    }

    let extracted_tilemaps: Vec<(Entity, ExtractedTilemapBundle)> =
        extracted_tilemaps.drain().map(|kv| kv.1).collect();

    commands.insert_or_spawn_batch(extracted_tiles);
    commands.insert_or_spawn_batch(extracted_tilemaps);
}
