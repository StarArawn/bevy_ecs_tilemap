use crate::map::TilemapId;
use crate::tiles::{TileBundle, TileColor, TilePos, TileTexture};
use crate::{TileStorage, TilemapSize};
use bevy::hierarchy::BuildChildren;
use bevy::prelude::{Color, Commands};

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
                    tilemap_id,
                    texture: tile_texture,
                    ..Default::default()
                })
                .id();
            commands.entity(tilemap_id.0).add_child(tile_entity);
            tile_storage.set(&tile_pos, Some(tile_entity));
        }
    }
}

/// Fills a rectangular region with the given tile.
///
/// The rectangular region is defined by an `origin` in [`TilePos`](crate::tiles::TilePos), and a
/// `size` in tiles ([`TilemapSize`](crate::map::TilemapSize)).  
pub fn fill_tilemap_rect(
    tile_texture: TileTexture,
    origin: TilePos,
    size: TilemapSize,
    tilemap_id: TilemapId,
    commands: &mut Commands,
    tile_storage: &mut TileStorage,
) {
    for x in 0..size.x {
        for y in 0..size.y {
            let tile_pos = TilePos {
                x: origin.x + x,
                y: origin.y + y,
            };

            let tile_entity = commands
                .spawn()
                .insert_bundle(TileBundle {
                    position: tile_pos,
                    tilemap_id,
                    texture: tile_texture,
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, Some(tile_entity));
        }
    }
}

/// Fills a rectangular region with colored versions of the given tile.
///
/// The rectangular region is defined by an `origin` in [`TilePos`](crate::tiles::TilePos), and a
/// `size` in tiles ([`TilemapSize`](crate::map::TilemapSize)).   
pub fn fill_tilemap_rect_color(
    tile_texture: TileTexture,
    origin: TilePos,
    size: TilemapSize,
    color: Color,
    tilemap_id: TilemapId,
    commands: &mut Commands,
    tile_storage: &mut TileStorage,
) {
    for x in 0..size.x {
        for y in 0..size.y {
            let tile_pos = TilePos {
                x: origin.x + x,
                y: origin.y + y,
            };

            let tile_entity = commands
                .spawn()
                .insert_bundle(TileBundle {
                    position: tile_pos,
                    tilemap_id,
                    texture: tile_texture,
                    color: TileColor(color),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, Some(tile_entity));
        }
    }
}
