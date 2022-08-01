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

    let layer_settings = LayerSettings::new(
        MapSize(2, 2),
        ChunkSize(8, 8),
        TileSize(16.0, 16.0),
        TextureSize(96.0, 16.0),
    );

    let center = layer_settings.get_pixel_center();

    let (mut layer_builder, ground_layer) =
        LayerBuilder::new(&mut commands, layer_settings, 0u16, 0u16);
    map.add_layer(&mut commands, 0u16, ground_layer);

    let mut random = thread_rng();

    for _ in 0..100 {
        let position = TilePos(random.gen_range(0..16), random.gen_range(0..16));
        // Ignore errors for demo sake.
        let _ = layer_builder.set_tile(position, TileBundle::default());
    }

    map_query.build_layer(&mut commands, layer_builder, texture_handle);

    // Spawn Map
    // Required in order to use map_query to retrieve layers/tiles.
    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(-center.x, -center.y, 0.0))
        .insert(GlobalTransform::default());
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Sparse Tiles Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup)
        .add_system(helpers::camera::movement)
        .add_system(helpers::texture::set_texture_filters_to_nearest)
        .run();
}
