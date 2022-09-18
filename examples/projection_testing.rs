use bevy::app::App;
use bevy::log::LogPlugin;
use bevy::math::Vec2;
use bevy_ecs_tilemap::map::{IsoCoordSystem, TilemapSize, TilemapTileSize, TilemapType};
use bevy_ecs_tilemap::prelude::TilemapGridSize;
use bevy_ecs_tilemap::tiles::TilePos;

fn startup() {
    let tile_size = TilemapTileSize { x: 100.0, y: 50.0 };
    let grid_size: TilemapGridSize = tile_size.into();
    let world_pos = Vec2::new(50.0, -25.0);
    let map_type = TilemapType::Isometric {
        coord_system: IsoCoordSystem::Diamond,
        diagonal_neighbors: false,
    };
    let map_size = TilemapSize { x: 1, y: 1 };
    TilePos::from_world_pos(&world_pos, &map_size, &grid_size, &map_type);
}

fn main() {
    App::new()
        .add_plugin(LogPlugin)
        .add_startup_system(startup)
        .run();
}
