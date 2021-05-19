use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    prelude::*,
};
use bevy_ecs_tilemap::prelude::*;

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

    let map_size = UVec2::new(5 * 16, 5 * 16);

    let layer_entity = commands.spawn().id();
    let mut layer_builder = LayerBuilder::<TileBundle>::new(
        &mut commands,
        layer_entity,
        LayerSettings::new(
            UVec2::new(5, 5),
            UVec2::new(16, 16),
            Vec2::new(16.0, 16.0),
            Vec2::new(96.0, 256.0),
        )
    );
    
    let mut i = 0;
    for x in 0..map_size.x {
        for y in 0..map_size.y {
            let position = UVec2::new(x, y);
            // Ignore errors for demo sake.
            let _ = layer_builder.set_tile(
                position,
                Tile {
                    texture_index: 0,
                    ..Default::default()
                }.into(),
                i % 2 == 0 || i % 7 == 0,
            );
            i += 1;
        }
    }

    map_query.create_layer(&mut commands, layer_builder, material_handle.clone());

    commands
        .entity(layer_entity)
        .insert(LastUpdate(0.0));
}

pub struct LastUpdate(f64);

fn update(
    mut commands: Commands,
    time: Res<Time>,
    mut last_update_query: Query<&mut LastUpdate>,
    visible: Query<&bevy_ecs_tilemap::prelude::VisibleTile>,
    tile_query: Query<(Entity, &UVec2), With<Tile>>,
    mut map_query: MapQuery,
) {
    let current_time = time.seconds_since_startup();
    if let Ok(mut last_update) = last_update_query.single_mut() {
        if current_time - last_update.0 > 0.1 {
            for (entity, pos) in tile_query.iter() {
                // Get neighbor count.
                let neighbor_count = map_query
                    .get_tile_neighbors(*pos, 0u32)
                    .iter()
                    .filter(|x| {
                        if let Some(entity) = x.1 {
                            return visible.get(entity).is_ok();
                        }
                        return false;
                    })
                    .count();
                let was_alive = visible.get(entity).is_ok();

                let is_alive = match (was_alive, neighbor_count) {
                    (true, x) if x < 2 => false,
                    (true, 2) | (true, 3) => true,
                    (true, x) if x > 3 => false,
                    (false, 3) => true,
                    (otherwise, _) => otherwise,
                };

                if is_alive && !was_alive {
                    commands
                        .entity(entity)
                        .insert(bevy_ecs_tilemap::prelude::VisibleTile);
                    map_query.notify_chunk_for_tile(*pos, 0u32);
                } else if !is_alive && was_alive {
                    commands
                        .entity(entity)
                        .remove::<bevy_ecs_tilemap::prelude::VisibleTile>();
                    map_query.notify_chunk_for_tile(*pos, 0u32);
                }
            }

            last_update.0 = current_time;
        }
    }
}

fn main() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Error)
        .init();

    App::build()
        .insert_resource(WindowDescriptor {
            width: 1270.0,
            height: 720.0,
            title: String::from("Game Of Life"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup.system())
        .add_system(helpers::camera::movement.system())
        .add_system(helpers::texture::set_texture_filters_to_nearest.system())
        .add_system(update.system())
        .run();
}
