use bevy::prelude::Res;
use bevy::prelude::Time;
use bevy::{math::Vec4, prelude::*, utils::HashMap, render::Extract};

use crate::render::SecondsSinceStartup;
use crate::tiles::AnimatedTile;
use crate::{
    map::{
        Tilemap2dSize, Tilemap2dSpacing, Tilemap2dTextureSize, Tilemap2dTileSize, TilemapId,
        TilemapMeshType, TilemapTexture,
    },
    tiles::{TileColor, TileFlip, TilePos2d, TileTexture, TileVisible},
};

use super::{chunk::PackedTileData, RemovedTileEntity};

#[cfg(not(feature = "atlas"))]
use bevy::render::render_resource::TextureUsages;

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

#[derive(Component)]
pub struct ExtractedTilemapTexture {
    pub tile_size: Tilemap2dTileSize,
    pub texture_size: Tilemap2dTextureSize,
    pub spacing: Tilemap2dSpacing,
    pub texture: TilemapTexture,
}

#[derive(Bundle)]
pub struct ExtractedTilemapTextureBundle {
    data: ExtractedTilemapTexture,
}

pub fn extract(
    mut commands: Commands,
    changed_tiles_query: Extract<Query<
        (
            Entity,
            &TilePos2d,
            &TilemapId,
            &TileTexture,
            &TileVisible,
            &TileFlip,
            &TileColor,
            Option<&AnimatedTile>,
        ),
        Or<(
            Added<TilePos2d>,
            Changed<TilePos2d>,
            Changed<TileVisible>,
            Changed<TileTexture>,
            Changed<TileFlip>,
            Changed<TileColor>,
        )>,
    >>,
    tilemap_query: Extract<Query<(
        Entity,
        &GlobalTransform,
        &Tilemap2dTileSize,
        &Tilemap2dTextureSize,
        &Tilemap2dSpacing,
        &TilemapMeshType,
        &TilemapTexture,
        &Tilemap2dSize,
    )>>,
    changed_tilemap_query: Extract<Query<
        Entity,
        Or<(
            Added<TilemapMeshType>,
            Changed<TilemapMeshType>,
            Changed<GlobalTransform>,
        )>,
    >>,
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
        if let Ok(data) = tilemap_query.get(tilemap_entity) {
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
    }

    let extracted_tilemaps: Vec<(Entity, ExtractedTilemapBundle)> =
        extracted_tilemaps.drain().map(|kv| kv.1).collect();

    // Extracts tilemap textures.
    for (entity, _, tile_size, texture_size, spacing, _, texture, _) in tilemap_query.iter() {
        if let Some(_atlas_image) = images.get(&texture.0) {
            #[cfg(not(feature = "atlas"))]
            if !_atlas_image
                .texture_descriptor
                .usage
                .contains(TextureUsages::COPY_SRC)
            {
                log::warn!("Texture atlas MUST have COPY_SRC texture usages defined! You may ignore this warning if the atlas already has the COPY_SRC usage flag. Please see: https://github.com/StarArawn/bevy_ecs_tilemap/blob/main/examples/helpers/texture.rs");
                continue;
            }
        } else {
            continue;
        }

        extracted_tilemap_textures.push((
            entity,
            ExtractedTilemapTextureBundle {
                data: ExtractedTilemapTexture {
                    tile_size: *tile_size,
                    texture_size: *texture_size,
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
