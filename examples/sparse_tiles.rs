use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::{thread_rng, Rng};

mod helpers;

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut map_query: MapQuery,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let texture_handle = asset_server.load("tiles.png");
    let material_handle = materials.add(ColorMaterial::texture(texture_handle));

    let layer_settings = LayerSettings::new(
        UVec2::new(2, 2),
        UVec2::new(8, 8),
        Vec2::new(16.0, 16.0),
        Vec2::new(96.0, 256.0),
    );

    let mut ground_layer = None;

    commands
        .spawn()
        .insert(Transform::from_xyz(-128.0, -128.0, 0.0))
        .insert(GlobalTransform::default())
        .with_children(|child_builder| {
            ground_layer = Some(child_builder.spawn().id());
        });

    let mut layer_builder = LayerBuilder::new(&mut commands, ground_layer.unwrap(), layer_settings);

    let mut random = thread_rng();

    for _ in 0..100 {
        let position = UVec2::new(random.gen_range(0..16), random.gen_range(0..16));
        // Ignore errors for demo sake.
        let _ = layer_builder.set_tile(position, TileBundle::default(), true);
    }

    map_query.create_layer(&mut commands, layer_builder, material_handle);
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Warn)
        .init();

    App::build()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Sparse Tiles Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup.system())
        .add_system(helpers::camera::movement.system())
        .add_system(helpers::texture::set_texture_filters_to_nearest.system())
        .run();
}
