use crate::helpers::hex_grid::axial::AxialPos;
use crate::helpers::hex_grid::neighbors::{HexDirection, HEX_DIRECTIONS};
use crate::map::TilemapId;
use crate::prelude::HexCoordSystem;
use crate::tiles::{TileBundle, TileColor, TilePos, TileTextureIndex};
use crate::{TileStorage, TilemapSize};
use bevy::hierarchy::BuildChildren;
use bevy::prelude::{ChildBuild, Color, Commands};

/// Fills an entire tile storage with the given tile.
pub fn fill_tilemap(
    texture_index: TileTextureIndex,
    size: TilemapSize,
    tilemap_id: TilemapId,
    commands: &mut Commands,
    tile_storage: &mut TileStorage,
) {
    commands.entity(tilemap_id.0).with_children(|parent| {
        for x in 0..size.x {
            for y in 0..size.y {
                let tile_pos = TilePos { x, y };
                let tile_entity = parent
                    .spawn(TileBundle {
                        position: tile_pos,
                        tilemap_id,
                        texture_index,
                        ..Default::default()
                    })
                    .id();
                tile_storage.set(&tile_pos, tile_entity);
            }
        }
    });
}

/// Fills a rectangular region with the given tile.
///
/// The rectangular region is defined by an `origin` in [`TilePos`], and a
/// `size` in tiles ([`TilemapSize`]).  
pub fn fill_tilemap_rect(
    texture_index: TileTextureIndex,
    origin: TilePos,
    size: TilemapSize,
    tilemap_id: TilemapId,
    commands: &mut Commands,
    tile_storage: &mut TileStorage,
) {
    commands.entity(tilemap_id.0).with_children(|parent| {
        for x in 0..size.x {
            for y in 0..size.y {
                let tile_pos = TilePos {
                    x: origin.x + x,
                    y: origin.y + y,
                };

                let tile_entity = parent
                    .spawn(TileBundle {
                        position: tile_pos,
                        tilemap_id,
                        texture_index,
                        ..Default::default()
                    })
                    .id();
                tile_storage.set(&tile_pos, tile_entity);
            }
        }
    });
}

/// Fills a rectangular region with colored versions of the given tile.
///
/// The rectangular region is defined by an `origin` in [`TilePos`], and a
/// `size` in tiles ([`TilemapSize`]).   
pub fn fill_tilemap_rect_color(
    texture_index: TileTextureIndex,
    origin: TilePos,
    size: TilemapSize,
    color: Color,
    tilemap_id: TilemapId,
    commands: &mut Commands,
    tile_storage: &mut TileStorage,
) {
    commands.entity(tilemap_id.0).with_children(|parent| {
        for x in 0..size.x {
            for y in 0..size.y {
                let tile_pos = TilePos {
                    x: origin.x + x,
                    y: origin.y + y,
                };

                let tile_entity = parent
                    .spawn(TileBundle {
                        position: tile_pos,
                        tilemap_id,
                        texture_index,
                        color: TileColor(color),
                        ..Default::default()
                    })
                    .id();
                tile_storage.set(&tile_pos, tile_entity);
            }
        }
    });
}

/// Generates a vector of hex positions that form a ring of given `radius` around the specified
/// `origin`.
///
/// If `radius` is zero, `origin` is the only position in the returned vector.
pub fn generate_hex_ring(origin: AxialPos, radius: u32) -> Vec<AxialPos> {
    if radius == 0 {
        vec![origin]
    } else {
        let mut ring = Vec::with_capacity((radius * 6) as usize);
        let corners = HEX_DIRECTIONS
            .iter()
            .map(|direction| origin + radius * AxialPos::from(direction))
            .collect::<Vec<AxialPos>>();
        // The "tangent" is the direction we must travel in to reach the next corner
        let tangents = (0..6)
            .map(|ix| HexDirection::from(ix + 2).into())
            .collect::<Vec<AxialPos>>();

        for (&corner, &tangent) in corners.iter().zip(tangents.iter()) {
            for k in 0..radius {
                ring.push(corner + k * tangent);
            }
        }

        ring
    }
}

/// Generates a vector of hex positions that form a hexagon of given `radius` around the specified
/// `origin`.
pub fn generate_hexagon(origin: AxialPos, radius: u32) -> Vec<AxialPos> {
    let mut hexagon = Vec::with_capacity(1 + (6 * radius * (radius + 1) / 2) as usize);
    for r in 0..(radius + 1) {
        hexagon.extend(generate_hex_ring(origin, r));
    }
    hexagon
}

/// Fills a hexagonal region with the given `tile_texture`.
///
/// The rectangular region is defined by an `origin` in [`TilePos`], and a
/// `radius`.
///
/// Tiles that do not fit in the tilemap will not be created.
pub fn fill_tilemap_hexagon(
    texture_index: TileTextureIndex,
    origin: TilePos,
    radius: u32,
    hex_coord_system: HexCoordSystem,
    tilemap_id: TilemapId,
    commands: &mut Commands,
    tile_storage: &mut TileStorage,
) {
    let tile_positions = generate_hexagon(
        AxialPos::from_tile_pos_given_coord_system(&origin, hex_coord_system),
        radius,
    )
    .into_iter()
    .map(|axial_pos| axial_pos.as_tile_pos_given_coord_system(hex_coord_system))
    .collect::<Vec<TilePos>>();

    commands.entity(tilemap_id.0).with_children(|parent| {
        for tile_pos in tile_positions {
            let tile_entity = parent
                .spawn(TileBundle {
                    position: tile_pos,
                    tilemap_id,
                    texture_index,
                    ..Default::default()
                })
                .id();
            tile_storage.checked_set(&tile_pos, tile_entity)
        }
    });
}
