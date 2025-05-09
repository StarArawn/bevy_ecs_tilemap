use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
mod helpers;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2d);

    let texture_handle: Handle<Image> = asset_server.load("tiles.png");

    let map_size = TilemapSize { x: 2, y: 1 };

    // Create a tilemap entity a little early.
    let tilemap_entity = commands.spawn_empty().id();

    let mut tile_storage = TileStorage::empty(map_size);

    // Spawn the elements of the tilemap.

    let tile_pos = TilePos { x: 0, y: 0 };
    let tile_entity = commands
        .spawn((Tile, tile_pos, TilemapId(tilemap_entity)))
        .id();
    tile_storage.set(&tile_pos, tile_entity);

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = TilemapGridSize::from(tile_size);
    let map_type = TilemapType::default();

    commands.entity(tilemap_entity).insert((
        Tilemap,
        grid_size,
        map_type,
        map_size,
        tile_storage,
        TilemapTexture::Single(texture_handle),
        TilemapMaterial::standard(),
        tile_size,
        TilemapAnchor::Center,
    ));
}

fn swap_pos(keyboard_input: Res<ButtonInput<KeyCode>>, mut query: Query<&mut TilePos>) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for mut pos in query.iter_mut() {
            if pos.x == 0 {
                pos.x = 1;
            } else {
                pos.x = 0;
            }
        }
    }
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Update tile positions without despawning."),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(TilemapPlugin)
        .add_systems(Startup, startup)
        .add_systems(Update, helpers::camera::movement)
        .add_systems(Update, swap_pos)
        .run();
}
