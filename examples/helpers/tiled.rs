// Limitations:
//   Some Tiled tilesets use a single image (a.k.a spritesheet) and then find the image based on
//   caclculated pixel offsets within that image. Other tilesets use a separate image per tile in
//   the tileset. This loader is compatible with either style but will not work with maps that mix
//   the two styles.
//   * Only finite tile layers are loaded. Infinite tile layers and object layers will be skipped.

use std::io::BufReader;
use std::path::{Path, PathBuf};

use bevy::{
    asset::{AssetLoader, AssetPath, LoadedAsset},
    log,
    prelude::{
        AddAsset, Added, AssetEvent, Assets, Bundle, Commands, Component, DespawnRecursiveExt,
        Entity, EventReader, GlobalTransform, Handle, Image, Plugin, Query, Res, Transform,
    },
    reflect::TypeUuid,
    utils::HashMap,
};
use bevy_ecs_tilemap::prelude::*;

use anyhow::{Context, Result};

#[derive(Default)]
pub struct TiledMapPlugin;

impl Plugin for TiledMapPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_asset::<TiledMap>()
            .add_asset_loader(TiledLoader)
            .add_system(process_loaded_maps);
    }
}

#[derive(Debug, TypeUuid)]
#[uuid = "e51081d0-6168-4881-a1c6-4249b2000d7f"]
pub struct TiledMap {
    pub map: tiled::Map,

    // For maps that use a single image per tileset the offset into this vector will be the tileset
    // index in the map (0-based). For maps that use a separate image for each tile in the tileset
    // the offset into this vector must be found via lookup in the tile_texture_indexes map below.
    pub tilesets: Vec<Handle<Image>>,
    pub tile_texture_indexes: HashMap<(usize, tiled::TileId), TileTextureIndex>,
}

// Stores a list of tiled layers.
#[derive(Component, Default)]
pub struct TiledLayersStorage {
    pub storage: HashMap<u32, Entity>,
}

#[derive(Default, Bundle)]
pub struct TiledMapBundle {
    pub tiled_map: Handle<TiledMap>,
    pub storage: TiledLayersStorage,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

pub struct TiledLoader;

impl AssetLoader for TiledLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut bevy::asset::LoadContext,
    ) -> bevy::asset::BoxedFuture<'a, Result<()>> {
        Box::pin(async move {
            // The load context path is the TMX file itself. If the file is at the root of the
            // assets/ directory structure then the tmx_dir will be empty, which is fine.
            let tmx_dir = load_context
                .path()
                .parent()
                .expect("The asset load context was empty.");

            let mut loader = tiled::Loader::new();
            let map = loader
                .load_tmx_map_from(BufReader::new(bytes), load_context.path())
                .map_err(|e| anyhow::anyhow!("Could not load TMX map: {e}"))?;

            let mut dependencies = Vec::new();
            let mut handles = Vec::new();
            let mut tile_texture_indexes = HashMap::default();

            for (tileset_index, tileset) in map.tilesets().iter().enumerate() {
                match &tileset.image {
                    None => {
                        for (tile_id, tile) in tileset.tiles() {
                            if let Some(img) = &tile.image {
                                let tile_path = tmx_dir.join(&img.source);
                                let asset_path = AssetPath::new(tile_path, None);
                                log::debug!("Loading tile image from {asset_path:?} as image ({tileset_index}, {tile_id})");
                                let texture: Handle<Image> =
                                    load_context.get_handle(asset_path.clone());
                                handles.push(texture.clone());
                                tile_texture_indexes.insert(
                                    (tileset_index, tile_id),
                                    TileTextureIndex(handles.len() as u32 - 1),
                                );
                                dependencies.push(asset_path);
                            }
                        }
                    }
                    Some(img) => {
                        let tile_path = tmx_dir.join(&img.source);
                        let asset_path = AssetPath::new(tile_path, None);
                        let texture: Handle<Image> = load_context.get_handle(asset_path.clone());
                        handles.push(texture.clone());
                        dependencies.push(asset_path);
                    }
                }
            }

            let asset_map = TiledMap {
                map,
                tilesets: handles,
                tile_texture_indexes: tile_texture_indexes,
            };
            log::info!("Loaded map: {}", load_context.path().display());
            let loaded_asset = LoadedAsset::new(asset_map);
            load_context.set_default_asset(loaded_asset.with_dependencies(dependencies));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        static EXTENSIONS: &[&str] = &["tmx"];
        EXTENSIONS
    }
}

pub fn process_loaded_maps(
    mut commands: Commands,
    mut map_events: EventReader<AssetEvent<TiledMap>>,
    maps: Res<Assets<TiledMap>>,
    tile_storage_query: Query<(Entity, &TileStorage)>,
    mut map_query: Query<(&Handle<TiledMap>, &mut TiledLayersStorage)>,
    new_maps: Query<&Handle<TiledMap>, Added<Handle<TiledMap>>>,
) {
    let mut changed_maps = Vec::<Handle<TiledMap>>::default();
    for event in map_events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                log::info!("Map added!");
                changed_maps.push(handle.clone());
            }
            AssetEvent::Modified { handle } => {
                log::info!("Map changed!");
                changed_maps.push(handle.clone());
            }
            AssetEvent::Removed { handle } => {
                log::info!("Map removed!");
                // if mesh was modified and removed in the same update, ignore the modification
                // events are ordered so future modification events are ok
                changed_maps.retain(|changed_handle| changed_handle == handle);
            }
        }
    }

    // If we have new map entities add them to the changed_maps list.
    for new_map_handle in new_maps.iter() {
        changed_maps.push(new_map_handle.clone_weak());
    }

    for changed_map in changed_maps.iter() {
        for (map_handle, mut layer_storage) in map_query.iter_mut() {
            // only deal with currently changed map
            if map_handle != changed_map {
                continue;
            }
            if let Some(tiled_map) = maps.get(map_handle) {
                // TODO: Create a RemoveMap component..
                for layer_entity in layer_storage.storage.values() {
                    if let Ok((_, layer_tile_storage)) = tile_storage_query.get(*layer_entity) {
                        for tile in layer_tile_storage.iter().flatten() {
                            commands.entity(*tile).despawn_recursive()
                        }
                    }
                    // commands.entity(*layer_entity).despawn_recursive();
                }

                // The TilemapBundle requires that all tile images come exclusively from a single
                // tiled texture or from a Vec of independent per-tile images. Furthermore, all of
                // the per-tile images must be the same size. Since Tiled allows tiles of mixed
                // tilesets on each layer and allows differently-sized tile images in each tileset,
                // this means we need to load each combination of tileset and layer separately.
                for (tileset_index, tileset) in tiled_map.map.tilesets().iter().enumerate() {
                    let tile_size = TilemapTileSize {
                        x: tileset.tile_width as f32,
                        y: tileset.tile_height as f32,
                    };

                    let tile_spacing = TilemapSpacing {
                        x: tileset.spacing as f32,
                        y: tileset.spacing as f32,
                    };

                    // Once materials have been created/added we need to then create the layers.
                    for (layer_index, layer) in tiled_map.map.layers().enumerate() {
                        let offset_x = layer.offset_x;
                        let offset_y = layer.offset_y;

                        let tiled::LayerType::TileLayer(tile_layer) = layer.layer_type() else {
                            log::info!(
                                "Skipping layer because {:?} is not supported.",
                                layer.layer_type()
                            );
                            continue;
                        };

                        match tile_layer {
                            tiled::TileLayer::Finite(layer_data) => {
                                let map_size = TilemapSize {
                                    x: tiled_map.map.width,
                                    y: tiled_map.map.height,
                                };

                                let grid_size = TilemapGridSize {
                                    x: tiled_map.map.tile_width as f32,
                                    y: tiled_map.map.tile_height as f32,
                                };

                                let map_type = match tiled_map.map.orientation {
                                    tiled::Orientation::Hexagonal => {
                                        TilemapType::Hexagon(HexCoordSystem::Row)
                                    }
                                    tiled::Orientation::Isometric => {
                                        TilemapType::Isometric(IsoCoordSystem::Diamond)
                                    }
                                    tiled::Orientation::Staggered => {
                                        TilemapType::Isometric(IsoCoordSystem::Staggered)
                                    }
                                    tiled::Orientation::Orthogonal => TilemapType::Square,
                                };

                                let mut tile_storage = TileStorage::empty(map_size);
                                let layer_entity = commands.spawn_empty().id();

                                for x in 0..map_size.x {
                                    for y in 0..map_size.y {
                                        let mut mapped_y = y;
                                        if tiled_map.map.orientation
                                            == tiled::Orientation::Orthogonal
                                        {
                                            mapped_y = (tiled_map.map.height - 1) - y;
                                        }

                                        let mapped_x = x as i32;
                                        let mapped_y = mapped_y as i32;

                                        let layer_tile = match layer_data
                                            .get_tile(mapped_x as i32, mapped_y as i32)
                                        {
                                            Some(t) => t,
                                            None => {
                                                continue;
                                            }
                                        };
                                        if tileset_index != layer_tile.tileset_index() {
                                            continue;
                                        }
                                        let layer_tile_data =
                                            match layer_data.get_tile_data(mapped_x, mapped_y) {
                                                Some(d) => d,
                                                None => {
                                                    continue;
                                                }
                                            };

                                        let texture_index = match tileset.image {
                                            Some(_) => TileTextureIndex(layer_tile_data.id()),
                                            None => tiled_map
                                                .tile_texture_indexes
                                                .get(&(tileset_index, layer_tile.id()))
                                                .expect(
                                                    "The tile should have been mapped previously.",
                                                )
                                                .clone(),
                                        };

                                        let tile_pos = TilePos { x, y };
                                        let tile_entity = commands
                                            .spawn(TileBundle {
                                                position: tile_pos,
                                                tilemap_id: TilemapId(layer_entity),
                                                texture_index: texture_index,
                                                flip: TileFlip {
                                                    x: layer_tile_data.flip_h,
                                                    y: layer_tile_data.flip_v,
                                                    d: layer_tile_data.flip_d,
                                                },
                                                ..Default::default()
                                            })
                                            .id();
                                        tile_storage.set(&tile_pos, tile_entity);
                                    }
                                }

                                let texture = if tiled_map.tile_texture_indexes.is_empty() {
                                    TilemapTexture::Single(
                                        tiled_map.tilesets[tileset_index].clone_weak(),
                                    )
                                } else {
                                    TilemapTexture::Vector(tiled_map.tilesets.clone())
                                };

                                commands.entity(layer_entity).insert(TilemapBundle {
                                    grid_size,
                                    size: map_size,
                                    storage: tile_storage,
                                    texture: texture,
                                    tile_size,
                                    spacing: tile_spacing,
                                    transform: get_tilemap_center_transform(
                                        &map_size,
                                        &grid_size,
                                        &map_type,
                                        layer_index as f32,
                                    ) * Transform::from_xyz(offset_x, -offset_y, 0.0),
                                    map_type,
                                    ..Default::default()
                                });

                                layer_storage
                                    .storage
                                    .insert(layer_index as u32, layer_entity);
                            }
                            _ => {
                                log::info!(
                                    "Skipping layer because {:?} is not supported.",
                                    tile_layer
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}
