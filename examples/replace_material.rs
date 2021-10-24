use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod helpers;

#[derive(Default)]
struct DidUpdate {
    value: bool,
}

struct Materials {
    a: Handle<ColorMaterial>,
    b: Handle<ColorMaterial>,
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut map_query: MapQuery,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let texture_handle_a = asset_server.load("tiles.png");
    let material_handle_a = materials.add(ColorMaterial::texture(texture_handle_a));
    let texture_handle_b = asset_server.load("tiles2.png");
    let material_handle_b = materials.add(ColorMaterial::texture(texture_handle_b));

    let materials = Materials {
        a: material_handle_a,
        b: material_handle_b,
    };

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

    map_query.build_layer(&mut commands, layer_builder, materials.a.clone());

    commands.entity(layer_entity).insert(DidUpdate::default());

    commands.insert_resource(materials);

    // Spawn Map
    // Required in order to use map_query to retrieve layers/tiles.
    commands
        .entity(map_entity)
        .insert(map)
        .insert(Transform::from_xyz(-center.x, -center.y, 0.0))
        .insert(GlobalTransform::default());
}

fn replace_material(
    time: Res<Time>,
    materials: Res<Materials>,
    mut did_update_query: Query<&mut DidUpdate>,
    map_query: MapQuery,
    mut material_query: Query<&mut Handle<ColorMaterial>>,
) {
    let current_time = time.seconds_since_startup();
    for mut did_update in did_update_query.iter_mut() {
        if did_update.value {
            return;
        }
        // Replace the material after two seconds.
        if current_time > 2.0 {
            let layer = map_query.get_layer(0u16, 0u16);
            if let Some((_, layer)) = layer {
                // Replace the material in the first chunk.
                let chunk = layer.get_chunk(ChunkPos::default());
                if let Some(e) = chunk {
                    // Get the chunk's ColorMaterial component.
                    let material = material_query.get_component_mut::<Handle<ColorMaterial>>(e);
                    if let Ok(mut m) = material {
                        // Mutate the component to the new material.
                        *m = materials.b.clone();
                    }
                }
            }

            did_update.value = true;
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
            title: String::from("Re Example"),
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(TilemapPlugin)
        .add_startup_system(startup.system())
        .add_system(helpers::camera::movement.system())
        .add_system(replace_material.system())
        .add_system(helpers::texture::set_texture_filters_to_nearest.system())
        .run();
}
