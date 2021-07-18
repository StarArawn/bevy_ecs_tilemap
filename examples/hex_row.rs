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

    let texture_handle = asset_server.load("pointy_hex_tiles.png");
    let material_handle = materials.add(ColorMaterial::texture(texture_handle));

    // Create map entity and component:
    let map_entity = commands.spawn().id();
    let mut map = Map::new(0u16, map_entity);

    let map_settings = LayerSettings::new_with_map_type(
        UVec2::new(2, 2),
        UVec2::new(64, 64),
        Vec2::new(15.0, 17.0),
        Vec2::new(105.0, 17.0),
        TilemapMeshType::Hexagon(HexType::Row)
    );

    let (mut layer_builder, layer_entity) =
        LayerBuilder::<TileBundle>::new(&mut commands, map_settings.clone(), 0u16, 0u16, None);
    map.add_layer(&mut commands, 0u16, layer_entity);

    layer_builder.fill(
        UVec2::new(0, 0),
        UVec2::new(64, 64),
        Tile {
            texture_index: 0,
            ..Default::default()
        }
        .into(),
    );
    layer_builder.fill(
        UVec2::new(64, 0),
        UVec2::new(128, 64),
        Tile {
            texture_index: 1,
            ..Default::default()
        }
        .into(),
    );
    layer_builder.fill(
        UVec2::new(0, 64),
        UVec2::new(64, 128),
        Tile {
            texture_index: 2,
            ..Default::default()
        }
        .into(),
    );
    layer_builder.fill(
        UVec2::new(64, 64),
        UVec2::new(128, 128),
        Tile {
            texture_index: 3,
            ..Default::default()
        }
        .into(),
    );

    map_query.build_layer(&mut commands, layer_builder, material_handle.clone());

    for z in 0..2 {
        let mut new_settings = map_settings.clone();
        new_settings.layer_id = z + 1;
        let (mut layer_builder, layer_entity) =
            LayerBuilder::<TileBundle>::new(&mut commands, new_settings, 0u16, 0u16, None);
        map.add_layer(&mut commands, z, layer_entity);

        let mut random = thread_rng();

        for _ in 0..100 {
            let position = UVec2::new(random.gen_range(0..128), random.gen_range(0..128));
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

        map_query.build_layer(&mut commands, layer_builder, material_handle.clone());
    }

    // Spawn Map
    // Required in order to use map_query to retrieve layers/tiles.
    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(-48.0, -24.0, 0.0))
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
            title: String::from("Hex Map Row Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup.system())
        .add_system(helpers::camera::movement.system())
        .add_system(helpers::texture::set_texture_filters_to_nearest.system())
        .run();
}
