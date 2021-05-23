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
            UVec2::new(4, 4),
            UVec2::new(8, 8),
            Vec2::new(16.0, 16.0),
            Vec2::new(96.0, 256.0),
        ),
    );
    layer_builder.set_all(
        TileBundle {
            tile: Tile {
                texture_index: 1,
                ..Default::default()
            },
            ..Default::default()
        },
    );

    map_query.create_layer(&mut commands, layer_builder, material_handle);

    commands.entity(layer_entity).insert(LastUpdate::default());
}

fn remove_tiles(
    mut commands: Commands,
    time: Res<Time>,
    mut last_update_query: Query<&mut LastUpdate>,
    mut tile_query: Query<&mut Tile>,
    mut map_query: MapQuery,
) {
    let current_time = time.seconds_since_startup();
    for mut last_update in last_update_query.iter_mut() {
        // Remove a tile every half second.
        if (current_time - last_update.value) > 0.1 {
            let mut random = thread_rng();
            let position = UVec2::new(random.gen_range(0..32), random.gen_range(0..32));

            // Instead of removing the tile entity we want to hide the tile by removing the Visible component.
            if let Ok(tile_entity) = map_query.get_tile_entity(position, 0u32) {
                if let Ok(mut tile) = tile_query.get_mut(tile_entity) {
                    if tile.visible {
                        tile.visible = false;
                    } else {
                        tile.visible = true;
                    }
                }
            }

            map_query.notify_chunk_for_tile(position, 0u32);

            last_update.value = current_time;
        }
    }
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Trace)
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
