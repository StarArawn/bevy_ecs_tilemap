use bevy::prelude::Res;
use bevy::prelude::Time;
use bevy::render::primitives::{Aabb, Frustum};
use bevy::render::render_resource::FilterMode;
use bevy::render::render_resource::TextureFormat;
use bevy::{math::Vec4, prelude::*, render::Extract, utils::HashMap};

use crate::prelude::TilemapGridSize;
use crate::prelude::TilemapPhysicalTileSize;
use crate::render::{DefaultSampler, SecondsSinceStartup};
use crate::tiles::AnimatedTile;
use crate::tiles::TilePosOld;
use crate::{
    map::{
        TilemapId, TilemapSize, TilemapSpacing, TilemapTexture, TilemapTextureSize,
        TilemapTileSize, TilemapType,
    },
    tiles::{TileColor, TileFlip, TilePos, TileTextureIndex, TileVisible},
    FrustumCulling,
};

use super::RemovedMapEntity;
use super::{chunk::PackedTileData, RemovedTileEntity};

#[derive(Component)]
pub struct ExtractedTile {
    pub entity: Entity,
    pub position: TilePos,
    pub old_position: TilePosOld,
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
    tile_size: TilemapTileSize,
    physical_tile_size: TilemapPhysicalTileSize,
    grid_size: TilemapGridSize,
    texture_size: TilemapTextureSize,
    spacing: TilemapSpacing,
    map_type: TilemapType,
    texture: TilemapTexture,
    map_size: TilemapSize,
    visibility: ComputedVisibility,
    frustum_culling: FrustumCulling,
}

#[derive(Component)]
pub(crate) struct ExtractedTilemapTexture {
    pub tilemap_id: TilemapId,
    pub tile_size: TilemapTileSize,
    pub texture_size: TilemapTextureSize,
    pub tile_spacing: TilemapSpacing,
    pub tile_count: u32,
    pub texture: TilemapTexture,
    pub filtering: FilterMode,
    pub format: TextureFormat,
}

impl ExtractedTilemapTexture {
    pub fn new(
        tilemap_entity: Entity,
        texture: TilemapTexture,
        tile_size: TilemapTileSize,
        tile_spacing: TilemapSpacing,
        filtering: FilterMode,
        image_assets: &Res<Assets<Image>>,
    ) -> ExtractedTilemapTexture {
        let (tile_count, texture_size, format) = match &texture {
            TilemapTexture::Single(handle) => {
                let image = image_assets.get(handle).expect(
                    "Expected image to have finished loading if \
                    it is being extracted as a texture!",
                );
                let texture_size: TilemapTextureSize = image.size().into();
                let tile_count_x = ((texture_size.x) / (tile_size.x + tile_spacing.x)).floor();
                let tile_count_y = ((texture_size.y) / (tile_size.y + tile_spacing.y)).floor();
                (
                    (tile_count_x * tile_count_y) as u32,
                    texture_size,
                    image.texture_descriptor.format,
                )
            }
            #[cfg(not(feature = "atlas"))]
            TilemapTexture::Vector(handles) => {
                for handle in handles {
                    let image = image_assets.get(handle).expect(
                        "Expected image to have finished loading if \
                        it is being extracted as a texture!",
                    );
                    let this_tile_size: TilemapTileSize = image.size().into();
                    if this_tile_size != tile_size {
                        panic!(
                            "Expected all provided image assets to have size {:?}, \
                                    but found image with size: {:?}",
                            tile_size, this_tile_size
                        );
                    }
                }
                let first_format = image_assets
                    .get(handles.first().unwrap())
                    .unwrap()
                    .texture_descriptor
                    .format;

                for handle in handles {
                    let image = image_assets.get(handle).unwrap();
                    if image.texture_descriptor.format != first_format {
                        panic!("Expected all provided image assets to have a format of: {:?} but found image with format: {:?}", first_format, image.texture_descriptor.format);
                    }
                }

                (handles.len() as u32, tile_size.into(), first_format)
            }
            #[cfg(not(feature = "atlas"))]
            TilemapTexture::TextureContainer(image_handle) => {
                let image = image_assets.get(image_handle).expect(
                    "Expected image to have finished loading if \
                        it is being extracted as a texture!",
                );
                let tile_size: TilemapTileSize = image.size().into();
                (
                    image.texture_descriptor.array_layer_count(),
                    tile_size.into(),
                    image.texture_descriptor.format,
                )
            }
        };

        ExtractedTilemapTexture {
            tilemap_id: TilemapId(tilemap_entity),
            texture,
            tile_size,
            tile_spacing,
            filtering,
            tile_count,
            texture_size,
            format,
        }
    }
}

#[derive(Bundle)]
pub(crate) struct ExtractedTilemapTextureBundle {
    data: ExtractedTilemapTexture,
}

#[derive(Component, Debug)]
pub struct ExtractedFrustum {
    frustum: Frustum,
}

impl ExtractedFrustum {
    pub fn intersects_obb(&self, aabb: &Aabb, transform_matrix: &Mat4) -> bool {
        self.frustum.intersects_obb(aabb, transform_matrix, false)
    }
}

#[allow(clippy::too_many_arguments)]
pub fn extract(
    mut commands: Commands,
    default_image_settings: Res<DefaultSampler>,
    changed_tiles_query: Extract<
        Query<
            (
                Entity,
                &TilePos,
                &TilePosOld,
                &TilemapId,
                &TileTextureIndex,
                &TileVisible,
                &TileFlip,
                &TileColor,
                Option<&AnimatedTile>,
            ),
            Or<(
                Changed<TilePos>,
                Changed<TileVisible>,
                Changed<TileTextureIndex>,
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
            &TilemapPhysicalTileSize,
            &TilemapSpacing,
            &TilemapGridSize,
            &TilemapType,
            &TilemapTexture,
            &TilemapSize,
            &ComputedVisibility,
            &FrustumCulling,
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
                Changed<TilemapPhysicalTileSize>,
                Changed<TilemapSpacing>,
                Changed<TilemapGridSize>,
                Changed<TilemapSize>,
                Changed<ComputedVisibility>,
                Changed<FrustumCulling>,
            )>,
        >,
    >,
    camera_query: Extract<Query<(Entity, &Frustum), With<Camera>>>,
    images: Extract<Res<Assets<Image>>>,
    time: Extract<Res<Time>>,
) {
    let mut extracted_tiles = Vec::new();
    let mut extracted_tilemaps = HashMap::default();
    let mut extracted_tilemap_textures = Vec::new();
    // Process all tiles
    for (
        entity,
        tile_pos,
        tile_pos_old,
        tilemap_id,
        tile_texture,
        visible,
        flip,
        color,
        animated,
    ) in changed_tiles_query.iter()
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
                    tile_size: *data.2,
                    physical_tile_size: *data.3,
                    texture_size: TilemapTextureSize::default(),
                    spacing: *data.4,
                    grid_size: *data.5,
                    map_type: *data.6,
                    texture: data.7.clone(),
                    map_size: *data.8,
                    visibility: data.9.clone(),
                    frustum_culling: *data.10,
                },
            ),
        );

        extracted_tiles.push((
            entity,
            ExtractedTileBundle {
                tile: ExtractedTile {
                    entity,
                    position: *tile_pos,
                    old_position: *tile_pos_old,
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
                        tile_size: *data.2,
                        physical_tile_size: *data.3,
                        texture_size: TilemapTextureSize::default(),
                        spacing: *data.4,
                        grid_size: *data.5,
                        map_type: *data.6,
                        texture: data.7.clone(),
                        map_size: *data.8,
                        visibility: data.9.clone(),
                        frustum_culling: *data.10,
                    },
                ),
            );
        }
    }

    let extracted_tilemaps: Vec<(Entity, ExtractedTilemapBundle)> =
        extracted_tilemaps.drain().map(|kv| kv.1).collect();

    // Extracts tilemap textures.
    for (entity, _, tile_size, _physical_tile_size, tile_spacing, _, _, texture, _, _, _) in
        tilemap_query.iter()
    {
        if texture.verify_ready(&images) {
            extracted_tilemap_textures.push((
                entity,
                ExtractedTilemapTextureBundle {
                    data: ExtractedTilemapTexture::new(
                        entity,
                        texture.clone(),
                        *tile_size,
                        *tile_spacing,
                        default_image_settings.0.min_filter,
                        &images,
                    ),
                },
            ))
        }
    }

    for (entity, frustum) in camera_query.iter() {
        commands
            .get_or_spawn(entity)
            .insert(ExtractedFrustum { frustum: *frustum });
    }

    commands.insert_or_spawn_batch(extracted_tiles);
    commands.insert_or_spawn_batch(extracted_tilemaps);
    commands.insert_or_spawn_batch(extracted_tilemap_textures);
    commands.insert_resource(SecondsSinceStartup(time.elapsed_seconds_f64() as f32));
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
