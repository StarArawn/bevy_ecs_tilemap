use std::marker::PhantomData;

use bevy::{
    math::{Mat4, UVec4, Vec3Swizzles},
    prelude::{
        Commands, Component, ComputedVisibility, Entity, GlobalTransform, Query, Res, ResMut,
        Transform, Vec2,
    },
    render::{
        render_resource::{DynamicUniformBuffer, ShaderType},
        renderer::{RenderDevice, RenderQueue},
    },
};

use crate::helpers::transform::chunk_index_to_world_space;
use crate::{
    map::{
        TilemapId, TilemapSize, TilemapSpacing, TilemapTexture, TilemapTextureSize,
        TilemapTileSize, TilemapType,
    },
};
use crate::{prelude::TilemapGridSize, render::RenderChunkSize, render::SecondsSinceStartup};

use super::{
    chunk::{ChunkId, PackedTileData, RenderChunk2dStorage, TilemapUniformData},
    extract::{ExtractedRemovedMap, ExtractedRemovedTile, ExtractedTile, ExtractedTilemapTexture},
    DynamicUniformIndex,
};

#[derive(ShaderType, Component, Clone)]
pub struct MeshUniform {
    pub transform: Mat4,
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(crate) fn prepare(
    mut commands: Commands,
    mut chunk_storage: ResMut<RenderChunk2dStorage>,
    mut mesh_uniforms: ResMut<DynamicUniformBuffer<MeshUniform>>,
    mut tilemap_uniforms: ResMut<DynamicUniformBuffer<TilemapUniformData>>,
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
        &ComputedVisibility,
    )>,
    extracted_tilemap_textures: Query<&ExtractedTilemapTexture>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    seconds_since_startup: Res<SecondsSinceStartup>,
) {
    for tile in extracted_tiles.iter() {
        let chunk_pos = chunk_size.map_tile_to_chunk(&tile.position);
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
        ) = extracted_tilemaps.get(tile.tilemap_id.0).unwrap();

        let scale = transform.compute_transform().scale;

        let chunk_data = UVec4::new(
            chunk_pos.x * scale.x as u32,
            chunk_pos.y * scale.y as u32,
            transform.translation().z as u32,
            tile.tilemap_id.0.id(),
        );

        let relative_tile_pos = chunk_size.map_tile_to_chunk_tile(&tile.position, &chunk_pos);
        let chunk = chunk_storage.get_or_add(
            tile.entity,
            relative_tile_pos,
            &chunk_data,
            **chunk_size,
            *mesh_type,
            (*tile_size).into(),
            (*texture_size).into(),
            (*spacing).into(),
            (*grid_size).into(),
            texture.clone(),
            *map_size,
            *transform,
            visibility,
        );
        chunk.set(
            &relative_tile_pos.into(),
            Some(PackedTileData {
                position: chunk_size
                    .map_tile_to_chunk_tile(&tile.position, &chunk_pos)
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
        transform,
        tile_size,
        texture_size,
        spacing,
        grid_size,
        mesh_type,
        texture,
        map_size,
        visibility,
    ) in extracted_tilemaps.iter()
    {
        let chunks = chunk_storage.get_chunk_storage(&UVec4::new(0, 0, 0, entity.id()));
        for chunk in chunks.values_mut() {
            chunk.map_type = *mesh_type;
            chunk.transform = *transform;
            chunk.texture = texture.clone();
            chunk.map_size = *map_size;
            chunk.tile_size = (*tile_size).into();
            chunk.texture_size = (*texture_size).into();
            chunk.spacing = (*spacing).into();
            chunk.grid_size = (*grid_size).into();
            chunk.visible = visibility.is_visible();
        }
    }

    for tilemap in extracted_tilemap_textures.iter() {
        let texture_size: Vec2 = tilemap.texture_size.into();
        let chunks =
            chunk_storage.get_chunk_storage(&UVec4::new(0, 0, 0, tilemap.tilemap_id.0.id()));
        for chunk in chunks.values_mut() {
            chunk.texture_size = texture_size;
        }
    }

    mesh_uniforms.clear();
    tilemap_uniforms.clear();

    for chunk in chunk_storage.iter_mut() {
        if !chunk.visible {
            continue;
        }

        chunk.prepare(&render_device);

        let chunk_global_transform: Transform = chunk.transform.into();

        let chunk_pos = chunk_index_to_world_space(
            chunk.position.xy(),
            chunk.size,
            chunk.grid_size,
            &chunk.map_type,
        );
        let chunk_transform = Transform::from_xyz(chunk_pos.x, chunk_pos.y, 0.0);

        let transform = chunk_transform * chunk_global_transform;

        let mut chunk_uniform: TilemapUniformData = chunk.into();
        chunk_uniform.time = **seconds_since_startup;

        commands
            .spawn()
            .insert(chunk.texture.0.clone_weak())
            .insert(transform)
            .insert(ChunkId(chunk.position))
            .insert(chunk.map_type)
            .insert(TilemapId(Entity::from_raw(chunk.tilemap_id)))
            .insert(DynamicUniformIndex::<MeshUniform> {
                index: mesh_uniforms.push(MeshUniform {
                    transform: transform.compute_matrix(),
                }),
                marker: PhantomData,
            })
            .insert(DynamicUniformIndex::<TilemapUniformData> {
                index: tilemap_uniforms.push(chunk_uniform),
                marker: PhantomData,
            });
    }

    mesh_uniforms.write_buffer(&render_device, &render_queue);
    tilemap_uniforms.write_buffer(&render_device, &render_queue);
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
