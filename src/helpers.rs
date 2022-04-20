use bevy::{
    math::{UVec3, Vec2},
    prelude::{Commands, Transform},
};

use crate::{
    map::{Tilemap2dSize, Tilemap2dTileSize, TilemapId, TilemapMeshType},
    tiles::{Tile2dStorage, TilePos2d, TileTexture, TileVisible},
};

pub fn pos_2d_to_index(tile_pos: &TilePos2d, size: &Tilemap2dSize) -> usize {
    ((tile_pos.y * size.x as u32) + tile_pos.x) as usize
}

pub fn uvec3_to_index(position: &UVec3, size: UVec3) -> usize {
    ((position.z * size.x * size.y) + (position.y * size.x) + position.x) as usize
}

pub fn get_chunk_2d_transform(
    chunk_position: Vec2,
    grid_size: Vec2,
    chunk_size: Vec2,
    z_index: u32,
    _mesh_type: TilemapMeshType,
) -> Transform {
    Transform::from_xyz(
        chunk_position.x as f32 * chunk_size.x * grid_size.x,
        chunk_position.y as f32 * chunk_size.y * grid_size.x,
        z_index as f32,
    )
}

pub fn fill_tilemap(
    tile_texture: TileTexture,
    size: Tilemap2dSize,
    tilemap_id: TilemapId,
    commands: &mut Commands,
    tile_storage: &mut Tile2dStorage,
) {
    for x in 0..size.x {
        for y in 0..size.y {
            let tile_pos = TilePos2d { x, y };
            let tile_entity = commands
                .spawn()
                .insert(tile_pos)
                .insert(tile_texture)
                .insert(tilemap_id)
                .insert(TileVisible(true))
                .id();
            tile_storage.set(&tile_pos, Some(tile_entity));
        }
    }
}

pub fn get_centered_transform_2d(
    size: &Tilemap2dSize,
    tile_size: &Tilemap2dTileSize,
    z_index: f32,
) -> Transform {
    Transform::from_xyz(
        -(size.x as f32 * tile_size.x as f32) / 2.0,
        -(size.y as f32 * tile_size.y as f32) / 2.0,
        z_index,
    )
}
