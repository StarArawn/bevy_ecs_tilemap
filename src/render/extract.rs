use bevy::{
    math::Vec4,
    prelude::{
        Added, Bundle, Changed, Commands, Component, Entity, GlobalTransform, Or, Query, With,
    },
    utils::HashMap,
};

use crate::tiles::AnimatedTile;
use crate::{
    map::{
        Tilemap2dSize, Tilemap2dSpacing, Tilemap2dTextureSize, Tilemap2dTileSize, TilemapId,
        TilemapMeshType, TilemapTexture,
    },
    tiles::{TileFlip, TilePos2d, TileTexture, TileVisible},
};

use super::{chunk::PackedTileData, RemovedTileEntity};

#[derive(Component)]
pub struct ExtractedTile {
    pub entity: Entity,
    pub position: TilePos2d,
    pub tile: PackedTileData,
    pub tilemap_id: TilemapId,
}

#[derive(Bundle)]
pub struct ExtractedTileBundle {
    tile: ExtractedTile,
}

#[derive(Component)]
pub struct ExtractedRemovedTile {
    pub entity: Entity,
}

#[derive(Bundle)]
pub struct ExtractedRemovedTileBundle {
    tile: ExtractedRemovedTile,
}

#[derive(Bundle)]
pub struct ExtractedTilemapBundle {
    transform: GlobalTransform,
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
        (
            Entity,
            &TilePos2d,
            &TilemapId,
            &TileTexture,
            &TileVisible,
            &TileFlip,
            Option<&AnimatedTile>,
        ),
        Or<(
            Added<TilePos2d>,
            Changed<TilePos2d>,
            Changed<TileVisible>,
            Changed<TileTexture>,
            Changed<TileFlip>,
        )>,
    >,
    tilemap_query: Query<(
        Entity,
        &GlobalTransform,
        &Tilemap2dTileSize,
        &Tilemap2dTextureSize,
        &Tilemap2dSpacing,
        &TilemapMeshType,
        &TilemapTexture,
        &Tilemap2dSize,
    )>,
    changed_tilemap_query: Query<Entity, (With<TilemapMeshType>, Changed<GlobalTransform>)>,
) {
    let mut extracted_tiles = Vec::new();
    let mut extracted_tilemaps = HashMap::default();
    // Process all tiles
    for (entity, tile_pos, tilemap_id, tile_texture, visible, flip, animated) in
        changed_tiles_query.iter()
    {
        // flipping and rotation packed in bits
        // bit 0 : flip_x
        // bit 1 : flip_y
        // bit 2 : flip_d (anti diagonal)
        let tile_flip_bits = flip.x as i32 | (flip.y as i32) << 1 | (flip.d as i32) << 2;

        let tile = if let Some(animated) = animated {
            PackedTileData {
                visible: visible.0,
                position: Vec4::new(tile_pos.x as f32, tile_pos.y as f32, animated.speed, 0.0),
                texture: Vec4::new(
                    tile_texture.0 as f32,
                    tile_flip_bits as f32,
                    animated.start as f32,
                    animated.end as f32,
                ),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            }
        } else {
            PackedTileData {
                visible: visible.0,
                position: Vec4::new(tile_pos.x as f32, tile_pos.y as f32, 0.0, 0.0),
                texture: Vec4::new(tile_texture.0 as f32, tile_flip_bits as f32, 0.0, 0.0),
                color: Vec4::new(1.0, 1.0, 1.0, 1.0),
            }
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
                    entity,
                    position: *tile_pos,
                    tile,
                    tilemap_id: *tilemap_id,
                },
            },
        ));
    }

    for tilemap_entity in changed_tilemap_query.iter() {
        let data = tilemap_query.get(tilemap_entity).unwrap();
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
    }

    let extracted_tilemaps: Vec<(Entity, ExtractedTilemapBundle)> =
        extracted_tilemaps.drain().map(|kv| kv.1).collect();

    commands.insert_or_spawn_batch(extracted_tiles);
    commands.insert_or_spawn_batch(extracted_tilemaps);
}

pub fn extract_removal(
    mut commands: Commands,
    // removed_tiles_query: Query<(Entity, &, &TilemapId), With<RemoveTile>>,
    removed_tiles_query: Query<&RemovedTileEntity>,
) {
    let mut removed_tiles: Vec<(Entity, ExtractedRemovedTileBundle)> = Vec::new();
    for entity in removed_tiles_query.iter() {
        removed_tiles.push((
            entity.0,
            ExtractedRemovedTileBundle {
                tile: ExtractedRemovedTile { entity: entity.0 },
            },
        ));
    }

    commands.insert_or_spawn_batch(removed_tiles);
}
