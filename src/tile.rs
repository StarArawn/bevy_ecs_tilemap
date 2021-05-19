use bevy::prelude::*;

/// A component that represents the basic tile information.
#[derive(Debug, Clone, Copy)]
pub struct Tile {
    /// The chunk entity which is this tiles parent entity.
    pub chunk: Entity,
    /// The texture index in the atlas or array.
    pub texture_index: u32,
    /// Flip tile along the x axis.
    pub flip_x: bool,
    /// Flip tile along the Y axis.
    pub flip_y: bool,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            chunk: Entity::new(0),
            texture_index: 0,
            flip_x: false,
            flip_y: false,
        }
    }
}

impl Into<TileBundle> for Tile {
    fn into(self) -> TileBundle {
        TileBundle {
            tile: self,
            ..Default::default()
        }
    }
}

/// Allows the tile to be meshed and rendered.
/// Tiles without this tag will be ignored by the meshing and rendering systems.
#[derive(Debug, Copy, Clone)]
pub struct VisibleTile;

/// A tag that allows you to remove a tile from the world
pub struct RemoveTile;

/// A component that is attached to a Tile entity that
/// tells the GPU how to animate the tile.
/// Currently all frames must be aligned in your tilemap.
#[derive(Debug, Clone, Copy)]
pub struct GPUAnimated {
    /// The start frame index in the tilemap atlas/array.
    pub start: u32,
    /// The end frame index in the tilemap atlas/array.
    pub end: u32,
    /// The speed the animation plays back at.
    pub speed: f32,
}

impl GPUAnimated {
    pub fn new(start: u32, end: u32, speed: f32) -> Self {
        Self { start, end, speed }
    }
}

/// This trait allows you to create your own entity bundles which
/// allow the layer_builder to access the tile component.
pub trait TileBundleTrait: Bundle + Clone {
    fn get_tile_mut(&mut self) -> &mut Tile;
}

/// The standard tile bundle.
#[derive(Bundle, Clone)]
pub struct TileBundle {
    /// Tile component.
    pub tile: Tile,
}

impl Default for TileBundle {
    fn default() -> Self {
        Self {
            tile: Tile::default(),
        }
    }
}

impl TileBundleTrait for TileBundle {
    fn get_tile_mut(&mut self) -> &mut Tile {
        &mut self.tile
    }
}
