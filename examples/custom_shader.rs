use bevy::{prelude::*, reflect::TypePath, render::render_resource::AsBindGroup};
use bevy_ecs_tilemap::{prelude::*, MaterialTilemap};
mod helpers;

#[derive(AsBindGroup, TypePath, Debug, Clone, Default, Asset)]
pub struct MyMaterial {
    #[uniform(0)]
    brightness: f32,
    // webgl2 requires 16 byte alignment
    #[uniform(0)]
    _padding: Vec3,
}

impl TilemapMaterial for MyMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "custom_shader.wgsl".into()
    }
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<MyMaterial>>,
) {
    commands.spawn(Camera2d);

    let my_material_handle = TilemapMaterialHandle(materials.add(MyMaterial {
        brightness: 0.5,
        ..default()
    }));

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

    commands.entity(tilemap_entity).insert((
        // TODO: is this actually what we want here?
        MaterialTilemap::<MyMaterial>(std::marker::PhantomData),
        grid_size,
        map_type,
        map_size,
        tile_storage,
        TilemapTexture::Single(texture_handle.clone()),
        tile_size,
        get_tilemap_center_transform(&map_size, &grid_size, &map_type, 0.0),
        my_material_handle.clone(),
    ));

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

    commands.entity(tilemap_entity).insert((
        MaterialTilemap::<MyMaterial>(std::marker::PhantomData),
        grid_size,
        map_type,
        map_size,
        tile_storage,
        TilemapTexture::Single(texture_handle),
        TilemapTileSize { x: 16.0, y: 16.0 },
        get_tilemap_center_transform(&map_size, &grid_size, &map_type, 1.0)
            * Transform::from_xyz(32.0, 32.0, 0.0),
        my_material_handle,
    ));
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
        .add_plugins(TilemapPlugin)
        .add_plugins(TilemapMaterialPlugin::<MyMaterial>::default())
        .add_systems(Startup, startup)
        .add_systems(Update, helpers::camera::movement)
        .run();
}
