use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod helpers;

fn fill_tilemap_rect_color(
    tile_texture: TileTexture,
    pos: TilePos2d,
    size: Tilemap2dSize,
    color: Color,
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
                    color: TileColor(color),
                    ..Default::default()
                })
                .id();
            tile_storage.set(&tile_pos, Some(tile_entity));
        }
    }
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());

    let texture_handle: Handle<Image> = asset_server.load("tiles.png");

    let tilemap_size = Tilemap2dSize { x: 128, y: 128 };
    let mut tile_storage = Tile2dStorage::empty(tilemap_size);
    let tilemap_entity = commands.spawn().id();
    let tilemap_id = TilemapId(tilemap_entity);

    fill_tilemap_rect_color(
        TileTexture(5),
        TilePos2d { x: 0, y: 0 },
        Tilemap2dSize { x: 128, y: 128 },
        Color::rgba(1.0, 0.0, 0.0, 1.0),
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    fill_tilemap_rect_color(
        TileTexture(5),
        TilePos2d { x: 64, y: 0 },
        Tilemap2dSize { x: 128, y: 64 },
        Color::rgba(1.0, 1.0, 0.0, 1.0),
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    fill_tilemap_rect_color(
        TileTexture(5),
        TilePos2d { x: 0, y: 64 },
        Tilemap2dSize { x: 64, y: 128 },
        Color::rgba(0.0, 1.0, 0.0, 1.0),
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    fill_tilemap_rect_color(
        TileTexture(5),
        TilePos2d { x: 64, y: 64 },
        Tilemap2dSize { x: 128, y: 128 },
        Color::rgba(0.0, 0.0, 1.0, 1.0),
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    let tile_size = Tilemap2dTileSize { x: 16.0, y: 16.0 };

    commands
        .entity(tilemap_entity)
        .insert_bundle(TilemapBundle {
            grid_size: Tilemap2dGridSize { x: 16.0, y: 16.0 },
            size: tilemap_size,
            storage: tile_storage,
            texture_size: Tilemap2dTextureSize { x: 96.0, y: 16.0 },
            texture: TilemapTexture(texture_handle),
            tile_size,
            mesh_type: TilemapMeshType::Square,
            ..Default::default()
        });
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Iso Diamond Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(Tilemap2dPlugin)
        .add_startup_system(startup)
        .add_system(helpers::camera::movement)
        .add_system(helpers::texture::set_texture_filters_to_nearest)
        .run();
}
