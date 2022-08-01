use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod helpers;

#[derive(Component)]
struct CurrentColor(u16);

#[derive(Component)]
struct LastUpdate(f64);

fn startup(mut commands: Commands, asset_server: Res<AssetServer>, mut map_query: MapQuery) {
    commands.spawn_bundle(Camera2dBundle::default());

    let texture_handle = asset_server.load("tiles.png");

    // We can create maps by using the LayerBuilder
    // LayerBuilder creates the tile entities and makes sure they are attached to chunks correctly.
    // It also provides a way of accessing and viewing tiles during the creation phase.
    // Once a LayerBuilder is passed to the map_query.build_layer function it is consumed and
    // can no longer be accessed.
    // Layer builder accepts a generic bundle that must implement the `TileBundleTrait`.
    // This is used internally to access the tile in the bundle to attach the chunk correctly.

    // Create the layer builder
    let (mut layer_builder, layer_entity) = LayerBuilder::<TileBundle>::new(
        &mut commands,
        LayerSettings::new(
            MapSize(4, 4),
            ChunkSize(32, 32),
            TileSize(16.0, 16.0),
            TextureSize(96.0, 16.0),
        ),
        0u16,
        0u16,
    );

    // We can easily fill the entire map by using set_all
    layer_builder.set_all(Tile::default().into());

    // You can also fill in a portion of the map
    layer_builder.fill(
        TilePos(0, 0),
        TilePos(10, 10),
        Tile {
            texture_index: 1,
            ..Default::default()
        }
        .into(),
    );

    let neighbors = layer_builder.get_tile_neighbors(TilePos(0, 0));

    // We can access tiles like normal using:
    assert!(layer_builder.get_tile(TilePos(0, 0)).is_ok());
    assert!(neighbors.len() == 8);
    let neighbor_count = neighbors.iter().filter(|n| n.is_some()).count();
    assert!(neighbor_count == 3); // Only 3 neighbors since negative is outside of map.

    let mut color = 0;
    for x in (2..128).step_by(4) {
        color += 1;
        for y in (2..128).step_by(4) {
            // Grabbing neighbors is easy.
            let neighbors = get_neighboring_pos(TilePos(x, y));
            for &pos in neighbors.iter() {
                // We can set specific tiles like this:
                let _ = layer_builder.set_tile(
                    pos.expect("Tile position does not exist."),
                    Tile {
                        texture_index: color,
                        ..Default::default()
                    }
                    .into(),
                );
            }
        }
    }

    // Once build_layer is called you can no longer access the tiles in this system.
    map_query.build_layer(&mut commands, layer_builder, texture_handle);

    commands
        .entity(layer_entity)
        .insert(CurrentColor(1))
        .insert(LastUpdate(0.0));

    // Create map entity and component:
    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);

    // Required to keep track of layers for a map internally.
    map.add_layer(&mut commands, 0u16, layer_entity);

    // Spawn Map
    // Required in order to use map_query to retrieve layers/tiles.
    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(-1024.0, -1024.0, 0.0))
        .insert(GlobalTransform::default());
}

// Should run after the commands from startup have been processed.
// An example of how to manipulate tiles.
fn update_map(
    time: Res<Time>,
    mut extra_data_query: Query<(&mut CurrentColor, &mut LastUpdate)>,
    mut tile_query: Query<&mut Tile>,
    mut map_query: MapQuery,
) {
    let current_time = time.seconds_since_startup();
    for (mut current_color, mut last_update) in extra_data_query.iter_mut() {
        if (current_time - last_update.0) > 0.1 {
            current_color.0 += 1;
            if current_color.0 > 5 {
                current_color.0 = 1;
            }
            let mut color = current_color.0;
            for x in (2..128).step_by(4) {
                for y in (2..128).step_by(4) {
                    // First we get the neighboring entities for the given tile.
                    let neighboring_tile_pos = get_neighboring_pos(TilePos(x, y));
                    let neighboring_entities =
                        map_query.get_tile_neighbors(TilePos(x, y), 0u16, 0u16);

                    // Iterating over each neighbor
                    for i in 0..8 {
                        if neighboring_entities[i].is_ok() {
                            // We query tiles using a query coming from this system.
                            // This has the add advantage of being able to query "extra" data per tile.
                            if let Ok(mut tile) =
                                tile_query.get_mut(neighboring_entities[i].unwrap())
                            {
                                *tile = Tile {
                                    texture_index: color,
                                    ..Default::default()
                                };
                                // Finally after mutating the tile we can tell the internal systems to "remesh" the tilemap.
                                // This sends the new tile data to the gpu.
                                map_query.notify_chunk_for_tile(
                                    neighboring_tile_pos[i].unwrap(),
                                    0u16,
                                    0u16,
                                );
                            }
                        }
                    }
                }
                color += 1;
                if color > 5 {
                    color = 1;
                }
            }

            last_update.0 = current_time;
        }
    }
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Accessing tiles"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup)
        .add_system(update_map)
        .add_system(helpers::camera::movement)
        .add_system(helpers::texture::set_texture_filters_to_nearest)
        .run();
}
