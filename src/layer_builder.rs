use crate::{
    chunk::ChunkBundle,
    layer::LayerId,
    map::MapId,
    morton_index,
    render2::TilemapUniformData,
    round_to_power_of_two,
    tile::{TileBundleTrait, TileParent},
    Chunk, ChunkPos, IsoType, Layer, LayerBundle, LayerSettings, MapTileError, TilePos,
    TilemapMeshType,
};
use bevy::{
    math::Vec2,
    prelude::{Assets, BuildChildren, Commands, Entity, Handle, ResMut, Transform},
    render2::texture::Image,
    render2::{
        mesh::{Indices, Mesh, VertexAttributeValues},
        render_resource::PrimitiveTopology,
    },
};

/// Useful for creating and modifying a layer in the same system.
pub struct LayerBuilder<T> {
    pub settings: LayerSettings,
    pub(crate) tiles: Vec<(Option<Entity>, Option<T>)>,
    pub(crate) layer_entity: Entity,
}

impl<T> LayerBuilder<T>
where
    T: TileBundleTrait,
{
    /// Creates the layer builder using the layer settings.
    /// The `pipeline` parameter allows you to pass in a custom RenderPipelines
    /// which will be used for rendering each chunk entity.
    pub fn new(
        commands: &mut Commands,
        mut settings: LayerSettings,
        map_id: impl MapId,
        layer_id: impl LayerId,
    ) -> (Self, Entity) {
        let layer_entity = commands.spawn().id();
        let tile_size_x =
            round_to_power_of_two((settings.map_size.0 * settings.chunk_size.0) as f32);
        let tile_size_y =
            round_to_power_of_two((settings.map_size.1 * settings.chunk_size.1) as f32);
        let tile_count = tile_size_x.max(tile_size_y);

        settings.set_map_id(map_id);
        settings.set_layer_id(layer_id);

        (
            Self {
                settings,
                tiles: (0..tile_count * tile_count).map(|_| (None, None)).collect(),
                layer_entity,
            },
            layer_entity,
        )
    }

    /// Uses bevy's `spawn_batch` to quickly create large amounts of tiles.
    /// Note: Limited to T(Bundle + TileBundleTrait) for what gets spawned.
    /// The `pipeline` parameter allows you to pass in a custom RenderPipelines
    /// which will be used for rendering each chunk entity.
    pub fn new_batch<F: FnMut(TilePos) -> Option<T>>(
        commands: &mut Commands,
        mut settings: LayerSettings,
        meshes: &mut ResMut<Assets<Mesh>>,
        material_handle: Handle<Image>,
        map_id: impl MapId,
        layer_id: impl LayerId,
        mut f: F,
    ) -> Entity {
        let layer_entity = commands.spawn().id();

        let size_x = settings.map_size.0 * settings.chunk_size.0;
        let size_y = settings.map_size.1 * settings.chunk_size.1;

        settings.set_map_id(map_id);
        settings.set_layer_id(layer_id);

        let mut layer = Layer::new(settings.clone());
        for x in 0..layer.settings.map_size.0 {
            for y in 0..layer.settings.map_size.1 {
                let mut chunk_entity = None;
                commands
                    .entity(layer_entity)
                    .with_children(|child_builder| {
                        chunk_entity = Some(child_builder.spawn().id());
                    });
                let chunk_entity = chunk_entity.unwrap();

                let chunk_pos = ChunkPos(x, y);
                let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
                mesh.set_attribute("Vertex_Position", VertexAttributeValues::Float32x3(vec![]));
                mesh.set_attribute("Vertex_Texture", VertexAttributeValues::Sint32x4(vec![]));
                mesh.set_indices(Some(Indices::U32(vec![])));
                let mesh_handle = meshes.add(mesh);
                let chunk = Chunk::new(
                    layer_entity,
                    settings.clone(),
                    chunk_pos,
                    mesh_handle.clone(),
                    material_handle.clone(),
                );

                let index = morton_index(chunk_pos);
                layer.chunks[index] = Some(chunk_entity);

                let transform = Self::get_chunk_coords(chunk_pos, &settings);

                let tilemap_data = TilemapUniformData::from(&chunk);

                commands.entity(chunk_entity).insert_bundle(ChunkBundle {
                    chunk,
                    mesh: mesh_handle,
                    transform,
                    tilemap_data,
                    ..Default::default()
                });
            }
        }

        let ref_layer = &layer;
        let chunk_size = settings.chunk_size;
        let layer_id = settings.layer_id;
        let map_id = settings.map_id;
        let bundles: Vec<T> = (0..size_x)
            .flat_map(|x| (0..size_y).map(move |y| (x, y)))
            .filter_map(move |(x, y)| {
                let tile_pos = TilePos(x, y);
                let chunk_pos = ChunkPos(x / chunk_size.0, y / chunk_size.1);
                if let Some(mut tile_bundle) = f(tile_pos) {
                    let tile_parent = tile_bundle.get_tile_parent();
                    *tile_parent = TileParent {
                        chunk: ref_layer.get_chunk(chunk_pos).unwrap(),
                        layer_id,
                        map_id,
                    };
                    let tile_bundle_pos = tile_bundle.get_tile_pos_mut();
                    *tile_bundle_pos = tile_pos;

                    Some(tile_bundle)
                } else {
                    None
                }
            })
            .collect();

        commands.spawn_batch(bundles);

        let layer_bundle = LayerBundle {
            layer,
            transform: Transform::from_xyz(0.0, 0.0, settings.layer_id as f32),
            ..LayerBundle::default()
        };

        let mut layer = layer_bundle.layer;
        let mut transform = layer_bundle.transform;
        layer.settings.layer_id = layer.settings.layer_id;
        transform.translation.z = 0.0; //layer.settings.layer_id as f32;
        commands.entity(layer_entity).insert_bundle(LayerBundle {
            layer,
            transform,
            ..layer_bundle
        });

        layer_entity
    }

    /// Sets a tile's data at the given position.
    pub fn set_tile(&mut self, tile_pos: TilePos, tile: T) -> Result<(), MapTileError> {
        let morton_tile_index = morton_index(tile_pos);
        if morton_tile_index < self.tiles.capacity() {
            self.tiles[morton_tile_index].1 = Some(tile);
            return Ok(());
        }
        Err(MapTileError::OutOfBounds)
    }

    /// Returns an existing tile entity or spawns a new one.
    pub fn get_tile_entity(
        &mut self,
        commands: &mut Commands,
        tile_pos: TilePos,
    ) -> Result<Entity, MapTileError> {
        let morton_tile_index = morton_index(tile_pos);
        if morton_tile_index < self.tiles.capacity() {
            let tile_entity = if self.tiles[morton_tile_index].0.is_some() {
                let tile_entity = self.tiles[morton_tile_index].0;
                tile_entity
            } else {
                let tile_entity = Some(commands.spawn().id());
                self.tiles[morton_tile_index].0 = tile_entity;
                tile_entity
            };

            return Ok(tile_entity.unwrap());
        }

        Err(MapTileError::OutOfBounds)
    }

    /// Returns an existing tile entity if it exists
    pub fn look_up_tile_entity(&self, tile_pos: TilePos) -> Option<Entity> {
        let morton_tile_index = morton_index(tile_pos);
        if morton_tile_index < self.tiles.capacity() {
            if self.tiles[morton_tile_index].0.is_some() {
                return self.tiles[morton_tile_index].0;
            }
        }

        None
    }

    pub(crate) fn get_tile_full(&self, tile_pos: TilePos) -> Option<(Option<Entity>, &T)> {
        let morton_tile_index = morton_index(tile_pos);
        if morton_tile_index < self.tiles.capacity() {
            let tile = &self.tiles[morton_tile_index];
            if let Some(bundle) = &tile.1 {
                return Some((tile.0, bundle));
            }
        }
        None
    }

    /// Gets a reference to the tile data using a tile position.
    pub fn get_tile(&self, tile_pos: TilePos) -> Result<&T, MapTileError> {
        let morton_tile_index = morton_index(tile_pos);
        if morton_tile_index < self.tiles.capacity() {
            if let Some(tile) = &self.tiles[morton_tile_index].1 {
                return Ok(&tile);
            } else {
                return Err(MapTileError::NonExistent);
            }
        }
        Err(MapTileError::OutOfBounds)
    }

    /// Gets a mutable reference to the tile data using the a tile position.
    pub fn get_tile_mut(&mut self, tile_pos: TilePos) -> Result<&mut T, MapTileError> {
        let morton_tile_index = morton_index(tile_pos);
        if morton_tile_index < self.tiles.capacity() {
            if let Some(tile) = &mut self.tiles[morton_tile_index].1 {
                return Ok(tile);
            } else {
                return Err(MapTileError::NonExistent);
            }
        }
        Err(MapTileError::OutOfBounds)
    }

    /// Loops through each tile entity and tile bundle in the builder.
    /// Note: The boolean is for visibility.
    pub fn for_each_tiles<F>(&mut self, mut f: F)
    where
        F: FnMut(Option<Entity>, &Option<T>),
    {
        self.tiles.iter().for_each(|tile| {
            f(tile.0, &tile.1);
        });
    }

    /// Mutably loops through each tile entity and tile bundle in the builder.
    /// Note: The boolean is for visibility.
    pub fn for_each_tiles_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Option<Entity>, &mut Option<T>),
    {
        self.tiles.iter_mut().for_each(|tile| {
            f(&mut tile.0, &mut tile.1);
        });
    }

    /// Fills a section of the map with tiles.
    ///
    /// All tiles within the rectangle defined by
    /// the `[start`, `stop`) positions will be filled.
    /// This includes the `start` position, but not the `stop` position
    pub fn fill(&mut self, start: TilePos, end: TilePos, tile: T) {
        for x in start.0..end.0 {
            for y in start.1..end.1 {
                // Ignore fill errors.
                let _ = self.set_tile(TilePos(x, y), tile.clone());
            }
        }
    }

    /// Sets all of the tiles in the layer builder.
    pub fn set_all(&mut self, tile: T) {
        for tile_option in self.tiles.iter_mut() {
            *tile_option = (tile_option.0, Some(tile.clone()));
        }
    }

    /// Creates a layer bundle from the layer builder.
    pub fn build(
        &mut self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        material: Handle<Image>,
    ) -> LayerBundle {
        let mut layer = Layer::new(self.settings.clone());
        for x in 0..layer.settings.map_size.0 {
            for y in 0..layer.settings.map_size.1 {
                let mut chunk_entity = None;
                commands
                    .entity(self.layer_entity)
                    .with_children(|child_builder| {
                        chunk_entity = Some(child_builder.spawn().id());
                    });
                let chunk_entity = chunk_entity.unwrap();

                let chunk_pos = ChunkPos(x, y);
                let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
                mesh.set_attribute("Vertex_Position", VertexAttributeValues::Float32x3(vec![]));
                mesh.set_attribute("Vertex_Texture", VertexAttributeValues::Sint32x4(vec![]));
                mesh.set_indices(Some(Indices::U32(vec![])));
                let mesh_handle = meshes.add(mesh);
                let mut chunk = Chunk::new(
                    self.layer_entity,
                    self.settings.clone(),
                    chunk_pos,
                    mesh_handle.clone(),
                    material.clone(),
                );

                chunk.build_tiles(chunk_entity, |tile_pos, chunk_entity| {
                    let morton_tile_index = morton_index(tile_pos);

                    if let Some(mut tile_bundle) = self.tiles[morton_tile_index].1.take() {
                        let tile_entity = if let Some(entity) = self.tiles[morton_tile_index].0 {
                            Some(entity)
                        } else {
                            Some(commands.spawn().id())
                        };
                        let tile_parent = tile_bundle.get_tile_parent();
                        *tile_parent = TileParent {
                            chunk: chunk_entity,
                            layer_id: self.settings.layer_id,
                            map_id: self.settings.map_id,
                        };
                        let tile_bundle_pos = tile_bundle.get_tile_pos_mut();
                        *tile_bundle_pos = tile_pos;
                        commands
                            .entity(tile_entity.unwrap())
                            .insert_bundle(tile_bundle);

                        return tile_entity;
                    }
                    None
                });

                let index = morton_index(chunk_pos);
                layer.chunks[index] = Some(chunk_entity);

                let transform = Self::get_chunk_coords(chunk_pos, &self.settings);

                let tilemap_data = TilemapUniformData::from(&chunk);

                commands.entity(chunk_entity).insert_bundle(ChunkBundle {
                    chunk,
                    mesh: mesh_handle,
                    transform,
                    tilemap_data,
                    ..Default::default()
                });
            }
        }

        LayerBundle {
            layer,
            transform: Transform::from_xyz(0.0, 0.0, self.settings.layer_id as f32),
            ..LayerBundle::default()
        }
    }

    fn project_iso_diamond(
        x: f32,
        y: f32,
        chunk_pixel_width: f32,
        chunk_pixel_height: f32,
    ) -> Vec2 {
        let new_x = (x - y) * chunk_pixel_width / 2.0;
        let new_y = (x + y) * chunk_pixel_height / 2.0;
        Vec2::new(new_x, -new_y)
    }

    fn project_iso_staggered(
        x: f32,
        y: f32,
        chunk_pixel_width: f32,
        chunk_pixel_height: f32,
    ) -> Vec2 {
        let new_x = x * chunk_pixel_width;
        let new_y = y * chunk_pixel_height;
        Vec2::new(new_x, new_y)
    }

    fn get_chunk_coords(chunk_pos: ChunkPos, settings: &LayerSettings) -> Transform {
        let chunk_pos = match settings.mesh_type {
            TilemapMeshType::Square => {
                let chunk_pos_x =
                    chunk_pos.0 as f32 * settings.chunk_size.0 as f32 * settings.tile_size.0;
                let chunk_pos_y =
                    chunk_pos.1 as f32 * settings.chunk_size.1 as f32 * settings.tile_size.1;
                Vec2::new(chunk_pos_x, chunk_pos_y)
            }
            TilemapMeshType::Hexagon(crate::HexType::Row) => {
                let chunk_pos_x = (chunk_pos.1 as f32
                    * settings.chunk_size.0 as f32
                    * (0.5 * settings.tile_size.0).floor())
                    + (chunk_pos.0 as f32 * settings.chunk_size.0 as f32 * settings.tile_size.0);
                let chunk_pos_y = chunk_pos.1 as f32
                    * settings.chunk_size.1 as f32
                    * (0.75 * settings.tile_size.1).floor();
                Vec2::new(chunk_pos_x, chunk_pos_y)
            }
            TilemapMeshType::Hexagon(crate::HexType::Column) => {
                let chunk_pos_x = chunk_pos.0 as f32
                    * settings.chunk_size.0 as f32
                    * (0.75 * settings.tile_size.0).floor();
                let chunk_pos_y = (chunk_pos.0 as f32
                    * settings.chunk_size.1 as f32
                    * (0.5 * settings.tile_size.1).ceil())
                    + chunk_pos.1 as f32 * settings.chunk_size.1 as f32 * settings.tile_size.1;
                Vec2::new(chunk_pos_x, chunk_pos_y)
            }
            TilemapMeshType::Hexagon(crate::HexType::RowOdd)
            | TilemapMeshType::Hexagon(crate::HexType::RowEven) => {
                let chunk_pos_x =
                    chunk_pos.0 as f32 * settings.chunk_size.0 as f32 * settings.tile_size.0;
                let chunk_pos_y = chunk_pos.1 as f32
                    * settings.chunk_size.1 as f32
                    * (0.75 * settings.tile_size.1).floor();
                Vec2::new(chunk_pos_x, chunk_pos_y)
            }
            TilemapMeshType::Hexagon(crate::HexType::ColumnOdd)
            | TilemapMeshType::Hexagon(crate::HexType::ColumnEven) => {
                let chunk_pos_x = chunk_pos.0 as f32
                    * settings.chunk_size.0 as f32
                    * (0.75 * settings.tile_size.0).floor();
                let chunk_pos_y =
                    chunk_pos.1 as f32 * settings.chunk_size.1 as f32 * settings.tile_size.1;
                Vec2::new(chunk_pos_x, chunk_pos_y)
            }
            TilemapMeshType::Isometric(IsoType::Diamond) => Self::project_iso_diamond(
                chunk_pos.0 as f32,
                chunk_pos.1 as f32,
                settings.chunk_size.0 as f32 * settings.grid_size.x,
                settings.chunk_size.1 as f32 * settings.grid_size.y,
            ),
            TilemapMeshType::Isometric(IsoType::Staggered) => Self::project_iso_staggered(
                chunk_pos.0 as f32,
                chunk_pos.1 as f32,
                settings.chunk_size.0 as f32 * settings.grid_size.x,
                settings.chunk_size.1 as f32,
            ),
        };

        Transform::from_xyz(chunk_pos.x, chunk_pos.y, 0.0)
    }
}
