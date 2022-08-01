use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_ecs_tilemap::prelude::*;

mod helpers;

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn_bundle(Camera2dBundle::default());

    let texture_handle = asset_server.load("tiles.png");

    // Create map with (10 * 128) ^ 2 tiles or 1,638,400 tiles.
    // Be patient when running this example as meshing does not run on multiple CPU's yet..
    let layer_entity = LayerBuilder::<TileBundle>::new_batch(
        &mut commands,
        LayerSettings::new(
            MapSize(10, 10),
            ChunkSize(128, 128),
            TileSize(16.0, 16.0),
            TextureSize(96.0, 16.0),
        ),
        &mut meshes,
        texture_handle,
        0u16,
        0u16,
        |_| Some(TileBundle::default()),
    );

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
        .insert(Transform::from_xyz(-10240.0, -10240.0, 0.0))
        .insert(GlobalTransform::default());
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Benchmark Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup)
        .add_system(helpers::camera::movement)
        .add_system(helpers::texture::set_texture_filters_to_nearest)
        .run();
}
