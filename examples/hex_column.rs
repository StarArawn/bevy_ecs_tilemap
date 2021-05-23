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

    let texture_handle = asset_server.load("flat_hex_tiles.png");
    let material_handle = materials.add(ColorMaterial::texture(texture_handle));

    let layer_entity = commands.spawn().id();
    let mut map_settings = LayerSettings::new(
        UVec2::new(1, 1),
        UVec2::new(64, 64),
        Vec2::new(17.0, 15.0),
        Vec2::new(102.0, 15.0),
    );
    map_settings.mesh_type = TilemapMeshType::Hexagon(HexType::Column);

    let mut layer_builder =
        LayerBuilder::<TileBundle>::new(&mut commands, layer_entity, map_settings.clone());
    layer_builder.set_all(Tile::default().into());

    map_query.create_layer(&mut commands, layer_builder, material_handle.clone());

    for z in 0..2 {
        let mut new_settings = map_settings.clone();
        new_settings.layer_id = z + 1;
        let layer_entity = commands.spawn().id();
        let mut layer_builder =
            LayerBuilder::<TileBundle>::new(&mut commands, layer_entity, new_settings);

        let mut random = thread_rng();

        for _ in 0..100 {
            let position = UVec2::new(random.gen_range(0..64), random.gen_range(0..64));
            // Ignore errors for demo sake.
            let _ = layer_builder.set_tile(
                position,
                Tile {
                    texture_index: z + 1,
                    ..Default::default()
                }
                .into(),
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
            title: String::from("Hex Map Column Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup.system())
        .add_system(helpers::camera::movement.system())
        .add_system(helpers::texture::set_texture_filters_to_nearest.system())
        .run();
}
