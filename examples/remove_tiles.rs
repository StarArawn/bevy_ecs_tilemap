use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;
use rand::{thread_rng, Rng};

mod helpers;

#[derive(Default)]
struct LastUpdate {
    value: f64,
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut map_query: MapQuery,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let texture_handle = asset_server.load("tiles.png");
    let material_handle = materials.add(ColorMaterial::texture(texture_handle));

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

    let (mut layer_builder, layer_entity) =
        LayerBuilder::new(&mut commands, layer_settings, 0u16, 0u16, None);
    map.add_layer(&mut commands, 0u16, layer_entity);

    layer_builder.set_all(TileBundle::default());

    map_query.build_layer(&mut commands, layer_builder, material_handle);

    commands.entity(layer_entity).insert(LastUpdate::default());

    // Spawn Map
    // Required in order to use map_query to retrieve layers/tiles.
    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(-center.x, -center.y, 0.0))
        .insert(GlobalTransform::default());
}

fn remove_tiles(
    mut commands: Commands,
    time: Res<Time>,
    mut last_update_query: Query<&mut LastUpdate>,
    mut map_query: MapQuery,
) {
    let current_time = time.seconds_since_startup();
    for mut last_update in last_update_query.iter_mut() {
        // Remove a tile every half second.
        if (current_time - last_update.value) > 0.5 {
            let mut random = thread_rng();
            let position = TilePos(random.gen_range(0..16), random.gen_range(0..16));
            let tile_entity = map_query.get_tile_entity(position, 0u16, 0u16);

            if tile_entity.is_ok() {
                let _ = map_query.despawn_tile(&mut commands, position, 0u16, 0u16);
            }

            map_query.notify_chunk_for_tile(position, 0u16, 0u16);

            last_update.value = current_time;
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
            title: String::from("Remove Tiles Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup.system())
        .add_system(helpers::camera::movement.system())
        .add_system(remove_tiles.system())
        .add_system(helpers::texture::set_texture_filters_to_nearest.system())
        .run();
}
