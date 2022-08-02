use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod helpers;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn_bundle(Camera2dBundle::default());

    let texture_handle: Handle<Image> = asset_server.load("iso_color.png");

    let tilemap_size = Tilemap2dSize { x: 128, y: 128 };
    let mut tile_storage = Tile2dStorage::empty(tilemap_size);
    let tilemap_entity = commands.spawn().id();
    let tilemap_id = TilemapId(tilemap_entity);

    bevy_ecs_tilemap::helpers::fill_tilemap_rect(
        TileTexture(0),
        TilePos2d { x: 0, y: 0 },
        Tilemap2dSize { x: 128, y: 128 },
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    bevy_ecs_tilemap::helpers::fill_tilemap_rect(
        TileTexture(1),
        TilePos2d { x: 64, y: 0 },
        Tilemap2dSize { x: 128, y: 64 },
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    bevy_ecs_tilemap::helpers::fill_tilemap_rect(
        TileTexture(2),
        TilePos2d { x: 0, y: 64 },
        Tilemap2dSize { x: 64, y: 128 },
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    bevy_ecs_tilemap::helpers::fill_tilemap_rect(
        TileTexture(3),
        TilePos2d { x: 64, y: 64 },
        Tilemap2dSize { x: 128, y: 128 },
        tilemap_id,
        &mut commands,
        &mut tile_storage,
    );

    let tile_size = Tilemap2dTileSize { x: 64.0, y: 32.0 };

    commands
        .entity(tilemap_entity)
        .insert_bundle(TilemapBundle {
            grid_size: Tilemap2dGridSize { x: 64.0, y: 32.0 },
            size: tilemap_size,
            storage: tile_storage,
            texture_size: Tilemap2dTextureSize { x: 384.0, y: 32.0 },
            texture: TilemapTexture(texture_handle),
            tile_size,
            mesh_type: TilemapMeshType::Isometric(IsoType::Staggered),
            ..Default::default()
        });
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Iso Staggered Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(Tilemap2dPlugin)
        .add_startup_system(startup)
        .add_system(helpers::camera::movement)
        .add_system(helpers::texture::set_texture_filters_to_nearest)
        .run();
}
