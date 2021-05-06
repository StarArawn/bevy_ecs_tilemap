use std::array::IntoIter;

use crate::{chunk::ChunkSettings, prelude::*, tile::GPUAnimated};
use bevy::{
    ecs::component::Component,
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        pipeline::PrimitiveTopology,
    },
};
use dyn_clone::DynClone;

// TODO: Drop DynClone from here.

pub(crate) trait TilemapChunkMesher: Component + DynClone + std::fmt::Debug {
    fn mesh(
        &self,
        chunk: ChunkSettings,
        chunk_tiles: &Vec<Option<Entity>>,
        tile_query: &Query<(&UVec2, &Tile, Option<&GPUAnimated>), With<VisibleTile>>,
    ) -> (Handle<Mesh>, Mesh);

    fn should_cull(&self) -> bool;
}

#[derive(Debug, Clone)]
pub(crate) struct SquareChunkMesher;

impl TilemapChunkMesher for SquareChunkMesher {
    fn mesh(
        &self,
        chunk: ChunkSettings,
        chunk_tiles: &Vec<Option<Entity>>,
        tile_query: &Query<(&UVec2, &Tile, Option<&GPUAnimated>), With<VisibleTile>>,
    ) -> (Handle<Mesh>, Mesh) {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        let size = ((chunk.size.x * chunk.size.y) * 4) as usize;
        let mut positions: Vec<[f32; 3]> = Vec::with_capacity(size);
        let mut textures: Vec<[i32; 4]> = Vec::with_capacity(size);
        let mut indices: Vec<u32> =
            Vec::with_capacity(((chunk.size.x * chunk.size.y) * 6) as usize);

        let mut i = 0;
        for tile_entity in chunk_tiles.iter() {
            if let Some(tile_entity) = tile_entity {
                if let Ok((tile_position, tile, gpu_animated)) = tile_query.get(*tile_entity) {
                    let tile_pos = Vec2::new(
                        (tile_position.x - (chunk.position.x * chunk.size.x)) as f32,
                        (tile_position.y - (chunk.position.y * chunk.size.y)) as f32,
                    );
                    let (animation_start, animation_end, animation_speed) =
                        if let Some(ani) = gpu_animated {
                            (ani.start as i32, ani.end as i32, ani.speed)
                        } else {
                            (tile.texture_index as i32, tile.texture_index as i32, 0.0)
                        };

                    positions.extend(IntoIter::new([
                        // X, Y
                        [tile_pos.x, tile_pos.y, animation_speed],
                        // X, Y + 1
                        //[tile_pos.x, tile_pos.y + 1.0, animation_speed],
                        [tile_pos.x, tile_pos.y, animation_speed],
                        // X + 1, Y + 1
                        //[tile_pos.x + 1.0, tile_pos.y + 1.0, animation_speed],
                        [tile_pos.x, tile_pos.y, animation_speed],
                        // X + 1, Y
                        //[tile_pos.x + 1.0, tile_pos.y, animation_speed],
                        [tile_pos.x, tile_pos.y, animation_speed],
                    ]));

                    let tile_flip_bits = match (tile.flip_x, tile.flip_y) {
                        // no flip
                        (false, false) => 0,
                        // flip x
                        (true, false) => 1,
                        // flip y
                        (false, true) => 2,
                        // flip both
                        (true, true) => 3,
                    };
                    
                    textures.extend(IntoIter::new([
                        [tile.texture_index as i32, tile_flip_bits, animation_start, animation_end],
                        [tile.texture_index as i32, tile_flip_bits, animation_start, animation_end],
                        [tile.texture_index as i32, tile_flip_bits, animation_start, animation_end],
                        [tile.texture_index as i32, tile_flip_bits, animation_start, animation_end],
                    ]));

                    indices.extend_from_slice(&[i + 0, i + 2, i + 1, i + 0, i + 3, i + 2]);
                    i += 4;
                }
            }
        }
        mesh.set_attribute("Vertex_Position", VertexAttributeValues::Float3(positions));
        mesh.set_attribute("Vertex_Texture", VertexAttributeValues::Int4(textures));
        mesh.set_indices(Some(Indices::U32(indices)));

        (chunk.mesh_handle, mesh)
    }

    fn should_cull(&self) -> bool {
        true
    }
}
