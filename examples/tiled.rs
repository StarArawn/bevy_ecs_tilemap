use bevy::{ecs::query::QueryFilter, prelude::*};
use bevy_ecs_tilemap::prelude::*;

use crate::helpers::tiled::TilesetLayerToStorageEntity;

mod helpers;

#[derive(Resource, Deref, DerefMut)]
pub struct DebouncedTimer(Timer);

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let map_handle: Handle<helpers::tiled::TiledMap> = asset_server.load("map.tmx");

    commands.spawn(helpers::tiled::TiledMapBundle {
        tiled_map: map_handle,
        ..Default::default()
    });
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: String::from("Tiled Map Editor Example"),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest()),
        )
        .add_plugins(TilemapPlugin)
        .add_plugins(helpers::tiled::TiledMapPlugin)
        .insert_resource(DebouncedTimer(Timer::from_seconds(0.5, TimerMode::Once)))
        .add_systems(Startup, startup)
        .add_systems(Update, helpers::camera::movement)
        .add_systems(Update, show_hide_tiles)
        .run();
}

fn show_hide_tiles(
    time: Res<Time>,
    mut timer: ResMut<DebouncedTimer>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    tile_storage_query: Query<&TileStorage>,
    map_query: Query<&TilesetLayerToStorageEntity>,
    mut tile_query: Query<&mut TileVisible>,
) {
    timer.0.tick(time.delta());

    if keyboard_input.pressed(KeyCode::Space) && timer.0.finished() {
        timer.0.reset();
        info!("Hide/Show all tiles");
        let tileset_layer_entity = map_query.single();
        for entity in tileset_layer_entity.get_entities() {
            toggle_tiles_visibility(&tile_storage_query, &mut tile_query, entity);
        }
    }
    if keyboard_input.pressed(KeyCode::KeyC) && timer.0.finished() {
        timer.0.reset();
        info!("Hide/Show castle tiles");
        let tileset_layer_entity = map_query.single();
        for entity in tileset_layer_entity.storage.get(&2).unwrap().values() {
            toggle_tiles_visibility(&tile_storage_query, &mut tile_query, entity);
        }
    }
}

fn toggle_tiles_visibility(
    tile_storage_query: &Query<&TileStorage>,
    tile_query: &mut Query<&mut TileVisible>,
    entity: &Entity,
) {
    if let Ok(map_tiles) = tile_storage_query.get(*entity) {
        for tile in map_tiles.iter().flatten() {
            if let Ok(mut t) = tile_query.get_mut(*tile) {
                t.0 = !t.0;
            }
        }
    }
}
