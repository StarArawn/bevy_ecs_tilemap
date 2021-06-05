use bevy::prelude::*;
use bevy_ecs_tilemap::prelude::*;

mod helpers;

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
) {
    let mut camera_bundle = OrthographicCameraBundle::new_2d();
    camera_bundle.orthographic_projection.scale = 0.5;
    commands.spawn_bundle(camera_bundle);

    let handle: Handle<TiledMap> = asset_server.load("iso_map.tmx");

    let map_entity = commands.spawn().id();

    commands.entity(map_entity)
        .insert_bundle(TiledMapBundle {
            tiled_map: handle,
            map: Map::new(0u16, map_entity),
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            ..Default::default()
        });

    let x_pos = 8.0;
    let y_pos = 8.0;
    // TODO: Replace this with like a "get_z_map_position" or something.
    let center = project_iso(Vec2::new(x_pos, y_pos), 64.0, 32.0);
    dbg!(center);
    let sprite_pos = Transform::from_xyz(center.x, center.y, 1.0 + (1.0 - (center.y / 10000.0)));
    dbg!(sprite_pos);
    let texture_handle: Handle<Texture> = asset_server.load("player.png");
    let material_handle = color_materials.add(ColorMaterial::texture(texture_handle));

    commands.spawn_bundle(SpriteBundle {
        material: material_handle,
        transform: sprite_pos,
        ..Default::default()
    }).insert(helpers::movement::Player);
}

fn project_iso(pos: Vec2, tile_width: f32, tile_height: f32) -> Vec2 {
    let x = (pos.x - pos.y) * tile_width / 2.0;
    let y = (pos.x + pos.y) * tile_height / 2.0;
    return Vec2::new(x, -y);
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
        .add_plugin(TiledMapPlugin)
        .add_startup_system(startup.system())
        .add_system(helpers::camera::movement.system())
        .add_system(helpers::movement::update.system())
        .add_system(helpers::texture::set_texture_filters_to_nearest.system())
        .run();
}
