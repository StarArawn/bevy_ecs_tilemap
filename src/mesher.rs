use dyn_clone::DynClone;
use bevy::{ecs::component::Component, prelude::*, render::mesh::{Indices, VertexAttributeValues}};
use crate::prelude::*;

pub trait TilemapChunkMesher : Component + DynClone {
    fn mesh(&self, chunk: &Chunk, meshes: &mut ResMut<Assets<Mesh>>, tile_query: &Query<(&MapVec2, &Tile)>);
}

#[derive(Debug, Clone)]
pub struct SquareChunkMesher;

impl TilemapChunkMesher for SquareChunkMesher {
    fn mesh(&self, chunk: &Chunk, meshes: &mut ResMut<Assets<Mesh>>, tile_query: &Query<(&MapVec2, &Tile)>) {
        let mesh = meshes.get_mut(chunk.mesh_handle.clone()).unwrap();
        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
    
        let columns = (chunk.texture_size.x / chunk.tile_size.x).floor();
    
        let mut i = 0;
        for x in 0..chunk.size.x {
            for y in 0..chunk.size.y {
                let tile_position = MapVec2::new((chunk.position.x * chunk.size.x) + x, (chunk.position.y * chunk.size.y) + y);
                if let Some(tile_entity) = chunk.tiles.get(&tile_position) {
                    if let Ok((_, tile)) = tile_query.get(*tile_entity) {
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
    
                        uvs.push([start_u, end_v]);
                        uvs.push([start_u, start_v]);
                        uvs.push([end_u, start_v]);
                        uvs.push([end_u, end_v]);
    
                        indices.extend_from_slice(&[i + 0, i + 2, i + 1, i + 0, i + 3, i + 2]);
                        i += 4;
                    }
                }
            }
        }
        mesh.set_attribute("Vertex_Position", VertexAttributeValues::Float3(positions.clone()));
        mesh.set_attribute("Vertex_Normal", VertexAttributeValues::Float3(positions));
        mesh.set_attribute("Vertex_Uv", VertexAttributeValues::Float2(uvs));
        mesh.set_indices(Some(Indices::U32(indices)));
    }
}


// TODO: Add more meshing types for hexagons.
#[derive(Debug, Clone)]
pub enum HexType {
    RowEven,
    RowOdd,
    ColumnEven,
    ColumnOdd,
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

    fn offset_coords(&self, actual_pos: MapVec2, mut pos: Vec3, tile_size: Vec2) -> Vec3 {
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
        }
    }
}

impl TilemapChunkMesher for HexChunkMesher {
    fn mesh(&self, chunk: &Chunk, meshes: &mut ResMut<Assets<Mesh>>, tile_query: &Query<(&MapVec2, &Tile)>) {
        let mesh = meshes.get_mut(chunk.mesh_handle.clone()).unwrap();
        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
    
        let columns = (chunk.texture_size.x / chunk.tile_size.x).floor();
    
        let mut i = 0;
        for x in 0..chunk.size.x {
            for y in 0..chunk.size.y {
                let tile_position = MapVec2::new((chunk.position.x * chunk.size.x) + x, (chunk.position.y * chunk.size.y) + y);
                if let Some(tile_entity) = chunk.tiles.get(&tile_position) {
                    if let Ok((_, tile)) = tile_query.get(*tile_entity) {    
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
    
                        uvs.push([start_u, end_v]);
                        uvs.push([start_u, start_v]);
                        uvs.push([end_u, start_v]);
                        uvs.push([end_u, end_v]);
    
                        indices.extend_from_slice(&[i + 0, i + 2, i + 1, i + 0, i + 3, i + 2]);
                        i += 4;
                    }
                }
            }
        }
        mesh.set_attribute("Vertex_Position", VertexAttributeValues::Float3(positions.clone()));
        mesh.set_attribute("Vertex_Normal", VertexAttributeValues::Float3(positions));
        mesh.set_attribute("Vertex_Uv", VertexAttributeValues::Float2(uvs));
        mesh.set_indices(Some(Indices::U32(indices)));
    }
}