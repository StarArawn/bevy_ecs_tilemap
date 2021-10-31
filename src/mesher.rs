use std::array::IntoIter;

use crate::{prelude::*, tile::GPUAnimated};
use bevy::{
    prelude::*,
    render::mesh::{Indices, VertexAttributeValues},
};

#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct ChunkMesher;

impl ChunkMesher {
    pub fn mesh(
        &self,
        chunk: &Chunk,
        chunk_tiles: &Vec<Option<Entity>>,
        tile_query: &Query<(&TilePos, &Tile, Option<&GPUAnimated>)>,
        meshes: &mut ResMut<Assets<Mesh>>,
    ) {
        let mesh = meshes.get_mut(chunk.mesh_handle.clone()).unwrap();
        let size = ((chunk.settings.chunk_size.0 * chunk.settings.chunk_size.1) * 4) as usize;
        let mut positions: Vec<[f32; 3]> = Vec::with_capacity(size);
        let mut textures: Vec<[i32; 4]> = Vec::with_capacity(size);
        let mut colors: Vec<[f32; 4]> = Vec::with_capacity(size);
        let mut indices: Vec<u32> = Vec::with_capacity(
            ((chunk.settings.chunk_size.0 * chunk.settings.chunk_size.1) * 6) as usize,
        );

        let mut i = 0;
        for tile_entity in chunk_tiles.iter() {
            if let Some(tile_entity) = tile_entity {
                if let Ok((tile_position, tile, gpu_animated)) = tile_query.get(*tile_entity) {
                    if !tile.visible {
                        continue;
                    }

                    let tile_pos = Vec2::new(
                        (tile_position.0 - (chunk.position.0 * chunk.settings.chunk_size.0)) as f32,
                        (tile_position.1 - (chunk.position.1 * chunk.settings.chunk_size.1)) as f32,
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

                    colors.extend(IntoIter::new([
                        [
                            tile.color.r(),
                            tile.color.g(),
                            tile.color.b(),
                            tile.color.a(),
                        ],
                        [
                            tile.color.r(),
                            tile.color.g(),
                            tile.color.b(),
                            tile.color.a(),
                        ],
                        [
                            tile.color.r(),
                            tile.color.g(),
                            tile.color.b(),
                            tile.color.a(),
                        ],
                        [
                            tile.color.r(),
                            tile.color.g(),
                            tile.color.b(),
                            tile.color.a(),
                        ],
                    ]));

                    // flipping and rotation packed in bits
                    // bit 0 : flip_x
                    // bit 1 : flip_y
                    // bit 2 : flip_d (anti diagonal)

                    let tile_flip_bits =
                        tile.flip_x as i32 | (tile.flip_y as i32) << 1 | (tile.flip_d as i32) << 2;

                    textures.extend(IntoIter::new([
                        [
                            tile.texture_index as i32,
                            tile_flip_bits,
                            animation_start,
                            animation_end,
                        ],
                        [
                            tile.texture_index as i32,
                            tile_flip_bits,
                            animation_start,
                            animation_end,
                        ],
                        [
                            tile.texture_index as i32,
                            tile_flip_bits,
                            animation_start,
                            animation_end,
                        ],
                        [
                            tile.texture_index as i32,
                            tile_flip_bits,
                            animation_start,
                            animation_end,
                        ],
                    ]));

                    indices.extend_from_slice(&[i + 0, i + 2, i + 1, i + 0, i + 3, i + 2]);
                    i += 4;
                }
            }
        }
        mesh.set_attribute("Vertex_Position", VertexAttributeValues::Float3(positions));
        mesh.set_attribute("Vertex_Texture", VertexAttributeValues::Int4(textures));
        mesh.set_attribute("Vertex_Color", VertexAttributeValues::Float4(colors));
        mesh.set_indices(Some(Indices::U32(indices)));
    }
}
