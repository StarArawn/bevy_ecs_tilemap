use std::hash::{Hash, Hasher};

use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::Buffer;
use bevy::render::{mesh::BaseMeshPipelineKey, primitives::Aabb};
use bevy::{math::Mat4, render::mesh::PrimitiveTopology};
use bevy::{
    math::{UVec2, UVec3, UVec4, Vec2, Vec3Swizzles, Vec4, Vec4Swizzles},
    prelude::{Component, Entity, GlobalTransform, Mesh, Vec3},
    render::{
        mesh::{Indices, RenderMesh, RenderMeshBufferInfo, VertexAttributeValues},
        render_resource::{BufferInitDescriptor, BufferUsages, ShaderType},
        renderer::RenderDevice,
    },
    utils::HashMap,
};
use bevy::{
    prelude::{InheritedVisibility, Resource, Transform},
    render::mesh::MeshVertexBufferLayouts,
};

use crate::prelude::helpers::transform::{chunk_aabb, chunk_index_to_world_space};
use crate::render::extract::ExtractedFrustum;
use crate::{
    map::{TilemapSize, TilemapTexture, TilemapType},
    tiles::TilePos,
    FrustumCulling, TilemapGridSize, TilemapTileSize,
};

use super::RenderChunkSize;

#[derive(Resource, Default, Clone, Debug)]
pub struct RenderChunk2dStorage {
    chunks: HashMap<u32, HashMap<UVec3, RenderChunk2d>>,
    entity_to_chunk_tile: HashMap<Entity, (u32, UVec3, UVec2)>,
    entity_to_chunk: HashMap<Entity, UVec3>,
}

#[derive(Default, Component, Clone, Copy, Debug)]
pub struct ChunkId(pub UVec3);

impl RenderChunk2dStorage {
    #[allow(clippy::too_many_arguments)]
    pub fn get_or_add(
        &mut self,
        tile_entity: Entity,
        tile_pos: UVec2,
        chunk_entity: Entity,
        position: &UVec4,
        chunk_size: UVec2,
        mesh_type: TilemapType,
        tile_size: TilemapTileSize,
        texture_size: Vec2,
        spacing: Vec2,
        grid_size: TilemapGridSize,
        texture: TilemapTexture,
        map_size: TilemapSize,
        transform: GlobalTransform,
        visibility: &InheritedVisibility,
        frustum_culling: &FrustumCulling,
        render_size: RenderChunkSize,
        y_sort: bool,
    ) -> &mut RenderChunk2d {
        let pos = position.xyz();

        self.entity_to_chunk_tile
            .insert(tile_entity, (position.w, pos, tile_pos));

        let chunk_storage = if self.chunks.contains_key(&position.w) {
            self.chunks.get_mut(&position.w).unwrap()
        } else {
            let hash_map = HashMap::default();
            self.chunks.insert(position.w, hash_map);
            self.chunks.get_mut(&position.w).unwrap()
        };

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        position.hash(&mut hasher);

        if chunk_storage.contains_key(&pos) {
            chunk_storage.get_mut(&pos).unwrap()
        } else {
            let chunk = RenderChunk2d::new(
                hasher.finish(),
                chunk_entity.to_bits(),
                &pos,
                chunk_size,
                mesh_type,
                tile_size,
                spacing,
                grid_size,
                texture,
                texture_size,
                map_size,
                transform,
                visibility.get(),
                **frustum_culling,
                render_size,
                y_sort,
            );
            self.entity_to_chunk.insert(chunk_entity, pos);
            chunk_storage.insert(pos, chunk);
            chunk_storage.get_mut(&pos).unwrap()
        }
    }

    pub fn get(&self, position: &UVec4) -> Option<&RenderChunk2d> {
        if let Some(chunk_storage) = self.chunks.get(&position.w) {
            return chunk_storage.get(&position.xyz());
        }
        None
    }

    pub fn get_mut(&mut self, position: &UVec4) -> &mut RenderChunk2d {
        let chunk_storage = self.chunks.get_mut(&position.w).unwrap();
        chunk_storage.get_mut(&position.xyz()).unwrap()
    }

    pub fn remove_tile_with_entity(&mut self, entity: Entity) {
        if let Some((chunk, tile_pos)) = self.get_mut_from_entity(entity) {
            chunk.set(&tile_pos.into(), None);
        }

        self.entity_to_chunk.remove(&entity);
        self.entity_to_chunk_tile.remove(&entity);
    }

    pub fn get_mut_from_entity(&mut self, entity: Entity) -> Option<(&mut RenderChunk2d, UVec2)> {
        if !self.entity_to_chunk_tile.contains_key(&entity) {
            return None;
        }

        let (tilemap_id, chunk_pos, tile_pos) = self.entity_to_chunk_tile.get(&entity).unwrap();

        let chunk_storage = self.chunks.get_mut(tilemap_id).unwrap();
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

    pub fn iter(&self) -> impl Iterator<Item = &RenderChunk2d> {
        self.chunks.iter().flat_map(|(_, x)| x.iter().map(|x| x.1))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut RenderChunk2d> {
        self.chunks
            .iter_mut()
            .flat_map(|(_, x)| x.iter_mut().map(|x| x.1))
    }

    pub fn remove_map(&mut self, entity: Entity) {
        self.chunks.remove(&entity.index());
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PackedTileData {
    pub visible: bool,
    pub position: Vec4,
    pub texture: Vec4,
    pub color: [f32; 4],
}

#[derive(Clone, Debug)]
pub struct RenderChunk2d {
    pub id: u64,
    pub tilemap_id: u64,
    /// The index of the chunk. It is equivalent to the position of the chunk in "chunk
    /// coordinates".
    index: UVec3,
    /// The position of this chunk, in world space,
    position: Vec2,
    /// Size of the chunk, in tiles.
    pub size_in_tiles: UVec2,
    /// [`TilemapSize`] of the map this chunk belongs to.
    pub map_size: TilemapSize,
    /// [`TilemapType`] of the map this chunk belongs to.
    map_type: TilemapType,
    /// The grid size of the map this chunk belongs to.
    pub grid_size: TilemapGridSize,
    /// The tile size of the map this chunk belongs to.
    pub tile_size: TilemapTileSize,
    /// The [`Aabb`] of this chunk, based on the map type, grid size, and tile size. It is not
    /// transformed by the `global_transform` or [`local_transform`]
    aabb: Aabb,
    local_transform: Transform,
    /// The [`GlobalTransform`] of this chunk, stored as a [`Transform`].
    global_transform: Transform,
    /// The product of the local and global transforms.
    transform: Transform,
    /// The matrix computed from this chunk's `transform`.
    transform_matrix: Mat4,
    pub spacing: Vec2,
    pub tiles: Vec<Option<PackedTileData>>,
    pub texture: TilemapTexture,
    pub texture_size: Vec2,
    pub mesh: Mesh,
    pub render_mesh: Option<RenderMesh>,
    pub vertex_buffer: Option<Buffer>,
    pub index_buffer: Option<Buffer>,
    pub dirty_mesh: bool,
    pub visible: bool,
    pub frustum_culling: bool,
    pub render_size: RenderChunkSize,
    pub y_sort: bool,
}

impl RenderChunk2d {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: u64,
        tilemap_id: u64,
        index: &UVec3,
        size_in_tiles: UVec2,
        map_type: TilemapType,
        tile_size: TilemapTileSize,
        spacing: Vec2,
        grid_size: TilemapGridSize,
        texture: TilemapTexture,
        texture_size: Vec2,
        map_size: TilemapSize,
        global_transform: GlobalTransform,
        visible: bool,
        frustum_culling: bool,
        render_size: RenderChunkSize,
        y_sort: bool,
    ) -> Self {
        let position = chunk_index_to_world_space(index.xy(), size_in_tiles, &grid_size, &map_type);
        let local_transform = Transform::from_translation(position.extend(0.0));
        let global_transform: Transform = global_transform.into();
        let transform = local_transform * global_transform;
        let transform_matrix = transform.compute_matrix();
        let aabb = chunk_aabb(size_in_tiles, &grid_size, &tile_size, &map_type);
        Self {
            dirty_mesh: true,
            render_mesh: None,
            id,
            index: *index,
            position,
            size_in_tiles,
            map_size,
            map_type,
            grid_size,
            tile_size,
            aabb,
            local_transform,
            global_transform,
            transform,
            transform_matrix,
            mesh: Mesh::new(
                bevy::render::render_resource::PrimitiveTopology::TriangleList,
                RenderAssetUsages::default(),
            ),
            vertex_buffer: None,
            index_buffer: None,
            spacing,
            texture_size,
            texture,
            tilemap_id,
            tiles: vec![None; (size_in_tiles.x * size_in_tiles.y) as usize],
            visible,
            frustum_culling,
            render_size,
            y_sort,
        }
    }

    pub fn get(&self, tile_pos: &TilePos) -> &Option<PackedTileData> {
        &self.tiles[tile_pos.to_index(&self.size_in_tiles.into())]
    }

    pub fn get_mut(&mut self, tile_pos: &TilePos) -> &mut Option<PackedTileData> {
        self.dirty_mesh = true;
        &mut self.tiles[tile_pos.to_index(&self.size_in_tiles.into())]
    }

    pub fn set(&mut self, tile_pos: &TilePos, tile: Option<PackedTileData>) {
        self.dirty_mesh = true;
        self.tiles[tile_pos.to_index(&self.size_in_tiles.into())] = tile;
    }

    pub fn get_index(&self) -> UVec3 {
        self.index
    }

    pub fn get_map_type(&self) -> TilemapType {
        self.map_type
    }

    pub fn get_transform(&self) -> Transform {
        self.transform
    }

    pub fn get_transform_matrix(&self) -> Mat4 {
        self.transform_matrix
    }

    pub fn intersects_frustum(&self, frustum: &ExtractedFrustum) -> bool {
        frustum.intersects_obb(&self.aabb, &self.transform_matrix)
    }

    pub fn update_geometry(
        &mut self,
        global_transform: Transform,
        grid_size: TilemapGridSize,
        tile_size: TilemapTileSize,
        map_type: TilemapType,
    ) {
        let mut dirty_local_transform = false;

        if self.grid_size != grid_size || self.tile_size != tile_size || self.map_type != map_type {
            self.grid_size = grid_size;
            self.map_type = map_type;
            self.tile_size = tile_size;

            self.position = chunk_index_to_world_space(
                self.index.xy(),
                self.size_in_tiles,
                &self.grid_size,
                &self.map_type,
            );

            self.local_transform = Transform::from_translation(self.position.extend(0.0));
            dirty_local_transform = true;

            self.aabb = chunk_aabb(
                self.size_in_tiles,
                &self.grid_size,
                &self.tile_size,
                &self.map_type,
            );
        }

        let mut dirty_global_transform = false;
        if self.global_transform != global_transform {
            self.global_transform = global_transform;
            dirty_global_transform = true;
        }

        if dirty_local_transform || dirty_global_transform {
            self.transform = global_transform * self.local_transform;
            self.transform_matrix = self.transform.compute_matrix();
        }
    }

    pub fn prepare(
        &mut self,
        device: &RenderDevice,
        mesh_vertex_buffer_layouts: &mut MeshVertexBufferLayouts,
    ) {
        if self.dirty_mesh {
            let size = ((self.size_in_tiles.x * self.size_in_tiles.y) * 4) as usize;
            let mut positions: Vec<[f32; 4]> = Vec::with_capacity(size);
            let mut textures: Vec<[f32; 4]> = Vec::with_capacity(size);
            let mut colors: Vec<[f32; 4]> = Vec::with_capacity(size);
            let mut indices: Vec<u32> =
                Vec::with_capacity(((self.size_in_tiles.x * self.size_in_tiles.y) * 6) as usize);

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

                colors.extend(std::iter::repeat(tile.color).take(4));

                // flipping and rotation packed in bits
                // bit 0 : flip_x
                // bit 1 : flip_y
                // bit 2 : flip_d (anti diagonal)

                // let tile_flip_bits =
                //     tile.flip_x as i32 | (tile.flip_y as i32) << 1 | (tile.flip_d as i32) << 2;

                //let texture: [f32; 4] = tile.texture.xyxx().into();
                let texture: [f32; 4] = tile.texture.to_array();
                textures.extend([texture, texture, texture, texture].into_iter());

                indices.extend_from_slice(&[i, i + 2, i + 1, i, i + 3, i + 2]);
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
            self.mesh.insert_indices(Indices::U32(indices));

            let vertex_buffer_data = self.mesh.create_packed_vertex_buffer_data();
            let vertex_buffer = device.create_buffer_with_data(&BufferInitDescriptor {
                usage: BufferUsages::VERTEX,
                label: Some("Mesh Vertex Buffer"),
                contents: &vertex_buffer_data,
            });

            let index_buffer = device.create_buffer_with_data(&BufferInitDescriptor {
                usage: BufferUsages::INDEX,
                contents: self.mesh.get_index_buffer_bytes().unwrap(),
                label: Some("Mesh Index Buffer"),
            });

            let buffer_info = RenderMeshBufferInfo::Indexed {
                count: self.mesh.indices().unwrap().len() as u32,
                index_format: self.mesh.indices().unwrap().into(),
            };

            let mesh_vertex_buffer_layout = self
                .mesh
                .get_mesh_vertex_buffer_layout(mesh_vertex_buffer_layouts);
            self.render_mesh = Some(RenderMesh {
                vertex_count: self.mesh.count_vertices() as u32,
                buffer_info,
                morph_targets: None,
                layout: mesh_vertex_buffer_layout,
                key_bits: BaseMeshPipelineKey::from_primitive_topology(
                    PrimitiveTopology::TriangleList,
                ),
            });
            self.vertex_buffer = Some(vertex_buffer);
            self.index_buffer = Some(index_buffer);
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
        let chunk_ix: Vec2 = chunk.index.xy().as_vec2();
        let chunk_size: Vec2 = chunk.size_in_tiles.as_vec2();
        let map_size: Vec2 = chunk.map_size.into();
        let tile_size: Vec2 = chunk.tile_size.into();
        Self {
            texture_size: chunk.texture_size,
            tile_size,
            grid_size: chunk.grid_size.into(),
            spacing: chunk.spacing,
            chunk_pos: chunk_ix * chunk_size,
            map_size: map_size * tile_size,
            time: 0.0,
            pad: Vec3::ZERO,
        }
    }
}

impl From<&mut RenderChunk2d> for TilemapUniformData {
    fn from(chunk: &mut RenderChunk2d) -> Self {
        let chunk_pos: Vec2 = chunk.index.xy().as_vec2();
        let chunk_size: Vec2 = chunk.size_in_tiles.as_vec2();
        let map_size: Vec2 = chunk.map_size.into();
        let tile_size: Vec2 = chunk.tile_size.into();
        Self {
            texture_size: chunk.texture_size,
            tile_size,
            grid_size: chunk.grid_size.into(),
            spacing: chunk.spacing,
            chunk_pos: chunk_pos * chunk_size,
            map_size: map_size * tile_size,
            time: 0.0,
            pad: Vec3::ZERO,
        }
    }
}
