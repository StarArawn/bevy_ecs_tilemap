use bevy::{
    math::{UVec3, Vec2},
    prelude::{Commands, Transform},
};

use crate::{
    map::{HexType, IsoType, Tilemap2dSize, Tilemap2dTileSize, TilemapId, TilemapMeshType},
    tiles::{Tile2dStorage, TileBundle, TilePos2d, TileTexture},
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

pub fn fill_tilemap_rect(
    tile_texture: TileTexture,
    pos: TilePos2d,
    size: Tilemap2dSize,
    tilemap_id: TilemapId,
    commands: &mut Commands,
    tile_storage: &mut Tile2dStorage,
) {
    for x in pos.x..size.x {
        for y in pos.y..size.y {
            let tile_pos = TilePos2d { x, y };
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

pub fn project_iso_diamond(x: f32, y: f32, pixel_width: f32, pixel_height: f32) -> Vec2 {
    let new_x = (x - y) * pixel_width / 2.0;
    let new_y = (x + y) * pixel_height / 2.0;
    Vec2::new(new_x, -new_y)
}

pub fn project_iso_staggered(x: f32, y: f32, pixel_width: f32, pixel_height: f32) -> Vec2 {
    let new_x = x * pixel_width;
    let new_y = y * pixel_height;
    Vec2::new(new_x, new_y)
}
