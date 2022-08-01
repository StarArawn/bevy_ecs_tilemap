use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::{thread_rng, Rng};

mod helpers;

fn startup(mut commands: Commands, asset_server: Res<AssetServer>, mut map_query: MapQuery) {
    commands.spawn_bundle(Camera2dBundle::default());

    let texture_handle = asset_server.load("tiles.png");

    // Create map entity and component:
    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);

    let map_settings = LayerSettings::new(
        MapSize(2, 2),
        ChunkSize(8, 8),
        TileSize(16.0, 16.0),
        TextureSize(96.0, 16.0),
    );

    // Layer 0
    let (mut layer_0, layer_0_entity) =
        LayerBuilder::new(&mut commands, map_settings.clone(), 0u16, 0u16);

    // Required to keep track of layers for a map internally.
    map.add_layer(&mut commands, 0u16, layer_0_entity);

    layer_0.set_all(TileBundle::default());

    map_query.build_layer(&mut commands, layer_0, texture_handle.clone());

    // Make 2 layers on "top" of the base map.
    for z in 0..2 {
        let (mut layer_builder, layer_entity) =
            LayerBuilder::new(&mut commands, map_settings.clone(), 0u16, z + 1);

        let mut random = thread_rng();

        for _ in 0..100 {
            let position = TilePos(random.gen_range(0..16), random.gen_range(0..16));
            // Ignore errors for demo sake.
            let _ = layer_builder.set_tile(
                position,
                TileBundle {
                    tile: Tile {
                        texture_index: z + 1,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            );
        }

        map_query.build_layer(&mut commands, layer_builder, texture_handle.clone());

        // Required to keep track of layers for a map internally.
        map.add_layer(&mut commands, z + 1, layer_entity);
    }

    // Spawn Map
    // Required in order to use map_query to retrieve layers/tiles.
    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(-128.0, -128.0, 0.0))
        .insert(GlobalTransform::default());
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Layed Map Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup)
        .add_system(helpers::camera::movement)
        .add_system(helpers::texture::set_texture_filters_to_nearest)
        .run();
}
