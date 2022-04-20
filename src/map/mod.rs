use bevy::{
    math::{UVec2, Vec2},
    prelude::{Component, Entity, Handle, Image},
};

#[derive(Component, Clone, Copy, Debug)]
pub struct TilemapId(pub Entity);

impl Default for TilemapId {
    fn default() -> Self {
        Self(Entity::from_raw(0))
    }
}

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
    Diamond3d,
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
