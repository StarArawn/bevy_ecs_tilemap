use crate::TilePos;
use bevy::prelude::*;

/// A component that represents the basic tile information.
#[derive(Debug, Clone, Copy)]
pub struct Tile {
    /// The texture index in the atlas or array.
    pub texture_index: u16,
    /// Flip tile along the x axis.
    pub flip_x: bool,
    /// Flip tile along the Y axis.
    pub flip_y: bool,
    pub flip_d: bool, // anti
    /// Visibility, if false will still process tile events, but will not render the tile.
    pub visible: bool,
    pub color: Color,
}

impl Default for Tile {
    fn default() -> Self {
        Self {
            texture_index: 0,
            flip_x: false,
            flip_y: false,
            flip_d: false, // anti diagonal for rotation
            visible: true,
            color: Color::WHITE,
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

/// This trait is used to allow the layer builder to access specific information inside of the bundle.
pub trait TileBundleTrait: Bundle + Clone + Sized {
    /// Gets the tile position from inside of the bundle.
    fn get_tile_pos_mut(&mut self) -> &mut TilePos;
    /// Gets the tile parent component from inside the bundle.
    fn get_tile_parent(&mut self) -> &mut TileParent;
}

/// The standard tile bundle.
#[derive(Bundle, Clone, Default)]
pub struct TileBundle {
    /// Tile component.
    pub tile: Tile,
    /// The position in the tilemap grid.
    pub position: TilePos,
    /// The parent chunk.
    pub parent: TileParent,
}

impl TileBundleTrait for TileBundle {
    fn get_tile_pos_mut(&mut self) -> &mut TilePos {
        &mut self.position
    }

    fn get_tile_parent(&mut self) -> &mut TileParent {
        &mut self.parent
    }
}

impl TileBundle {
    pub fn new(tile: Tile, position: TilePos) -> Self {
        Self {
            tile,
            position,
            parent: TileParent::default(),
        }
    }
}

/// A component containing the tiles parent information.
#[derive(Clone)]
pub struct TileParent {
    /// The rendering chunk that the tile is attached to.
    pub chunk: Entity,
    /// The layer id the tile is under.
    pub layer_id: u16,
    /// The map id the tile is under.
    pub map_id: u16,
}

impl Default for TileParent {
    fn default() -> Self {
        Self {
            chunk: Entity::new(0),
            layer_id: 0,
            map_id: 0,
        }
    }
}
