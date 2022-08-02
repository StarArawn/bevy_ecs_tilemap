use std::hash::{Hash, Hasher};

use bevy::{
    math::{UVec2, UVec3, UVec4, Vec2, Vec3Swizzles, Vec4, Vec4Swizzles},
    prelude::{Component, Entity, GlobalTransform, Mesh, Vec3},
    render::{
        mesh::{GpuBufferInfo, GpuMesh, Indices, VertexAttributeValues},
        render_resource::{BufferInitDescriptor, BufferUsages, ShaderType},
        renderer::RenderDevice,
    },
    utils::HashMap,
};

use crate::{
    map::{Tilemap2dSize, TilemapMeshType, TilemapTexture},
    tiles::TilePos2d,
};

#[derive(Default, Clone, Debug)]
pub struct RenderChunk2dStorage {
    chunks: HashMap<u32, HashMap<UVec3, RenderChunk2d>>,
    entity_to_chunk_tile: HashMap<Entity, (u32, UVec3, UVec2)>,
}

#[derive(Default, Component, Clone, Copy, Debug)]
pub struct ChunkId(pub UVec3);

impl RenderChunk2dStorage {
    pub fn get_or_add(
        &mut self,
        tile_entity: Entity,
        tile_pos: UVec2,
        position: &UVec4,
        chunk_size: UVec2,
        mesh_type: TilemapMeshType,
        tile_size: Vec2,
        texture_size: Vec2,
        spacing: Vec2,
        texture: TilemapTexture,
        map_size: Tilemap2dSize,
        transform: GlobalTransform,
    ) -> &mut RenderChunk2d {
        let pos = position.xyz();

        self.entity_to_chunk_tile
            .insert(tile_entity, (position.w, pos, tile_pos));

        let chunk_storage = self.get_chunk_storage(position);

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        position.hash(&mut hasher);

        if chunk_storage.contains_key(&pos) {
            chunk_storage.get_mut(&pos).unwrap()
        } else {
            let chunk = RenderChunk2d::new(
                hasher.finish(),
                position.w,
                &pos,
                chunk_size,
                mesh_type,
                tile_size,
                spacing,
                texture,
                texture_size,
                map_size,
                transform,
            );
            chunk_storage.insert(pos, chunk);
            chunk_storage.get_mut(&pos).unwrap()
        }
    }

    pub fn get(&self, position: &UVec4) -> &RenderChunk2d {
        let chunk_storage = self.chunks.get(&position.w).unwrap();
        chunk_storage.get(&position.xyz()).unwrap()
    }

    pub fn get_mut(&mut self, position: &UVec4) -> &mut RenderChunk2d {
        let chunk_storage = self.chunks.get_mut(&position.w).unwrap();
        chunk_storage.get_mut(&position.xyz()).unwrap()
    }

    pub fn get_mut_from_entity(&mut self, entity: Entity) -> Option<(&mut RenderChunk2d, UVec2)> {
        if !self.entity_to_chunk_tile.contains_key(&entity) {
            return None;
        }

        let (tilemap_id, chunk_pos, tile_pos) = self.entity_to_chunk_tile.get(&entity).unwrap();

        let chunk_storage = self.chunks.get_mut(&tilemap_id).unwrap();
        Some((chunk_storage.get_mut(&chunk_pos.xyz()).unwrap(), *tile_pos))
    }

    pub fn get_chunk_storage(&mut self, position: &UVec4) -> &mut HashMap<UVec3, RenderChunk2d> {
        if self.chunks.contains_key(&position.w) {
            self.chunks.get_mut(&position.w).unwrap()
        } else {
            let hash_map = HashMap::default();
            self.chunks.insert(position.w, hash_map);
            self.chunks.get_mut(&position.w).unwrap()
        }
    }

    pub fn remove(&mut self, position: &UVec4) {
        let chunk_storage = self.get_chunk_storage(position);

        let pos = position.xyz();

        chunk_storage.remove(&pos);
    }

    pub fn count(&self) -> usize {
        self.chunks.len()
    }

    pub fn iter(&self) -> impl std::iter::Iterator<Item = &RenderChunk2d> {
        self.chunks.iter().flat_map(|(_, x)| x.iter().map(|x| x.1))
    }

    pub fn iter_mut(&mut self) -> impl std::iter::Iterator<Item = &mut RenderChunk2d> {
        self.chunks
            .iter_mut()
            .flat_map(|(_, x)| x.iter_mut().map(|x| x.1))
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PackedTileData {
    pub visible: bool,
    pub position: Vec4,
    pub texture: Vec4,
    pub color: Vec4,
}

#[derive(Clone, Debug)]
pub struct RenderChunk2d {
    pub id: u64,
    pub position: UVec3,
    pub size: UVec2,
    pub mesh_type: TilemapMeshType,
    pub tile_size: Vec2,
    pub tilemap_id: u32,
    pub spacing: Vec2,
    pub tiles: Vec<Option<PackedTileData>>,
    pub texture: TilemapTexture,
    pub texture_size: Vec2,
    pub map_size: Tilemap2dSize,
    pub mesh: Mesh,
    pub gpu_mesh: Option<GpuMesh>,
    pub dirty_mesh: bool,
    pub transform: GlobalTransform,
}

impl RenderChunk2d {
    pub fn new(
        id: u64,
        tilemap_id: u32,
        position: &UVec3,
        size: UVec2,
        mesh_type: TilemapMeshType,
        tile_size: Vec2,
        spacing: Vec2,
        texture: TilemapTexture,
        texture_size: Vec2,
        map_size: Tilemap2dSize,
        transform: GlobalTransform,
    ) -> Self {
        Self {
            dirty_mesh: true,
            gpu_mesh: None,
            id,
            map_size,
            mesh_type,
            mesh: Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList),
            position: *position,
            size,
            spacing,
            texture_size,
            texture,
            tile_size,
            tilemap_id,
            tiles: vec![None; (size.x * size.y) as usize],
            transform,
        }
    }

    pub fn get(&self, tile_pos: &TilePos2d) -> &Option<PackedTileData> {
        &self.tiles[crate::helpers::pos_2d_to_index(tile_pos, &self.size.into())]
    }

    pub fn get_mut(&mut self, tile_pos: &TilePos2d) -> &mut Option<PackedTileData> {
        self.dirty_mesh = true;
        &mut self.tiles[crate::helpers::pos_2d_to_index(tile_pos, &self.size.into())]
    }

    pub fn set(&mut self, tile_pos: &TilePos2d, tile: Option<PackedTileData>) {
        self.dirty_mesh = true;
        self.tiles[crate::helpers::pos_2d_to_index(tile_pos, &self.size.into())] = tile;
    }

    pub fn prepare(&mut self, device: &RenderDevice) {
        if self.dirty_mesh {
            let size = ((self.size.x * self.size.y) * 4) as usize;
            let mut positions: Vec<[f32; 4]> = Vec::with_capacity(size);
            let mut textures: Vec<[f32; 4]> = Vec::with_capacity(size);
            let mut colors: Vec<[f32; 4]> = Vec::with_capacity(size);
            let mut indices: Vec<u32> =
                Vec::with_capacity(((self.size.x * self.size.y) * 6) as usize);

            let mut i = 0;

            // Convert tile into mesh data.
            for tile in self.tiles.iter().filter_map(|x| x.as_ref()) {
                if !tile.visible {
                    continue;
                }

                let position: [f32; 4] = tile.position.to_array();
                positions.extend(
                    [
                        // X, Y
                        position,
                        // X, Y + 1
                        //[tile_pos.x, tile_pos.y + 1.0, animation_speed],
                        position,
                        // X + 1, Y + 1
                        //[tile_pos.x + 1.0, tile_pos.y + 1.0, animation_speed],
                        position,
                        // X + 1, Y
                        //[tile_pos.x + 1.0, tile_pos.y, animation_speed],
                        position,
                    ]
                    .into_iter(),
                );

                let color: [f32; 4] = tile.color.into();
                colors.extend([color, color, color, color].into_iter());

                // flipping and rotation packed in bits
                // bit 0 : flip_x
                // bit 1 : flip_y
                // bit 2 : flip_d (anti diagonal)

                // let tile_flip_bits =
                //     tile.flip_x as i32 | (tile.flip_y as i32) << 1 | (tile.flip_d as i32) << 2;

                //let texture: [f32; 4] = tile.texture.xyxx().into();
                let texture: [f32; 4] = tile.texture.to_array();
                textures.extend([texture, texture, texture, texture].into_iter());

                indices.extend_from_slice(&[i + 0, i + 2, i + 1, i + 0, i + 3, i + 2]);
                i += 4;
            }

            self.mesh.insert_attribute(
                crate::render::ATTRIBUTE_POSITION,
                VertexAttributeValues::Float32x4(positions),
            );
            self.mesh.insert_attribute(
                crate::render::ATTRIBUTE_TEXTURE,
                VertexAttributeValues::Float32x4(textures),
            );
            self.mesh.insert_attribute(
                crate::render::ATTRIBUTE_COLOR,
                VertexAttributeValues::Float32x4(colors),
            );
            self.mesh.set_indices(Some(Indices::U32(indices)));

            let vertex_buffer_data = self.mesh.get_vertex_buffer_data();
            let vertex_buffer = device.create_buffer_with_data(&BufferInitDescriptor {
                usage: BufferUsages::VERTEX,
                label: Some("Mesh Vertex Buffer"),
                contents: &vertex_buffer_data,
            });

            let buffer_info = self.mesh.get_index_buffer_bytes().map_or(
                GpuBufferInfo::NonIndexed {
                    vertex_count: self.mesh.count_vertices() as u32,
                },
                |data| GpuBufferInfo::Indexed {
                    buffer: device.create_buffer_with_data(&BufferInitDescriptor {
                        usage: BufferUsages::INDEX,
                        contents: data,
                        label: Some("Mesh Index Buffer"),
                    }),
                    count: self.mesh.indices().unwrap().len() as u32,
                    index_format: self.mesh.indices().unwrap().into(),
                },
            );

            let mesh_vertex_buffer_layout = self.mesh.get_mesh_vertex_buffer_layout();
            self.gpu_mesh = Some(GpuMesh {
                vertex_buffer,
                buffer_info,
                layout: mesh_vertex_buffer_layout,
                primitive_topology: bevy::render::render_resource::PrimitiveTopology::TriangleList,
            });
            self.dirty_mesh = false;
        }
    }
}

// Used to transfer info to the GPU for tile building.
#[derive(Debug, Default, Copy, Component, Clone, ShaderType)]
pub struct TilemapUniformData {
    pub texture_size: Vec2,
    pub tile_size: Vec2,
    pub grid_size: Vec2,
    pub spacing: Vec2,
    pub chunk_pos: Vec2,
    pub map_size: Vec2,
    pub time: f32,
    pub pad: Vec3,
}

impl From<&RenderChunk2d> for TilemapUniformData {
    fn from(chunk: &RenderChunk2d) -> Self {
        let chunk_pos: Vec2 = chunk.position.xy().as_vec2();
        let chunk_size: Vec2 = chunk.size.as_vec2();
        let map_size: Vec2 = chunk.map_size.into();
        Self {
            texture_size: chunk.texture_size.into(),
            tile_size: chunk.tile_size.into(),
            grid_size: chunk.tile_size.into(),
            spacing: chunk.spacing.into(),
            chunk_pos: chunk_pos * chunk_size,
            map_size: map_size * chunk.tile_size,
            time: 0.0,
            pad: Vec3::ZERO,
        }
    }
}

impl From<&mut RenderChunk2d> for TilemapUniformData {
    fn from(chunk: &mut RenderChunk2d) -> Self {
        let chunk_pos: Vec2 = chunk.position.xy().as_vec2();
        let chunk_size: Vec2 = chunk.size.as_vec2();
        let map_size: Vec2 = chunk.map_size.into();
        Self {
            texture_size: chunk.texture_size.into(),
            tile_size: chunk.tile_size.into(),
            grid_size: chunk.tile_size.into(),
            spacing: chunk.spacing.into(),
            chunk_pos: chunk_pos * chunk_size,
            map_size: map_size * chunk.tile_size,
            time: 0.0,
            pad: Vec3::ZERO,
        }
    }
}
