use std::marker::PhantomData;

use bevy::{
    math::{Mat4, UVec2, UVec4, Vec3Swizzles},
    prelude::{Commands, Component, Entity, GlobalTransform, Query, Res, ResMut, Transform},
    render::{
        render_resource::{DynamicUniformBuffer, ShaderType},
        renderer::{RenderDevice, RenderQueue},
    },
};

use crate::render::SecondsSinceStartup;
use crate::{
    helpers::get_chunk_2d_transform,
    map::{
        Tilemap2dSize, Tilemap2dSpacing, Tilemap2dTextureSize, Tilemap2dTileSize, TilemapId,
        TilemapMeshType, TilemapTexture,
    },
    tiles::TilePos2d,
};

pub const CHUNK_SIZE_2D: UVec2 = UVec2::from_array([64, 64]);

fn map_tile_to_chunk(tile_position: &TilePos2d) -> UVec2 {
    let tile_pos: UVec2 = tile_position.into();
    tile_pos / CHUNK_SIZE_2D
}

pub(crate) fn map_tile_to_chunk_tile(tile_position: &TilePos2d, chunk_position: &UVec2) -> UVec2 {
    let tile_pos: UVec2 = tile_position.into();
    tile_pos - (*chunk_position * CHUNK_SIZE_2D)
}

use super::{
    chunk::{ChunkId, PackedTileData, RenderChunk2dStorage, TilemapUniformData},
    extract::{ExtractedRemovedTile, ExtractedTile},
    DynamicUniformIndex,
};

#[derive(ShaderType, Component, Clone)]
pub struct MeshUniform {
    pub transform: Mat4,
}

pub fn prepare(
    mut commands: Commands,
    mut chunk_storage: ResMut<RenderChunk2dStorage>,
    mut mesh_uniforms: ResMut<DynamicUniformBuffer<MeshUniform>>,
    mut tilemap_uniforms: ResMut<DynamicUniformBuffer<TilemapUniformData>>,
    extracted_tiles: Query<&ExtractedTile>,
    extracted_tilemaps: Query<(
        Entity,
        &GlobalTransform,
        &Tilemap2dTileSize,
        &Tilemap2dTextureSize,
        &Tilemap2dSpacing,
        &TilemapMeshType,
        &TilemapTexture,
        &Tilemap2dSize,
    )>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
    seconds_since_startup: Res<SecondsSinceStartup>,
) {
    for tile in extracted_tiles.iter() {
        let chunk_pos = map_tile_to_chunk(&tile.position);
        let (_entity, transform, tile_size, texture_size, spacing, mesh_type, texture, map_size) =
            extracted_tilemaps.get(tile.tilemap_id.0).unwrap();

        let chunk_data = UVec4::new(
            chunk_pos.x,
            chunk_pos.y,
            transform.translation().z as u32,
            tile.tilemap_id.0.id(),
        );

        let chunk = chunk_storage.get_or_add(
            tile.entity,
            map_tile_to_chunk_tile(&tile.position, &chunk_pos).into(),
            &chunk_data,
            CHUNK_SIZE_2D,
            *mesh_type,
            (*tile_size).into(),
            (*texture_size).into(),
            (*spacing).into(),
            texture.clone(),
            *map_size,
            *transform,
        );
        chunk.set(
            &map_tile_to_chunk_tile(&tile.position, &chunk_pos).into(),
            Some(PackedTileData {
                position: map_tile_to_chunk_tile(&tile.position, &chunk_pos)
                    .as_vec2()
                    .extend(tile.tile.position.z)
                    .extend(tile.tile.position.w),
                ..tile.tile
            }),
        );
    }

    // Copies transform changes from tilemap to chunks.
    for (entity, transform, _, _, _, mesh_type, _, _) in extracted_tilemaps.iter() {
        let chunks = chunk_storage.get_chunk_storage(&UVec4::new(0, 0, 0, entity.id()));
        for chunk in chunks.values_mut() {
            chunk.mesh_type = *mesh_type;
            chunk.transform = *transform;
        }
    }

    mesh_uniforms.clear();
    tilemap_uniforms.clear();

    for chunk in chunk_storage.iter_mut() {
        chunk.prepare(&render_device);

        let chunk_global_transform: Transform = chunk.transform.into();

        let transform = get_chunk_2d_transform(
            chunk.position.as_vec3().xy(),
            chunk.tile_size,
            chunk.size.as_vec2(),
            0,
            chunk.mesh_type,
        ) * chunk_global_transform;

        let mut chunk_uniform: TilemapUniformData = chunk.into();
        chunk_uniform.time = **seconds_since_startup;

        commands
            .spawn()
            .insert(chunk.texture.0.clone_weak())
            .insert(transform)
            .insert(ChunkId(chunk.position))
            .insert(chunk.mesh_type)
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
) {
    for removed_tile in removed_tiles.iter() {
        if let Some((chunk, tile_pos)) = chunk_storage.get_mut_from_entity(removed_tile.entity) {
            chunk.set(&tile_pos.into(), None);
        }
    }
}
