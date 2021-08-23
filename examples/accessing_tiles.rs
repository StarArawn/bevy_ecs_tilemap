use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod helpers;

struct CurrentColor(u16);
struct LastUpdate(f64);

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut map_query: MapQuery,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let texture_handle = asset_server.load("tiles.png");
    let material_handle = materials.add(ColorMaterial::texture(texture_handle));

    // We can create maps by using the LayerBuilder
    // LayerBuilder creates the tile entities and makes sure they are attached to chunks correctly.
    // It also provides a way of accessing and viewing tiles during the creation phase.
    // Once a LayerBuilder is passed to the map_query.create_layer function it is consumed and
    // can no longer be accessed.
    // Layer builder accepts a generic bundle that must implement the `TileBundleTrait`.
    // This is used internally to access the tile in the bundle to attach the chunk correctly.

    // Create the layer builder
    let (mut layer_builder, layer_entity) = LayerBuilder::<TileBundle>::new(
        &mut commands,
        LayerSettings::new(
            UVec2::new(4, 4),
            UVec2::new(32, 32),
            Vec2::new(16.0, 16.0),
            Vec2::new(96.0, 256.0),
        ),
        0u16,
        0u16,
        None,
    );

    // We can easily fill the entire map by using set_all
    layer_builder.set_all(Tile::default().into());

    // You can also fill in a portion of the map
    layer_builder.fill(
        UVec2::new(0, 0),
        UVec2::new(10, 10),
        Tile {
            texture_index: 1,
            ..Default::default()
        }
        .into(),
    );

    let neighbors = layer_builder.get_tile_neighbors(UVec2::new(0, 0));

    // We can access tiles like normal using:
    assert!(layer_builder.get_tile(UVec2::new(0, 0)).is_ok());
    assert!(neighbors.len() == 8);
    assert!(neighbors.iter().filter(|n| n.1.is_some()).count() == 3); // Only 3 neighbors since negative is outside of map.

    let mut color = 0;
    for x in (2..128).step_by(4) {
        color += 1;
        for y in (2..128).step_by(4) {
            // Grabbing neighbors is easy.
            let neighbors: Vec<IVec2> = layer_builder
                .get_tile_neighbors(UVec2::new(x, y))
                .iter()
                .map(|(pos, _)| *pos)
                .collect();
            for pos in neighbors.iter() {
                // We can set specific tiles like this:
                let _ = layer_builder.set_tile(
                    UVec2::new(pos.x as u32, pos.y as u32),
                    Tile {
                        texture_index: color,
                        ..Default::default()
                    }
                    .into(),
                );
            }
        }
    }

    // Once create_layer is called you can no longer access the tiles in this system.
    map_query.build_layer(&mut commands, layer_builder, material_handle);

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
                    let neighbors = map_query.get_tile_neighbors(UVec2::new(x, y), 0u16, 0u16);
                    for (pos, neighbor) in neighbors.iter() {
                        // If the tile exists we will have an entity.
                        if let Some(neighbor) = neighbor {
                            // We query tiles using a query coming from this system.
                            // This has the add advantage of being able to query "extra" data per tile.
                            if let Ok(mut tile) = tile_query.get_mut(*neighbor) {
                                *tile = Tile {
                                    texture_index: color,
                                    ..Default::default()
                                };
                                // Finally after mutating the tile we can tell the internal systems to "remesh" the tilemap.
                                // This sends the new tile data to the gpu.
                                map_query.notify_chunk_for_tile(pos.as_u32(), 0u16, 0u16);
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
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Accessing tiles"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup.system())
        .add_system(update_map.system())
        .add_system(helpers::camera::movement.system())
        .add_system(helpers::texture::set_texture_filters_to_nearest.system())
        .run();
}
