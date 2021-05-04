use dyn_clone::DynClone;
use async_trait::async_trait;
use bevy::{ecs::component::Component, prelude::*, render::{mesh::{Indices, VertexAttributeValues}, pipeline::PrimitiveTopology}};
use crate::{chunk::ChunkSettings, morton_index, prelude::*};

/// TODO: DOCS
#[async_trait]
pub trait TilemapChunkMesher : Component + DynClone + std::fmt::Debug {
    async fn mesh(self: Box<Self>, chunk: ChunkSettings, tile_query: Vec<Option<Tile>>) -> (Handle<Mesh>, Mesh);

    // TODO: remove once proper iso/hex culling is implemented.
    fn should_cull(&self) -> bool;
}

/// TODO: DOCS
#[derive(Debug, Clone)]
pub struct SquareChunkMesher;

#[async_trait]
impl TilemapChunkMesher for SquareChunkMesher {
    async fn mesh(self: Box<Self>, chunk: ChunkSettings, tile_query: Vec<Option<Tile>>) -> (Handle<Mesh>, Mesh) {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
    
        let columns = (chunk.texture_size.x / chunk.tile_size.x).floor();
    
        let mut i = 0;
        for x in 0..chunk.size.x {
            for y in 0..chunk.size.y {
                let tile_position = UVec2::new((chunk.position.x * chunk.size.x) + x, (chunk.position.y * chunk.size.y) + y);
                if let Some(tile) = tile_query[morton_index(UVec2::new(x, y))] {
                    // log::info!("Getting vertices for tile at: {:?}", tile_position);

                    let tile_pixel_pos = Vec2::new(
                        tile_position.x as f32 * chunk.tile_size.x,
                        tile_position.y as f32 * chunk.tile_size.y
                    );

                    // X, Y
                    positions.push([tile_pixel_pos.x, tile_pixel_pos.y, 0.0]);
                    // X, Y + 1
                    positions.push([tile_pixel_pos.x, tile_pixel_pos.y + chunk.tile_size.y, 0.0]);
                    // X + 1, Y + 1
                    positions.push([tile_pixel_pos.x + chunk.tile_size.x, tile_pixel_pos.y + chunk.tile_size.y, 0.0]);
                    // X + 1, Y
                    positions.push([tile_pixel_pos.x + chunk.tile_size.x, tile_pixel_pos.y, 0.0]);

                    // This calculation is much simpler we only care about getting the remainder
                    // and multiplying that by the tile width.
                    let sprite_sheet_x: f32 =
                        ((tile.texture_index as f32 % columns) * chunk.tile_size.x).floor();

                    // Calculation here is (tile / columns).round_down * (tile_space + tile_height) - tile_space
                    // Example: tile 30 / 28 columns = 1.0714 rounded down to 1 * 16 tile_height = 16 Y
                    // which is the 2nd row in the sprite sheet.
                    // Example2: tile 10 / 28 columns = 0.3571 rounded down to 0 * 16 tile_height = 0 Y
                    // which is the 1st row in the sprite sheet.
                    let sprite_sheet_y: f32 =
                        (tile.texture_index as f32 / columns).floor() * chunk.tile_size.y;

                    // Calculate UV:
                    let start_u: f32 = sprite_sheet_x / chunk.texture_size.x;
                    let end_u: f32 = (sprite_sheet_x + chunk.tile_size.x) / chunk.texture_size.x;
                    let start_v: f32 = sprite_sheet_y / chunk.texture_size.y;
                    let end_v: f32 = (sprite_sheet_y + chunk.tile_size.y) / chunk.texture_size.y;

                    let mut new_uv = [
                        // X, Y
                        [start_u, end_v],
                        // X, Y + 1
                        [start_u, start_v],
                        // X + 1, Y + 1
                        [end_u, start_v],
                        // X + 1, Y
                        [end_u, end_v],
                    ];

                    if tile.flip_x {
                        new_uv.reverse();
                    }
                    if tile.flip_y {
                        new_uv.reverse();
                        new_uv.swap(0, 2);
                        new_uv.swap(1, 3);
                    }

                    new_uv.iter().for_each(|uv| uvs.push(*uv));

                    indices.extend_from_slice(&[i + 0, i + 2, i + 1, i + 0, i + 3, i + 2]);
                    i += 4;
                }
            }
        }
        mesh.set_attribute("Vertex_Position", VertexAttributeValues::Float3(positions.clone()));
        mesh.set_attribute("Vertex_Normal", VertexAttributeValues::Float3(positions));
        mesh.set_attribute("Vertex_Uv", VertexAttributeValues::Float2(uvs));
        mesh.set_indices(Some(Indices::U32(indices)));

        (chunk.mesh_handle, mesh)
    }

    fn should_cull(&self) -> bool {
        true
    }
}

/// TODO: DOCS
#[derive(Debug, Clone)]
pub enum HexType {
    RowEven,
    RowOdd,
    ColumnEven,
    ColumnOdd,
    Row,
    Column,
}

#[derive(Debug, Clone)]
pub struct HexChunkMesher {
    hex_type: HexType,
}

impl HexChunkMesher {
    pub fn new(hex_type: HexType) -> Self {
        Self {
            hex_type,
        }
    }

    fn offset_coords(&self, actual_pos: UVec2, mut pos: Vec3, tile_size: Vec2) -> Vec3 {
        match self.hex_type {
            HexType::RowEven => {
                let offset = (0.25 * tile_size.x).floor();
                if actual_pos.y % 2 == 0 {
                    pos.x -= offset;
                } else {
                    pos.x += offset;
                }
                pos.y -= actual_pos.y as f32 * (0.25 * tile_size.y).ceil();
                pos
            },
            HexType::RowOdd => {
                let offset = (0.25 * tile_size.x).floor();
                if actual_pos.y % 2 == 0 {
                    pos.x += offset;
                } else {
                    pos.x -= offset;
                }
                pos.y -= actual_pos.y as f32 * (0.25 * tile_size.y).ceil();
                pos
            },
            HexType::ColumnEven => {
                let offset = (0.25 * tile_size.y).floor();
                if actual_pos.x % 2 == 0 {
                    pos.y -= offset;
                } else {
                    pos.y += offset;
                }
                pos.x -= actual_pos.x as f32 * (0.25 * tile_size.x).ceil();
                pos
            },
            HexType::ColumnOdd => {
                let offset = (0.25 * tile_size.y).floor();
                if actual_pos.x % 2 == 0 {
                    pos.y += offset;
                } else {
                    pos.y -= offset;
                }
                pos.x -= actual_pos.x as f32 * (0.25 * tile_size.x).ceil();
                pos
            },
            HexType::Row => {
                pos.x += actual_pos.y as f32 * (0.5 * tile_size.x).floor();
                pos.y -= actual_pos.y as f32 * (0.25 * tile_size.y).ceil();
                pos
            },
            HexType::Column => {
                pos.x += actual_pos.x as f32 * (-0.25 * tile_size.x).floor();
                pos.y += actual_pos.x as f32 * (0.5 * tile_size.y).ceil();
                pos
            },
        }
    }
}

#[async_trait]
impl TilemapChunkMesher for HexChunkMesher {
    async fn mesh(self: Box<Self>, chunk: ChunkSettings, tile_query: Vec<Option<Tile>>) -> (Handle<Mesh>, Mesh) {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
    
        let columns = (chunk.texture_size.x / chunk.tile_size.x).floor();
    
        let mut i = 0;
        for x in 0..chunk.size.x {
            for y in 0..chunk.size.y {
                let tile_position = UVec2::new((chunk.position.x * chunk.size.x) + x, (chunk.position.y * chunk.size.y) + y);
                if let Some(tile) = tile_query[morton_index(UVec2::new(x, y))] {
                    let tile_pixel_pos = Vec2::new(
                        tile_position.x as f32 * chunk.tile_size.x,
                        tile_position.y as f32 * chunk.tile_size.y
                    );
                    
                    // X, Y
                    positions.push([tile_pixel_pos.x, tile_pixel_pos.y, 0.0]);
                    // X, Y + 1
                    positions.push([tile_pixel_pos.x, tile_pixel_pos.y + chunk.tile_size.y, 0.0]);
                    // X + 1, Y + 1
                    positions.push([tile_pixel_pos.x + chunk.tile_size.x, tile_pixel_pos.y + chunk.tile_size.y, 0.0]);
                    // X + 1, Y
                    positions.push([tile_pixel_pos.x + chunk.tile_size.x, tile_pixel_pos.y, 0.0]);

                    for j in 0..4 as i32 {
                        positions[(i as i32 + j) as usize] = self.offset_coords(
                            tile_position,
                            positions[(i as i32 + j) as usize].into(),
                            chunk.tile_size
                        ).into();
                    }

                    // This calculation is much simpler we only care about getting the remainder
                    // and multiplying that by the tile width.
                    let sprite_sheet_x: f32 =
                        ((tile.texture_index as f32 % columns) * chunk.tile_size.x).floor();

                    // Calculation here is (tile / columns).round_down * (tile_space + tile_height) - tile_space
                    // Example: tile 30 / 28 columns = 1.0714 rounded down to 1 * 16 tile_height = 16 Y
                    // which is the 2nd row in the sprite sheet.
                    // Example2: tile 10 / 28 columns = 0.3571 rounded down to 0 * 16 tile_height = 0 Y
                    // which is the 1st row in the sprite sheet.
                    let sprite_sheet_y: f32 =
                        (tile.texture_index as f32 / columns).floor() * chunk.tile_size.y;

                    // Calculate UV:
                    let start_u: f32 = sprite_sheet_x / chunk.texture_size.x;
                    let end_u: f32 = (sprite_sheet_x + chunk.tile_size.x) / chunk.texture_size.x;
                    let start_v: f32 = sprite_sheet_y / chunk.texture_size.y;
                    let end_v: f32 = (sprite_sheet_y + chunk.tile_size.y) / chunk.texture_size.y;

                    let mut new_uv = vec![
                        [start_u, end_v],
                        [start_u, start_v],
                        [end_u, start_v],
                        [end_u, end_v],
                    ];

                    if tile.flip_x {
                        new_uv.reverse();
                    }
                    if tile.flip_y {
                        new_uv.reverse();
                        new_uv.swap(0, 2);
                        new_uv.swap(1, 3);
                    }

                    uvs.extend(new_uv);


                    indices.extend_from_slice(&[i + 0, i + 2, i + 1, i + 0, i + 3, i + 2]);
                    i += 4;
                }
            }
        }
        mesh.set_attribute("Vertex_Position", VertexAttributeValues::Float3(positions.clone()));
        mesh.set_attribute("Vertex_Normal", VertexAttributeValues::Float3(positions));
        mesh.set_attribute("Vertex_Uv", VertexAttributeValues::Float2(uvs));
        mesh.set_indices(Some(Indices::U32(indices)));

        (chunk.mesh_handle, mesh)
    }

    fn should_cull(&self) -> bool {
        false
    }
}

/// TODO: DOCS
#[derive(Debug, Clone)]
pub struct IsoChunkMesher;

impl IsoChunkMesher {
    fn project_iso(pos: Vec2, tile_width: f32, tile_height: f32) -> Vec2 {
        let x = (pos.x - pos.y) * tile_width / 2.0;
        let y = (pos.x + pos.y) * tile_height / 2.0;
        Vec2::new(x, -y)
    }
}

#[async_trait]
impl TilemapChunkMesher for IsoChunkMesher {
    async fn mesh(self: Box<Self>, chunk: ChunkSettings, tile_query: Vec<Option<Tile>>) -> (Handle<Mesh>, Mesh) {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
    
        let columns = (chunk.texture_size.x / chunk.tile_size.x).floor();
        let mut i = 0;
        for x in 0..chunk.size.x {
            for y in 0..chunk.size.y {
                let tile_position = UVec2::new((chunk.position.x * chunk.size.x) + x, (chunk.position.y * chunk.size.y) + y);
                if let Some(tile) = tile_query[morton_index(UVec2::new(x, y))] {
                    // log::info!("Getting vertices for tile at: {:?}", tile_position);

                    let tile_pixel_pos = Vec2::new(
                        tile_position.x as f32,
                        tile_position.y as f32
                    );

                    let center = Self::project_iso(tile_pixel_pos, chunk.tile_size.x, chunk.tile_size.y);

                    let start = Vec2::new(
                        center.x - chunk.tile_size.x / 2.0,
                        center.y - chunk.tile_size.y,
                    );

                    let end = Vec2::new(center.x + chunk.tile_size.x / 2.0, center.y);

                    // X, Y
                    positions.push([start.x, start.y, 0.0]);
                    // X, Y + 1
                    positions.push([start.x, end.y, 0.0]);
                    // X + 1, Y + 1
                    positions.push([end.x, end.y, 0.0]);
                    // X + 1, Y
                    positions.push([end.x, start.y, 0.0]);

                    // This calculation is much simpler we only care about getting the remainder
                    // and multiplying that by the tile width.
                    let sprite_sheet_x: f32 =
                        ((tile.texture_index as f32 % columns) * chunk.tile_size.x).floor();

                    // Calculation here is (tile / columns).round_down * (tile_space + tile_height) - tile_space
                    // Example: tile 30 / 28 columns = 1.0714 rounded down to 1 * 16 tile_height = 16 Y
                    // which is the 2nd row in the sprite sheet.
                    // Example2: tile 10 / 28 columns = 0.3571 rounded down to 0 * 16 tile_height = 0 Y
                    // which is the 1st row in the sprite sheet.
                    let sprite_sheet_y: f32 =
                        (tile.texture_index as f32 / columns).floor() * chunk.tile_size.y;

                    // Calculate UV:
                    let start_u: f32 = sprite_sheet_x / chunk.texture_size.x;
                    let end_u: f32 = (sprite_sheet_x + chunk.tile_size.x) / chunk.texture_size.x;
                    let start_v: f32 = sprite_sheet_y / chunk.texture_size.y;
                    let end_v: f32 = (sprite_sheet_y + chunk.tile_size.y) / chunk.texture_size.y;

                    let mut new_uv = vec![
                        [start_u, end_v],
                        [start_u, start_v],
                        [end_u, start_v],
                        [end_u, end_v],
                    ];

                    if tile.flip_x {
                        new_uv.reverse();
                    }
                    if tile.flip_y {
                        new_uv.reverse();
                        new_uv.swap(0, 2);
                        new_uv.swap(1, 3);
                    }

                    uvs.extend(new_uv);

                    indices.extend_from_slice(&[i + 0, i + 2, i + 1, i + 0, i + 3, i + 2]);
                    i += 4;
                }
            }
        }
        mesh.set_attribute("Vertex_Position", VertexAttributeValues::Float3(positions.clone()));
        mesh.set_attribute("Vertex_Normal", VertexAttributeValues::Float3(positions));
        mesh.set_attribute("Vertex_Uv", VertexAttributeValues::Float2(uvs));
        mesh.set_indices(Some(Indices::U32(indices)));

        (chunk.mesh_handle, mesh)
    }

    fn should_cull(&self) -> bool {
        false
    }
}