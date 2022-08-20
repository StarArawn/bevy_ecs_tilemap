use bevy::prelude::Res;
use bevy::prelude::Time;
use bevy::{math::Vec4, prelude::*, render::Extract, utils::HashMap};

use crate::prelude::TilemapGridSize;
use crate::render::SecondsSinceStartup;
use crate::tiles::AnimatedTile;
use crate::{
    map::{
        TilemapId, TilemapSize, TilemapSpacing, TilemapTexture, TilemapTextureSize,
        TilemapTileSize, TilemapType,
    },
    tiles::{TileColor, TileFlip, TilePos, TileTexture, TileVisible},
};

use super::RemovedMapEntity;
use super::{chunk::PackedTileData, RemovedTileEntity};

#[cfg(not(feature = "atlas"))]
use bevy::render::render_resource::TextureUsages;

#[derive(Component)]
pub struct ExtractedTile {
    pub entity: Entity,
    pub position: TilePos,
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

#[derive(Component)]
pub struct ExtractedRemovedMap {
    pub entity: Entity,
}

#[derive(Bundle)]
pub struct ExtractedRemovedMapBundle {
    map: ExtractedRemovedMap,
}

#[derive(Bundle)]
pub struct ExtractedTilemapBundle {
    transform: GlobalTransform,
    size: TilemapTileSize,
    grid_size: TilemapGridSize,
    texture_size: TilemapTextureSize,
    spacing: TilemapSpacing,
    mesh_type: TilemapType,
    texture: TilemapTexture,
    map_size: TilemapSize,
    visibility: ComputedVisibility,
}

#[derive(Component)]
pub(crate) struct ExtractedTilemapTexture {
    pub tilemap_id: TilemapId,
    pub tile_size: TilemapTileSize,
    pub texture_size: TilemapTextureSize,
    pub spacing: TilemapSpacing,
    pub texture: TilemapTexture,
}

#[derive(Bundle)]
pub(crate) struct ExtractedTilemapTextureBundle {
    data: ExtractedTilemapTexture,
}

pub fn extract(
    mut commands: Commands,
    changed_tiles_query: Extract<
        Query<
            (
                Entity,
                &TilePos,
                &TilemapId,
                &TileTexture,
                &TileVisible,
                &TileFlip,
                &TileColor,
                Option<&AnimatedTile>,
            ),
            Or<(
                Added<TilePos>,
                Changed<TilePos>,
                Changed<TileVisible>,
                Changed<TileTexture>,
                Changed<TileFlip>,
                Changed<TileColor>,
            )>,
        >,
    >,
    tilemap_query: Extract<
        Query<(
            Entity,
            &GlobalTransform,
            &TilemapTileSize,
            &TilemapSpacing,
            &TilemapGridSize,
            &TilemapType,
            &TilemapTexture,
            &TilemapSize,
            &ComputedVisibility,
        )>,
    >,
    changed_tilemap_query: Extract<
        Query<
            Entity,
            Or<(
                Added<TilemapType>,
                Changed<TilemapType>,
                Changed<GlobalTransform>,
                Changed<TilemapTexture>,
                Changed<TilemapTileSize>,
                Changed<TilemapSpacing>,
                Changed<TilemapGridSize>,
                Changed<TilemapType>,
                Changed<TilemapSize>,
                Changed<ComputedVisibility>,
            )>,
        >,
    >,
    images: Extract<Res<Assets<Image>>>,
    time: Extract<Res<Time>>,
) {
    let mut extracted_tiles = Vec::new();
    let mut extracted_tilemaps = HashMap::default();
    let mut extracted_tilemap_textures = Vec::new();
    // Process all tiles
    for (entity, tile_pos, tilemap_id, tile_texture, visible, flip, color, animated) in
        changed_tiles_query.iter()
    {
        // flipping and rotation packed in bits
        // bit 0 : flip_x
        // bit 1 : flip_y
        // bit 2 : flip_d (anti diagonal)
        let tile_flip_bits = flip.x as i32 | (flip.y as i32) << 1 | (flip.d as i32) << 2;

        let mut position = Vec4::new(tile_pos.x as f32, tile_pos.y as f32, 0.0, 0.0);
        let mut texture = Vec4::new(tile_texture.0 as f32, tile_flip_bits as f32, 0.0, 0.0);
        if let Some(animation_data) = animated {
            position.z = animation_data.speed;
            texture.z = animation_data.start as f32;
            texture.w = animation_data.end as f32;
        } else {
            texture.z = tile_texture.0 as f32;
            texture.w = tile_texture.0 as f32;
        }

        let tile = PackedTileData {
            visible: visible.0,
            position,
            texture,
            color: color.0.into(),
        };

        let data = tilemap_query.get(tilemap_id.0).unwrap();

        extracted_tilemaps.insert(
            data.0,
            (
                data.0,
                ExtractedTilemapBundle {
                    transform: *data.1,
                    size: *data.2,
                    texture_size: TilemapTextureSize::default(),
                    spacing: *data.3,
                    grid_size: *data.4,
                    mesh_type: *data.5,
                    texture: data.6.clone(),
                    map_size: *data.7,
                    visibility: data.8.clone(),
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
        if let Ok(data) = tilemap_query.get(tilemap_entity) {
            extracted_tilemaps.insert(
                data.0,
                (
                    data.0,
                    ExtractedTilemapBundle {
                        transform: *data.1,
                        size: *data.2,
                        texture_size: TilemapTextureSize::default(),
                        spacing: *data.3,
                        grid_size: *data.4,
                        mesh_type: *data.5,
                        texture: data.6.clone(),
                        map_size: *data.7,
                        visibility: data.8.clone(),
                    },
                ),
            );
        }
    }

    let extracted_tilemaps: Vec<(Entity, ExtractedTilemapBundle)> =
        extracted_tilemaps.drain().map(|kv| kv.1).collect();

    // Extracts tilemap textures.
    for (entity, _, tile_size, spacing, _, _, texture, _, _) in tilemap_query.iter() {
        let texture_size = if let Some(_atlas_image) = images.get(&texture.0) {
            #[cfg(not(feature = "atlas"))]
            if !_atlas_image
                .texture_descriptor
                .usage
                .contains(TextureUsages::COPY_SRC)
            {
                continue;
            }

            _atlas_image.size().into()
        } else {
            continue;
        };

        extracted_tilemap_textures.push((
            entity,
            ExtractedTilemapTextureBundle {
                data: ExtractedTilemapTexture {
                    tilemap_id: TilemapId(entity),
                    tile_size: *tile_size,
                    texture_size: texture_size,
                    spacing: *spacing,
                    texture: texture.clone(),
                },
            },
        ));
    }

    commands.insert_or_spawn_batch(extracted_tiles);
    commands.insert_or_spawn_batch(extracted_tilemaps);
    commands.insert_or_spawn_batch(extracted_tilemap_textures);
    commands.insert_resource(SecondsSinceStartup(time.seconds_since_startup() as f32));
}

pub fn extract_removal(
    mut commands: Commands,
    removed_tiles_query: Extract<Query<&RemovedTileEntity>>,
    removed_maps_query: Extract<Query<&RemovedMapEntity>>,
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

    let mut removed_maps: Vec<(Entity, ExtractedRemovedMapBundle)> = Vec::new();
    for entity in removed_maps_query.iter() {
        removed_maps.push((
            entity.0,
            ExtractedRemovedMapBundle {
                map: ExtractedRemovedMap { entity: entity.0 },
            },
        ));
    }

    commands.insert_or_spawn_batch(removed_maps);
}
