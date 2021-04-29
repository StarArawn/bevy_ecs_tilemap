use std::collections::HashMap;
use bevy::{prelude::*, render::{mesh::{Indices, VertexAttributeValues}, pipeline::RenderPipeline, render_graph::base::MainPass}};
use crate::{map_vec2::MapVec2, render::pipeline::TILE_MAP_PIPELINE_HANDLE, tile::Tile};

#[derive(Bundle)]
pub struct ChunkBundle {
    pub chunk: Chunk,
    pub main_pass: MainPass,
    pub material: Handle<ColorMaterial>,
    pub render_pipeline: RenderPipelines,
    pub visible: Visible,
    pub draw: Draw,
    pub mesh: Handle<Mesh>,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

impl Default for ChunkBundle {
    fn default() -> Self {
        Self {
            chunk: Chunk::default(),
            visible: Visible {
                is_transparent: true,
                ..Default::default()
            },
            draw: Draw::default(),
            main_pass: MainPass,
            mesh: Handle::default(),
            material: Handle::default(),
            render_pipeline: RenderPipelines::from_pipelines(vec![RenderPipeline::new(
                TILE_MAP_PIPELINE_HANDLE.typed(),
            )]),
            transform: Transform::default(),
            global_transform: GlobalTransform::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Chunk {
    map_entity: Entity,
    pub position: MapVec2,
    pub size: MapVec2,
    pub(crate) tiles: HashMap<MapVec2, Entity>,
    mesh_handle: Handle<Mesh>,
    tile_size: Vec2,
    texture_size: Vec2,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            map_entity: Entity::new(0),
            position: Default::default(),
            size: Default::default(),
            tiles: HashMap::new(),
            mesh_handle: Default::default(),
            texture_size: Vec2::ZERO,
            tile_size: Vec2::ZERO,
        }
    }
}

impl Chunk {
    pub(crate) fn new(map_entity: Entity, position: MapVec2, chunk_size: MapVec2, tile_size: Vec2, texture_size: Vec2, mesh_handle: Handle<Mesh>) -> Self {
        let tiles = HashMap::new();
        Self {
            map_entity,
            position,
            tiles,
            size: chunk_size,
            tile_size,
            texture_size,
            mesh_handle,
        }
    }

    pub(crate) fn build_tiles(&mut self, commands: &mut Commands, chunk_entity: Entity) {
        for x in 0..self.size.x {
            for y in 0..self.size.y {
                let tile_pos = MapVec2 {
                    x: (self.position.x * self.size.x) + x,
                    y: (self.position.y * self.size.y) + y,
                };
                let tile_entity = commands.spawn()
                    .insert(Tile {
                        chunk: chunk_entity,
                        size: self.tile_size,
                        ..Tile::default()
                    })
                    .insert(tile_pos).id();
                self.tiles.insert(tile_pos, tile_entity);
            }
        }
    }

    pub fn get_tile_entity(&self, position: MapVec2) -> Option<&Entity> {
        self.tiles.get(&position)
    }
}

pub(crate) fn update_chunk_mesh(
    mut meshes: ResMut<Assets<Mesh>>,
    tile_change_query: Query<&Tile, Or<(Changed<MapVec2>, Changed<Tile>)>>,
    tile_query: Query<(&MapVec2, &Tile)>,
    mut chunk_query: Query<&Chunk>,
) {
    let mut updated_chunks = Vec::new();
    // If a tile has changed.
    for tile in tile_change_query.iter() {
        if let Ok(chunk) = chunk_query.get_mut(tile.chunk) {
            if !updated_chunks.iter().any(|position| chunk.position == *position) {
                // Rebuild tile map mesh.
                // log::info!("Re-meshing chunk at: {:?}", chunk.position);

                calculate_mesh(chunk, &mut meshes, &tile_query);

                // Make sure we don't recalculate the chunk until the next time this system updates at least.
                updated_chunks.push(chunk.position);
            }
        }
    }
}

pub fn calculate_mesh(chunk: &Chunk, meshes: &mut ResMut<Assets<Mesh>>, tile_query: &Query<(&MapVec2, &Tile)>) {
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
                        tile_position.x as f32 * tile.size.x,
                        tile_position.y as f32 * tile.size.y
                    );

                    // X, Y
                    positions.push([tile_pixel_pos.x, tile_pixel_pos.y, 0.0]);
                    // X, Y + 1
                    positions.push([tile_pixel_pos.x, tile_pixel_pos.y + tile.size.y, 0.0]);
                    // X + 1, Y + 1
                    positions.push([tile_pixel_pos.x + tile.size.x, tile_pixel_pos.y + tile.size.y, 0.0]);
                    // X + 1, Y
                    positions.push([tile_pixel_pos.x + tile.size.x, tile_pixel_pos.y, 0.0]);

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