use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_ecs_tilemap::prelude::*;
use rand::{thread_rng, Rng};

mod helpers;

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut map_query: MapQuery,
) {
    commands.spawn_bundle(OrthographicCameraBundle {
        transform: Transform::from_xyz(1042.0, 1024.0, 1000.0 - 0.1),
        ..OrthographicCameraBundle::new_2d()
    });

    let texture_handle = asset_server.load("tiles.png");
    let material_handle = materials.add(ColorMaterial::texture(texture_handle));

    let map_size = UVec2::new(20, 20);

    let layer_settings = LayerSettings::new(
        map_size,
        UVec2::new(32, 32),
        Vec2::new(16.0, 16.0),
        Vec2::new(96.0, 256.0),
    );

    let (mut layer_builder, layer_0_entity) =
        LayerBuilder::<TileBundle>::new(&mut commands, layer_settings.clone(), 0u16, 0u16, None);

    layer_builder.set_all(Tile::default().into());

    map_query.build_layer(&mut commands, layer_builder, material_handle);

    let texture_handle = asset_server.load("flower_sheet.png");
    let material_handle = materials.add(ColorMaterial::texture(texture_handle));

    let map_size = map_size / 2;
    let layer_settings = LayerSettings::new(
        map_size,
        UVec2::new(32, 32),
        Vec2::new(32.0, 32.0),
        Vec2::new(32.0, 448.0),
    );
    let (mut layer_builder, layer_1_entity) =
        LayerBuilder::<TileBundle>::new(&mut commands, layer_settings.clone(), 0u16, 1u16, None);

    let mut random = thread_rng();

    for _ in 0..10000 {
        let position = UVec2::new(
            random.gen_range(0..map_size.x * 32),
            random.gen_range(0..map_size.y * 32),
        );
        let _ = layer_builder.set_tile(
            position,
            Tile {
                texture_index: 0,
                animated: Some(GPUAnimated::new(0, 13, 0.95)),
                ..Default::default()
            }
            .into(),
        );
    }

    map_query.build_layer(&mut commands, layer_builder, material_handle);

    // Create map entity and component:
    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);

    // Required to keep track of layers for a map internally.
    map.add_layer(&mut commands, 0u16, layer_0_entity);
    map.add_layer(&mut commands, 1u16, layer_1_entity);

    // Spawn Map
    // Required in order to use map_query to retrieve layers/tiles.
    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(-5120.0, -5120.0, 0.0))
        .insert(GlobalTransform::default());
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    App::build()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Animated Map Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup.system())
        .add_system(helpers::camera::movement.system())
        .add_system(helpers::texture::set_texture_filters_to_nearest.system())
        .run();
}
