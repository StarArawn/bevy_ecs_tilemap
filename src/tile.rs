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

/// Allows the tile to be meshed and rendered.
/// Tiles without this tag will be ignored by the meshing and rendering systems.
#[derive(Debug, Copy, Clone)]
pub struct VisibleTile;

/// A tag that allows you to remove a tile from the world
pub struct RemoveTile;

#[derive(Bundle)]
pub(crate) struct TileBundle {
    tile: Tile,
    visible: VisibleTile,
    position: UVec2,
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
    pub fn new(
        start: u32,
        end: u32,
        speed: f32,
    ) -> Self {
        Self {
            start,
            end,
            speed,
        }
    }
}