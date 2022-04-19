use std::marker::PhantomData;

use bevy::{
    math::{const_uvec2, Mat4, UVec2, UVec4, Vec3Swizzles},
    prelude::{Commands, Component, Entity, Query, Res, ResMut, Transform},
    render::{
        render_resource::{std140::AsStd140, DynamicUniformVec},
        renderer::{RenderDevice, RenderQueue},
    },
};

use crate::{
    helpers::get_chunk_2d_transform,
    map::{
        Tilemap2dSize, Tilemap2dSpacing, Tilemap2dTextureSize, Tilemap2dTileSize, TilemapId,
        TilemapMeshType, TilemapTexture,
    },
    tiles::TilePos2d,
};

pub const CHUNK_SIZE_2D: UVec2 = const_uvec2!([64, 64]);

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
    extract::ExtractedTile,
    DynamicUniformIndex,
};

#[derive(AsStd140, Component, Clone)]
pub struct MeshUniform {
    pub transform: Mat4,
}

pub fn prepare(
    mut commands: Commands,
    mut chunk_storage: ResMut<RenderChunk2dStorage>,
    mut mesh_uniforms: ResMut<DynamicUniformVec<MeshUniform>>,
    mut tilemap_uniforms: ResMut<DynamicUniformVec<TilemapUniformData>>,
    extracted_tiles: Query<&ExtractedTile>,
    extracted_tilemaps: Query<(
        Entity,
        &Transform,
        &Tilemap2dTileSize,
        &Tilemap2dTextureSize,
        &Tilemap2dSpacing,
        &TilemapMeshType,
        &TilemapTexture,
        &Tilemap2dSize,
    )>,
    render_device: Res<RenderDevice>,
    render_queue: Res<RenderQueue>,
) {
    for tile in extracted_tiles.iter() {
        let chunk_pos = map_tile_to_chunk(&tile.position);
        let (_entity, transform, tile_size, texture_size, spacing, mesh_type, texture, map_size) =
            extracted_tilemaps.get(tile.tilemap_id.0).unwrap();

        let chunk_data = UVec4::new(
            chunk_pos.x,
            chunk_pos.y,
            transform.translation.z as u32,
            tile.tilemap_id.0.id(),
        );

        let chunk = chunk_storage.get_or_add(
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
                    .extend(0.0)
                    .extend(0.0),
                ..tile.tile
            }),
        );
    }

    mesh_uniforms.clear();
    tilemap_uniforms.clear();

    for chunk in chunk_storage.iter_mut() {
        chunk.prepare(&render_device);

        let transform = get_chunk_2d_transform(
            chunk.position.as_vec3().xy(),
            chunk.tile_size,
            chunk.size.as_vec2(),
            0,
            chunk.mesh_type,
        ) * chunk.transform;

        let chunk_uniform: TilemapUniformData = chunk.into();

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
