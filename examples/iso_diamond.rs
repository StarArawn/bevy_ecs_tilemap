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

    let texture_handle = asset_server.load("iso.png");
    let material_handle = materials.add(ColorMaterial::texture(texture_handle));

    let mut map_settings = LayerSettings::new(
        UVec2::new(4, 4),
        UVec2::new(32, 32),
        Vec2::new(64.0, 32.0),
        Vec2::new(640.0, 1024.0),
    );
    map_settings.mesh_type = TilemapMeshType::Isometric(IsoType::Diamond);

    // Layer 0
    let layer_0_entity = commands.spawn().id();
    let mut layer_0 = LayerBuilder::new(&mut commands, layer_0_entity, map_settings.clone());
    layer_0.set_all(TileBundle {
        tile: Tile {
            texture_index: 10,
            ..Default::default()
        },
        ..Default::default()
    }, true);

    map_query.create_layer(&mut commands, layer_0, material_handle.clone());


    // Make 2 layers on "top" of the base map.
    for z in 0..5 {
        let mut new_settings = map_settings.clone();
        new_settings.layer_id = z + 1;
        let layer_entity = commands.spawn().id();
        let mut layer_builder = LayerBuilder::new(&mut commands, layer_entity, new_settings.clone());
        
        let mut random = thread_rng();

        for _ in 0..1000 {
            let position = UVec2::new(random.gen_range(0..128), random.gen_range(0..128));
            // Ignore errors for demo sake.
            let _ = layer_builder.set_tile(
                position,
                TileBundle {
                    tile: Tile {
                        texture_index: 10 + z + 1,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                true,
            );
        }
        
        map_query.create_layer(&mut commands, layer_builder, material_handle.clone());
    }
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Info)
        .init();

    App::build()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Iso diamond Map"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup.system())
        .add_system(helpers::camera::movement.system())
        .add_system(helpers::texture::set_texture_filters_to_nearest.system())
        .run();
}
