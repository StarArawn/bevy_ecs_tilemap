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

    let layer_entity = commands.spawn().id();
    let mut layer_builder = LayerBuilder::new(
        &mut commands,
        layer_entity,
        LayerSettings::new(
            UVec2::new(2, 2),
            UVec2::new(8, 8),
            Vec2::new(16.0, 16.0),
            Vec2::new(96.0, 256.0),
        ),
    );
    layer_builder.set_all(TileBundle::default());

    map_query.create_layer(&mut commands, layer_builder, material_handle);

    commands.entity(layer_entity).insert(LastUpdate::default());
}

fn remove_tiles(
    mut commands: Commands,
    time: Res<Time>,
    mut last_update_query: Query<&mut LastUpdate>,
    map_query: MapQuery,
) {
    let current_time = time.seconds_since_startup();
    for mut last_update in last_update_query.iter_mut() {
        // Remove a tile every half second.
        if (current_time - last_update.value) > 0.5 {
            let mut random = thread_rng();
            let position = UVec2::new(random.gen_range(0..16), random.gen_range(0..16));
            let tile_entity = map_query.get_tile_entity(position, 0u32);

            // Note you can also call map.remove_tile() instead.
            if tile_entity.is_ok() {
                commands.entity(tile_entity.unwrap()).insert(RemoveTile);
            }

            last_update.value = current_time;
        }
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
