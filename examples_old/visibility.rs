use bevy::{core::Time, prelude::*};
use bevy_ecs_tilemap::prelude::*;
use rand::{thread_rng, Rng};

mod helpers;

#[derive(Default, Component)]
struct LastUpdate {
    value: f64,
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>, mut map_query: MapQuery) {
    commands.spawn_bundle(Camera2dBundle::default());

    let texture_handle = asset_server.load("tiles.png");

    // Create map entity and component:
    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);

    let layer_settings = LayerSettings::new(
        MapSize(4, 4),
        ChunkSize(8, 8),
        TileSize(16.0, 16.0),
        TextureSize(96.0, 16.0),
    );
    let center = layer_settings.get_pixel_center();

    let (mut layer_builder, layer_entity) =
        LayerBuilder::new(&mut commands, layer_settings, 0u16, 0u16);
    map.add_layer(&mut commands, 0u16, layer_entity);

    layer_builder.set_all(TileBundle {
        tile: Tile {
            texture_index: 1,
            ..Default::default()
        },
        ..Default::default()
    });

    map_query.build_layer(&mut commands, layer_builder, texture_handle);

    commands.entity(layer_entity).insert(LastUpdate::default());

    // Spawn Map
    // Required in order to use map_query to retrieve layers/tiles.
    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(-center.x, -center.y, 0.0))
        .insert(GlobalTransform::default());
}

fn remove_tiles(
    time: Res<Time>,
    mut last_update_query: Query<&mut LastUpdate>,
    mut tile_query: Query<&mut Tile>,
    mut map_query: MapQuery,
) {
    let current_time = time.seconds_since_startup();
    for mut last_update in last_update_query.iter_mut() {
        // Remove a tile every half second.
        if (current_time - last_update.value) > 0.1 {
            let mut random = thread_rng();
            let position = TilePos(random.gen_range(0..32), random.gen_range(0..32));

            // Instead of removing the tile entity we want to hide the tile by removing the Visible component.
            if let Ok(tile_entity) = map_query.get_tile_entity(position, 0u16, 0u16) {
                if let Ok(mut tile) = tile_query.get_mut(tile_entity) {
                    if tile.visible {
                        tile.visible = false;
                    } else {
                        tile.visible = true;
                    }
                }
            }

            map_query.notify_chunk_for_tile(position, 0u16, 0u16);

            last_update.value = current_time;
        }
    }
}

fn main() {
    // env_logger::Builder::from_default_env()
    //     .filter_module("bevy_ecs_tilemap", log::LevelFilter::Trace)
    //     .init();

    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Remove Tiles Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup)
        .add_system(helpers::camera::movement)
        .add_system(remove_tiles)
        .add_system(helpers::texture::set_texture_filters_to_nearest)
        .run();
}
