use bevy::{
    math::{UVec2, Vec2},
    prelude::{Component, Entity, Handle, Image},
};

// A component which stores a reference to the tilemap entity.
#[derive(Component, Clone, Copy, Debug)]
pub struct TilemapId(pub Entity);

impl Default for TilemapId {
    fn default() -> Self {
        Self(Entity::from_raw(0))
    }
}

/// Size of the tilemap in tiles.
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct Tilemap2dSize {
    pub x: u32,
    pub y: u32,
}

impl Tilemap2dSize {
    pub fn count(&self) -> usize {
        (self.x * self.y) as usize
    }
}

impl Into<UVec2> for Tilemap2dSize {
    fn into(self) -> UVec2 {
        UVec2::new(self.x, self.y)
    }
}

impl Into<Vec2> for Tilemap2dSize {
    fn into(self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32)
    }
}

impl From<UVec2> for Tilemap2dSize {
    fn from(vec: UVec2) -> Self {
        Tilemap2dSize { x: vec.x, y: vec.y }
    }
}

#[derive(Component, Clone, Default, Debug)]
pub struct TilemapTexture(pub Handle<Image>);

/// Size of the tiles in pixels
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct Tilemap2dTileSize {
    pub x: f32,
    pub y: f32,
}

impl Into<Vec2> for Tilemap2dTileSize {
    fn into(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

impl Into<Tilemap2dGridSize> for Tilemap2dTileSize {
    fn into(self) -> Tilemap2dGridSize {
        Tilemap2dGridSize {
            x: self.x,
            y: self.y,
        }
    }
}

/// Size of the tiles on the grid in pixels.
/// This can be used to overlay tiles on top of each other.
/// Ex. A 16x16 pixel tile can be overlapped by 8 pixels by using
/// a grid size of 16x8.
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct Tilemap2dGridSize {
    pub x: f32,
    pub y: f32,
}

impl Into<Vec2> for Tilemap2dGridSize {
    fn into(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

/// Spacing between tiles inside of the texture atlas.
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct Tilemap2dSpacing {
    pub x: f32,
    pub y: f32,
}

impl Into<Vec2> for Tilemap2dSpacing {
    fn into(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

impl Tilemap2dSpacing {
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

/// Size of the atlas texture in pixels.
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct Tilemap2dTextureSize {
    pub x: f32,
    pub y: f32,
}

impl Into<Vec2> for Tilemap2dTextureSize {
    fn into(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

/// Different hex coordinate systems. You can find out more at this link: https://www.redblobgames.com/grids/hexagons/
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HexType {
    RowEven,
    RowOdd,
    ColumnEven,
    ColumnOdd,
    Row,
    Column,
}

/// Different iso coordinate systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IsoType {
    Diamond,
    Staggered,
}

/// The type of tile to be rendered, currently we support: Square, Hex, and Isometric.
#[derive(Debug, Component, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TilemapMeshType {
    Square,
    Hexagon(HexType),
    Isometric(IsoType),
}

impl Default for TilemapMeshType {
    fn default() -> Self {
        Self::Square
    }
}
