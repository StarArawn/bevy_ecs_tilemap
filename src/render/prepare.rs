use std::marker::PhantomData;

use crate::map::{
    TilemapId, TilemapSize, TilemapSpacing, TilemapTexture, TilemapTextureSize, TilemapTileSize,
    TilemapType,
};
use crate::render::extract::ExtractedFrustum;
use crate::{
    prelude::TilemapGridSize, render::RenderChunkSize, render::SecondsSinceStartup, FrustumCulling,
};
use bevy::log::trace;
use bevy::prelude::{InheritedVisibility, Resource};
use bevy::{
    math::{Mat4, UVec4},
    prelude::{Commands, Component, Entity, GlobalTransform, Query, Res, ResMut, Vec2},
    render::{
        render_resource::{DynamicUniformBuffer, ShaderType},
        renderer::{RenderDevice, RenderQueue},
    },
};

use super::{
    chunk::{ChunkId, PackedTileData, RenderChunk2dStorage, TilemapUniformData},
    extract::{ExtractedRemovedMap, ExtractedRemovedTile, ExtractedTile, ExtractedTilemapTexture},
    DynamicUniformIndex,
};

#[derive(Resource, Default)]
pub struct MeshUniformResource(pub DynamicUniformBuffer<MeshUniform>);

#[derive(Resource, Default)]
pub struct TilemapUniformResource(pub DynamicUniformBuffer<TilemapUniformData>);

#[derive(ShaderType, Component, Clone)]
pub struct MeshUniform {
    pub transform: Mat4,
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(crate) fn prepare(
    mut commands: Commands,
    mut chunk_storage: ResMut<RenderChunk2dStorage>,
    mut mesh_uniforms: ResMut<MeshUniformResource>,
    mut tilemap_uniforms: ResMut<TilemapUniformResource>,
    chunk_size: Res<RenderChunkSize>,
    extracted_tiles: Query<&ExtractedTile>,
    extracted_tilemaps: Query<(
        Entity,
        &GlobalTransform,
        &TilemapTileSize,
        &TilemapTextureSize,
        &TilemapSpacing,
        &TilemapGridSize,
        &TilemapType,
        &TilemapTexture,
        &TilemapSize,
        &InheritedVisibility,
        &FrustumCulling,
    )>,
    extracted_tilemap_textures: Query<&ExtractedTilemapTexture>,
    extracted_frustum_query: Query<&ExtractedFrustum>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    seconds_since_startup: Res<SecondsSinceStartup>,
) {
    for tile in extracted_tiles.iter() {
        // First if the tile position has changed remove the tile from the old location.
        if tile.position != tile.old_position.0 {
            chunk_storage.remove_tile_with_entity(tile.entity);
        }

        let chunk_index = chunk_size.map_tile_to_chunk(&tile.position);
        let (
            _entity,
            transform,
            tile_size,
            texture_size,
            spacing,
            grid_size,
            mesh_type,
            texture,
            map_size,
            visibility,
            frustum_culling,
        ) = extracted_tilemaps.get(tile.tilemap_id.0).unwrap();

        let chunk_data = UVec4::new(
            chunk_index.x,
            chunk_index.y,
            transform.translation().z as u32,
            tile.tilemap_id.0.index(),
        );

        let in_chunk_tile_index = chunk_size.map_tile_to_chunk_tile(&tile.position, &chunk_index);
        let chunk = chunk_storage.get_or_add(
            tile.entity,
            in_chunk_tile_index,
            tile.tilemap_id.0,
            &chunk_data,
            **chunk_size,
            *mesh_type,
            *tile_size,
            (*texture_size).into(),
            (*spacing).into(),
            *grid_size,
            texture.clone(),
            *map_size,
            *transform,
            visibility,
            frustum_culling,
        );
        chunk.set(
            &in_chunk_tile_index.into(),
            Some(PackedTileData {
                position: chunk_size
                    .map_tile_to_chunk_tile(&tile.position, &chunk_index)
                    .as_vec2()
                    .extend(tile.tile.position.z)
                    .extend(tile.tile.position.w),
                ..tile.tile
            }),
        );
    }

    // Copies transform changes from tilemap to chunks.
    for (
        entity,
        global_transform,
        tile_size,
        texture_size,
        spacing,
        grid_size,
        map_type,
        texture,
        map_size,
        visibility,
        frustum_culling,
    ) in extracted_tilemaps.iter()
    {
        let chunks = chunk_storage.get_chunk_storage(&UVec4::new(0, 0, 0, entity.index()));
        for chunk in chunks.values_mut() {
            chunk.texture = texture.clone();
            chunk.map_size = *map_size;
            chunk.texture_size = (*texture_size).into();
            chunk.spacing = (*spacing).into();
            chunk.visible = visibility.get();
            chunk.frustum_culling = **frustum_culling;
            chunk.update_geometry(
                (*global_transform).into(),
                *grid_size,
                *tile_size,
                *map_type,
            );
        }
    }

    for tilemap in extracted_tilemap_textures.iter() {
        let texture_size: Vec2 = tilemap.texture_size.into();
        let chunks =
            chunk_storage.get_chunk_storage(&UVec4::new(0, 0, 0, tilemap.tilemap_id.0.index()));
        for chunk in chunks.values_mut() {
            chunk.texture_size = texture_size;
        }
    }

    mesh_uniforms.0.clear();
    tilemap_uniforms.0.clear();

    for chunk in chunk_storage.iter_mut() {
        if !chunk.visible {
            trace!("Visibility culled chunk: {:?}", chunk.get_index());
            continue;
        }

        if chunk.frustum_culling
            && !extracted_frustum_query
                .iter()
                .any(|frustum| chunk.intersects_frustum(frustum))
        {
            trace!("Frustum culled chunk: {:?}", chunk.get_index());
            continue;
        }
        trace!("Preparing chunk: {:?}", chunk.get_index());

        chunk.prepare(&render_device);

        let mut chunk_uniform: TilemapUniformData = chunk.into();
        chunk_uniform.time = **seconds_since_startup;

        commands.spawn((
            chunk.texture.clone_weak(),
            chunk.get_transform(),
            ChunkId(chunk.get_index()),
            chunk.get_map_type(),
            TilemapId(Entity::from_bits(chunk.tilemap_id)),
            DynamicUniformIndex::<MeshUniform> {
                index: mesh_uniforms.0.push(MeshUniform {
                    transform: chunk.get_transform_matrix(),
                }),
                marker: PhantomData,
            },
            DynamicUniformIndex::<TilemapUniformData> {
                index: tilemap_uniforms.0.push(chunk_uniform),
                marker: PhantomData,
            },
        ));
    }

    mesh_uniforms.0.write_buffer(&render_device, &render_queue);
    tilemap_uniforms
        .0
        .write_buffer(&render_device, &render_queue);
}

pub fn prepare_removal(
    mut chunk_storage: ResMut<RenderChunk2dStorage>,
    removed_tiles: Query<&ExtractedRemovedTile>,
    removed_maps: Query<&ExtractedRemovedMap>,
) {
    for removed_tile in removed_tiles.iter() {
        chunk_storage.remove_tile_with_entity(removed_tile.entity)
    }

    for removed_map in removed_maps.iter() {
        chunk_storage.remove_map(removed_map.entity);
    }
}
