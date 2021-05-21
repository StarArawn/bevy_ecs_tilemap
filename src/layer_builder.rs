use crate::{Chunk, Layer, LayerBundle, LayerSettings, MapTileError, VisibleTile, chunk::ChunkBundle, morton_index, render::TilemapData, round_to_power_of_two, tile::{TileBundleTrait, TileParent}};
use bevy::{
    prelude::*,
    render::{
        mesh::{Indices, VertexAttributeValues},
        pipeline::PrimitiveTopology,
    },
};

/// Useful for creating and modifying a layer in the same system.
pub struct LayerBuilder<T> {
    pub settings: LayerSettings,
    pub(crate) tiles: Vec<(Entity, Option<(T, bool)>)>,
    pub(crate) layer_entity: Entity,
}

impl<T> LayerBuilder<T>
where
    T: TileBundleTrait,
{
    /// Creates the layer builder using the layer settings.
    pub fn new(commands: &mut Commands, layer_entity: Entity, settings: LayerSettings) -> Self {
        let tile_size_x = round_to_power_of_two((settings.map_size.x * settings.chunk_size.x) as f32);
        let tile_size_y = round_to_power_of_two((settings.map_size.y * settings.chunk_size.y) as f32);
        let tile_count = tile_size_x * tile_size_y;
        Self {
            settings,
            tiles: (0..tile_count)
                .map(|_| {
                    //let mut tile_entity = None;
                    // commands.entity(layer_entity).with_children(|child_builder| {
                    // Commented out because child tiles cut performance in half.
                    let tile_entity = Some(commands.spawn().id());
                    // });
                    (tile_entity.unwrap(), None)
                })
                .collect(),
            layer_entity,
        }
    }

    pub fn new_and_build<F: FnMut(&mut Commands, UVec2) -> Entity>(
        commands: &mut Commands,
        settings: LayerSettings,
        meshes: &mut ResMut<Assets<Mesh>>,
        material_handle: Handle<ColorMaterial>,
        mut f: F,
    ) -> Entity {
        let layer_entity = commands.spawn().id();

        let mut layer = Layer::new(settings.clone());
        for x in 0..layer.settings.map_size.x {
            for y in 0..layer.settings.map_size.y {
                let mut chunk_entity = None;
                commands
                    .entity(layer_entity)
                    .with_children(|child_builder| {
                        chunk_entity = Some(child_builder.spawn().id());
                    });
                let chunk_entity = chunk_entity.unwrap();

                let chunk_pos = UVec2::new(x, y);
                let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
                mesh.set_attribute("Vertex_Position", VertexAttributeValues::Float3(vec![]));
                mesh.set_attribute("Vertex_Texture", VertexAttributeValues::Int4(vec![]));
                mesh.set_indices(Some(Indices::U32(vec![])));
                let mesh_handle = meshes.add(mesh);
                let mut chunk = Chunk::new(
                    layer_entity,
                    chunk_pos,
                    settings.chunk_size,
                    settings.tile_size,
                    settings.texture_size,
                    settings.tile_spacing,
                    mesh_handle.clone(),
                    settings.layer_id,
                    settings.mesh_type,
                    dyn_clone::clone_box(&*settings.mesher),
                    settings.cull,
                );

                chunk.build_tiles(chunk_entity, |tile_pos, chunk_entity| {
                    let tile_entity = f(commands, tile_pos);
                    commands
                        .entity(tile_entity)
                        .insert(TileParent(chunk_entity))
                        .insert(tile_pos);
                    Some(tile_entity)
                });

                let index = morton_index(chunk_pos);
                layer.chunks[index] = Some(chunk_entity);

                let transform = Transform::from_xyz(
                    chunk_pos.x as f32
                        * settings.chunk_size.x as f32
                        * settings.tile_size.x,
                    chunk_pos.y as f32
                        * settings.chunk_size.y as f32
                        * settings.tile_size.y,
                    0.0,
                );

                let tilemap_data = TilemapData::from(&chunk.settings);

                commands.entity(chunk_entity).insert_bundle(ChunkBundle {
                    chunk,
                    mesh: mesh_handle,
                    material: material_handle.clone(),
                    transform,
                    tilemap_data,
                    render_pipeline: settings.mesh_type.into(),
                    ..Default::default()
                });
            }
        }

        let layer_bundle = LayerBundle {
            layer,
            transform: Transform::from_xyz(0.0, 0.0, settings.layer_id as f32),
            ..LayerBundle::default()
        };

        let mut layer = layer_bundle.layer;
        let mut transform = layer_bundle.transform;
        layer.settings.layer_id = layer.settings.layer_id;
        transform.translation.z = layer.settings.layer_id as f32;
        commands
            .entity(layer_entity)
            .insert_bundle(LayerBundle {
                layer,
                transform,
                ..layer_bundle
            });

        layer_entity
    }

    /// Sets a tile's data at the given position.
    pub fn set_tile(
        &mut self,
        tile_pos: UVec2,
        tile: T,
        visible: bool,
    ) -> Result<(), MapTileError> {
        let morton_tile_index = morton_index(tile_pos);
        if morton_tile_index < self.tiles.capacity() {
            self.tiles[morton_tile_index].1 = Some((tile, visible));
            return Ok(());
        }
        Err(MapTileError::OutOfBounds)
    }

    /// Returns a tile entity.
    pub fn get_tile_entity(&mut self, tile_pos: UVec2) -> Result<Entity, MapTileError> {
        let morton_tile_index = morton_index(tile_pos);
        if morton_tile_index < self.tiles.capacity() {
            return Ok(self.tiles[morton_tile_index].0);
        }

        Err(MapTileError::OutOfBounds)
    }

    /// Gets a reference to the tile data using the a tile position.
    pub fn get_tile(&self, tile_pos: UVec2) -> Result<&T, MapTileError> {
        let morton_tile_index = morton_index(tile_pos);
        if morton_tile_index < self.tiles.capacity() {
            if let Some(tile) = &self.tiles[morton_tile_index].1 {
                return Ok(&tile.0);
            } else {
                return Err(MapTileError::NonExistent);
            }
        }
        Err(MapTileError::OutOfBounds)
    }

    fn get_tile_i32(&self, tile_pos: IVec2) -> Option<(bevy::prelude::Entity, &T)> {
        if tile_pos.x < 0 || tile_pos.y < 0 {
            return None;
        }
        let morton_tile_index = morton_index(tile_pos.as_u32());
        if morton_tile_index < self.tiles.capacity() {
            let tile = &self.tiles[morton_tile_index];
            if let Some((bundle, _)) = &tile.1 {
                return Some((tile.0, bundle));
            }
        }

        None
    }

    /// Gets a mutable reference to the tile data using the a tile position.
    pub fn get_tile_mut(&mut self, tile_pos: UVec2) -> Result<&mut T, MapTileError> {
        let morton_tile_index = morton_index(tile_pos);
        if morton_tile_index < self.tiles.capacity() {
            if let Some(tile) = &mut self.tiles[morton_tile_index].1 {
                return Ok(&mut tile.0);
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
        F: FnMut(Entity, &Option<(T, bool)>),
    {
        self.tiles.iter().for_each(|tile| {
            f(tile.0, &tile.1);
        });
    }

    /// Mutably loops through each tile entity and tile bundle in the builder.
    /// Note: The boolean is for visibility.
    pub fn for_each_tiles_mut<F>(&mut self, mut f: F)
    where
        F: FnMut(Entity, &mut Option<(T, bool)>),
    {
        self.tiles.iter_mut().for_each(|tile| {
            f(tile.0, &mut tile.1);
        });
    }

    /// Fills a section of the map with tiles.
    pub fn fill(&mut self, start: UVec2, end: UVec2, tile: T, visible: bool) {
        for x in start.x..end.x {
            for y in start.y..end.y {
                // Ignore fill errors.
                let _ = self.set_tile(UVec2::new(x, y), tile.clone(), visible);
            }
        }
    }

    /// Sets all of the tiles in the layer builder.
    pub fn set_all(&mut self, tile: T, visible: bool) {
        for tile_option in self.tiles.iter_mut() {
            *tile_option = (tile_option.0, Some((tile.clone(), visible)));
        }
    }

    /// Retrieves a list of neighbors in the following order:
    /// N, S, W, E, NW, NE, SW, SE.
    ///
    /// The returned neighbors are tuples that have an tilemap coordinate and an Option<(bevy::prelude::Entity, &T)>.
    ///
    /// A value of None will be returned for tiles that don't exist.
    ///
    /// ## Example
    ///
    /// ```
    /// let neighbors = map.get_tile_neighbors(UVec2::new(0, 0));
    /// assert!(neighbors[1].1.is_none()); // Outside of tile bounds.
    /// assert!(neighbors[0].1.is_none()); // Entity returned inside bounds.
    /// ```
    pub fn get_tile_neighbors(
        &self,
        tile_pos: UVec2,
    ) -> [(IVec2, Option<(bevy::prelude::Entity, &T)>); 8] {
        let n = IVec2::new(tile_pos.x as i32, tile_pos.y as i32 + 1);
        let s = IVec2::new(tile_pos.x as i32, tile_pos.y as i32 - 1);
        let w = IVec2::new(tile_pos.x as i32 - 1, tile_pos.y as i32);
        let e = IVec2::new(tile_pos.x as i32 + 1, tile_pos.y as i32);
        let nw = IVec2::new(tile_pos.x as i32 - 1, tile_pos.y as i32 + 1);
        let ne = IVec2::new(tile_pos.x as i32 + 1, tile_pos.y as i32 + 1);
        let sw = IVec2::new(tile_pos.x as i32 - 1, tile_pos.y as i32 - 1);
        let se = IVec2::new(tile_pos.x as i32 + 1, tile_pos.y as i32 - 1);
        [
            (n, self.get_tile_i32(n)),
            (s, self.get_tile_i32(s)),
            (w, self.get_tile_i32(w)),
            (e, self.get_tile_i32(e)),
            (nw, self.get_tile_i32(nw)),
            (ne, self.get_tile_i32(ne)),
            (sw, self.get_tile_i32(sw)),
            (se, self.get_tile_i32(se)),
        ]
    }

    /// Creates a layer bundle from the layer builder.
    pub fn build(
        &mut self,
        commands: &mut Commands,
        meshes: &mut ResMut<Assets<Mesh>>,
        material: Handle<ColorMaterial>,
    ) -> LayerBundle {
        let mut layer = Layer::new(self.settings.clone());
        for x in 0..layer.settings.map_size.x {
            for y in 0..layer.settings.map_size.y {
                let mut chunk_entity = None;
                commands
                    .entity(self.layer_entity)
                    .with_children(|child_builder| {
                        chunk_entity = Some(child_builder.spawn().id());
                    });
                let chunk_entity = chunk_entity.unwrap();

                let chunk_pos = UVec2::new(x, y);
                let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
                mesh.set_attribute("Vertex_Position", VertexAttributeValues::Float3(vec![]));
                mesh.set_attribute("Vertex_Texture", VertexAttributeValues::Int4(vec![]));
                mesh.set_indices(Some(Indices::U32(vec![])));
                let mesh_handle = meshes.add(mesh);
                let mut chunk = Chunk::new(
                    self.layer_entity,
                    chunk_pos,
                    self.settings.chunk_size,
                    self.settings.tile_size,
                    self.settings.texture_size,
                    self.settings.tile_spacing,
                    mesh_handle.clone(),
                    self.settings.layer_id,
                    self.settings.mesh_type,
                    dyn_clone::clone_box(&*self.settings.mesher),
                    self.settings.cull,
                );

                chunk.build_tiles(chunk_entity, |tile_pos, chunk_entity| {
                    let morton_tile_index = morton_index(tile_pos);
                    let tile_entity = self.tiles[morton_tile_index].0;
                    if let Some((tile_bundle, visible)) = self.tiles[morton_tile_index].1.take() {
                        commands
                            .entity(tile_entity)
                            .insert_bundle(tile_bundle)
                            .insert(TileParent(chunk_entity))
                            .insert(tile_pos);

                        if visible {
                            commands.entity(tile_entity).insert(VisibleTile);
                        }
                        Some(tile_entity)
                    } else {
                        None
                    }
                });

                let index = morton_index(chunk_pos);
                layer.chunks[index] = Some(chunk_entity);

                let transform = Transform::from_xyz(
                    chunk_pos.x as f32
                        * self.settings.chunk_size.x as f32
                        * self.settings.tile_size.x,
                    chunk_pos.y as f32
                        * self.settings.chunk_size.y as f32
                        * self.settings.tile_size.y,
                    0.0,
                );

                let tilemap_data = TilemapData::from(&chunk.settings);

                commands.entity(chunk_entity).insert_bundle(ChunkBundle {
                    chunk,
                    mesh: mesh_handle,
                    material: material.clone(),
                    transform,
                    tilemap_data,
                    render_pipeline: self.settings.mesh_type.into(),
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
}
