use bevy::{prelude::*, reflect::TypeUuid, render::render_resource::AsBindGroup};
use bevy_ecs_tilemap::prelude::*;
mod helpers;

#[derive(AsBindGroup, TypeUuid, Debug, Clone, Default)]
#[uuid = "31575692-a956-4762-98e2-5d457f552d0a"]
pub struct MyMaterial {}

impl MaterialTilemap for MyMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "custom_shader.wgsl".into()
    }
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<MyMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    let my_material_handle = materials.add(MyMaterial {});

    let texture_handle: Handle<Image> = asset_server.load("tiles.png");

    let map_size = TilemapSize { x: 32, y: 32 };

    // Layer 1
    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();

    fill_tilemap(
        TileTextureIndex(0),
        map_size,
        TilemapId(tilemap_entity),
        &mut commands,
        &mut tile_storage,
    );

    let tile_size = TilemapTileSize { x: 16.0, y: 16.0 };
    let grid_size = tile_size.into();
    let map_type = TilemapType::default();

    commands
        .entity(tilemap_entity)
        .insert(MaterialTilemapBundle {
            grid_size,
            map_type,
            size: map_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(texture_handle.clone()),
            tile_size,
            transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
            material: my_material_handle.clone(),
            ..Default::default()
        });

    // Layer 2
    let mut tile_storage = TileStorage::empty(map_size);
    let tilemap_entity = commands.spawn_empty().id();

    fill_tilemap(
        TileTextureIndex(2),
        map_size,
        TilemapId(tilemap_entity),
        &mut commands,
        &mut tile_storage,
    );

    commands
        .entity(tilemap_entity)
        .insert(MaterialTilemapBundle {
            grid_size,
            map_type,
            size: map_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(texture_handle),
            tile_size: TilemapTileSize { x: 16.0, y: 16.0 },
            transform: get_tilemap_center_transform(&map_size, &grid_size, &map_type, 1.0)
                * Transform::from_xyz(32.0, 32.0, 0.0),
            material: my_material_handle.clone(),
            ..Default::default()
        });
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Custom Shader"),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugin(TilemapPlugin)
        .add_plugin(MaterialTilemapPlugin::<MyMaterial>::default())
        .add_startup_system(startup)
        .add_system(helpers::camera::movement)
        .run();
}
