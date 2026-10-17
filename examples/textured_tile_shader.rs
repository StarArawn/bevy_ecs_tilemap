use bevy::{prelude::*, reflect::TypePath, render::render_resource::AsBindGroup};
use bevy_ecs_tilemap::prelude::*;
mod helpers;

#[derive(AsBindGroup, TypePath, Debug, Clone, Default, Asset)]
pub struct MyMaterial {
    #[texture(2)]
    #[sampler(3)]
    pub texture_overlay: Handle<Image>,
}

impl MaterialTilemap for MyMaterial {
    fn fragment_shader() -> bevy::render::render_resource::ShaderRef {
        "textured_tile.wgsl".into()
    }
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<MyMaterial>>,
) {
    commands.spawn(Camera2d);
    let texture_overlay: Handle<Image> = asset_server.load("tile_texture.png");

    let my_material_handle = MaterialTilemapHandle::from(materials.add(MyMaterial {
        texture_overlay
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

    commands
        .entity(tilemap_entity)
        .insert(MaterialTilemapBundle {
            grid_size,
            map_type,
            size: map_size,
            storage: tile_storage,
            texture: TilemapTexture::Single(texture_handle.clone()),
            tile_size,
            anchor: TilemapAnchor::Center,
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
                        title: String::from("Textured Tiles"),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(TilemapPlugin)
        .add_plugins(MaterialTilemapPlugin::<MyMaterial>::default())
        .add_systems(Startup, startup)
        .add_systems(Update, helpers::camera::movement)
        .run();
}
