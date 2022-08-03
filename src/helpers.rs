use bevy::{
    math::Vec2,
    prelude::{Commands, Transform},
};

use crate::{
    map::{HexType, IsoType, TilemapId, TilemapMeshType, TilemapSize, TilemapTileSize},
    tiles::{TileBundle, TilePos, TileStorage, TileTexture},
};

/// Converts a tile position into an index in a vector.
pub fn pos_2d_to_index(tile_pos: &TilePos, size: &TilemapSize) -> usize {
    ((tile_pos.y * size.x as u32) + tile_pos.x) as usize
}

/// Calculates a chunk position with the given information.
/// Note: The calculation is different depending on the tilemap's mesh type.
/// This calculation is mostly used internally for rendering but it might be helpful so it's exposed here.
pub fn get_chunk_2d_transform(
    chunk_position: Vec2,
    grid_size: Vec2,
    chunk_size: Vec2,
    z_index: u32,
    mesh_type: TilemapMeshType,
) -> Transform {
    let pos = match mesh_type {
        TilemapMeshType::Square => {
            let chunk_pos_x = chunk_position.x * chunk_size.x * grid_size.x;
            let chunk_pos_y = chunk_position.y * chunk_size.y * grid_size.y;
            Vec2::new(chunk_pos_x, chunk_pos_y)
        }
        TilemapMeshType::Hexagon(HexType::Row) => {
            let chunk_pos_x = (chunk_position.y * chunk_size.x * (0.5 * grid_size.x).floor())
                + (chunk_position.x * chunk_size.x * grid_size.x);
            let chunk_pos_y = chunk_position.y * chunk_size.y * (0.75 * grid_size.y).floor();
            Vec2::new(chunk_pos_x, chunk_pos_y)
        }
        TilemapMeshType::Hexagon(HexType::RowOdd) | TilemapMeshType::Hexagon(HexType::RowEven) => {
            let chunk_pos_x = chunk_position.x * chunk_size.x * grid_size.x;
            let chunk_pos_y = chunk_position.y * chunk_size.y * (0.75 * grid_size.y).floor();
            Vec2::new(chunk_pos_x, chunk_pos_y)
        }
        TilemapMeshType::Hexagon(HexType::Column) => {
            let chunk_pos_x = chunk_position.x * chunk_size.x * (0.75 * grid_size.x).floor();
            let chunk_pos_y = (chunk_position.x * chunk_size.y * (0.5 * grid_size.y).ceil())
                + chunk_position.y * chunk_size.y * grid_size.y;
            Vec2::new(chunk_pos_x, chunk_pos_y)
        }
        TilemapMeshType::Hexagon(HexType::ColumnOdd)
        | TilemapMeshType::Hexagon(HexType::ColumnEven) => {
            let chunk_pos_x = chunk_position.x * chunk_size.x * (0.75 * grid_size.x).floor();
            let chunk_pos_y = chunk_position.y * chunk_size.y * grid_size.y;
            Vec2::new(chunk_pos_x, chunk_pos_y)
        }
        TilemapMeshType::Isometric(IsoType::Diamond) => project_iso_diamond(
            chunk_position.x,
            chunk_position.y,
            chunk_size.x * grid_size.x,
            chunk_size.y * grid_size.y,
        ),
        TilemapMeshType::Isometric(IsoType::Staggered) => project_iso_staggered(
            chunk_position.x,
            chunk_position.y,
            chunk_size.x * grid_size.x,
            chunk_size.y,
        ),
    };

    Transform::from_xyz(pos.x, pos.y, z_index as f32)
}

/// Fills an entire tile storage with the given tile.
pub fn fill_tilemap(
    tile_texture: TileTexture,
    size: TilemapSize,
    tilemap_id: TilemapId,
    commands: &mut Commands,
    tile_storage: &mut TileStorage,
) {
    for x in 0..size.x {
        for y in 0..size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn()
                .insert_bundle(TileBundle {
                    position: tile_pos,
                    tilemap_id: tilemap_id,
                    texture: tile_texture,
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, Some(tile_entity));
        }
    }
}

/// Fills a rectangular region with the given tile.
pub fn fill_tilemap_rect(
    tile_texture: TileTexture,
    pos: TilePos,
    size: TilemapSize,
    tilemap_id: TilemapId,
    commands: &mut Commands,
    tile_storage: &mut TileStorage,
) {
    for x in pos.x..size.x {
        for y in pos.y..size.y {
            let tile_pos = TilePos { x, y };
            let tile_entity = commands
                .spawn()
                .insert_bundle(TileBundle {
                    position: tile_pos,
                    tilemap_id: tilemap_id,
                    texture: tile_texture,
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, Some(tile_entity));
        }
    }
}

/// Calculates a tilemap's centered position.
pub fn get_centered_transform_2d(
    size: &TilemapSize,
    tile_size: &TilemapTileSize,
    z_index: f32,
) -> Transform {
    Transform::from_xyz(
        -(size.x as f32 * tile_size.x as f32) / 2.0,
        -(size.y as f32 * tile_size.y as f32) / 2.0,
        z_index,
    )
}

/// Projects a 2D screen space point into isometric diamond space.
pub fn project_iso_diamond(x: f32, y: f32, pixel_width: f32, pixel_height: f32) -> Vec2 {
    let new_x = (x - y) * pixel_width / 2.0;
    let new_y = (x + y) * pixel_height / 2.0;
    Vec2::new(new_x, -new_y)
}

/// Projects a 2D screen space point into isometric staggered space.
pub fn project_iso_staggered(x: f32, y: f32, pixel_width: f32, pixel_height: f32) -> Vec2 {
    let new_x = x * pixel_width;
    let new_y = y * pixel_height;
    Vec2::new(new_x, new_y)
}
