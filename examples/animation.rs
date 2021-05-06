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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle {
        transform: Transform::from_xyz(1042.0, 1024.0, 1000.0 - 0.1),
        ..OrthographicCameraBundle::new_2d()
    });

    let texture_handle = asset_server.load("tiles.png");
    let material_handle = materials.add(ColorMaterial::texture(texture_handle));

    let map_size = UVec2::new(40, 40);

    let map_settings = MapSettings::new(
        map_size,
        UVec2::new(32, 32),
        Vec2::new(16.0, 16.0),
        Vec2::new(96.0, 256.0),
        0,
    );

    let mut map = Map::new(map_settings.clone());
    let map_entity = commands.spawn().id();
    map.build(
        &mut commands,
        &mut meshes,
        material_handle,
        map_entity,
        true,
    );
    commands.entity(map_entity).insert_bundle(MapBundle {
        map,
        ..Default::default()
    });

    let texture_handle = asset_server.load("flower_sheet.png");
    let material_handle = materials.add(ColorMaterial::texture(texture_handle));

    let map_size = map_size / 2;
    let map_settings = MapSettings::new(
        map_size,
        UVec2::new(32, 32),
        Vec2::new(32.0, 32.0),
        Vec2::new(32.0, 448.0),
        1,
    );
    let mut map = Map::new(map_settings);
    let map_entity = commands.spawn().id();
    map.build(
        &mut commands,
        &mut meshes,
        material_handle,
        map_entity,
        false,
    );

    let mut random = thread_rng();

    for _ in 0..750000 {
        let position = UVec2::new(
            random.gen_range(0..map_size.x * 32),
            random.gen_range(0..map_size.y * 32),
        );
        let entity = map.add_tile(
            &mut commands,
            position,
            Tile {
                texture_index: 0,
                ..Default::default()
            },
            true,
        );

        if let Ok(entity) = entity {
            commands
                .entity(entity)
                .insert(GPUAnimated::new(0, 13, 0.95));
        }
    }
    commands.entity(map_entity).insert_bundle(MapBundle {
        map,
        transform: Transform::from_xyz(0.0, 0.0, 1.0),
        ..Default::default()
    });
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
