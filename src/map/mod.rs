use bevy::{
    math::{UVec2, Vec2},
    prelude::{Component, Entity, Handle, Image},
};

/// A component which stores a reference to the tilemap entity.
#[derive(Component, Clone, Copy, Debug, Hash)]
pub struct TilemapId(pub Entity);

impl Default for TilemapId {
    fn default() -> Self {
        Self(Entity::from_raw(0))
    }
}

/// Size of the tilemap in tiles.
#[derive(Component, Default, Clone, Copy, Debug, Hash)]
pub struct TilemapSize {
    pub x: u32,
    pub y: u32,
}

impl TilemapSize {
    pub fn count(&self) -> usize {
        (self.x * self.y) as usize
    }
}

impl Into<UVec2> for TilemapSize {
    fn into(self) -> UVec2 {
        UVec2::new(self.x, self.y)
    }
}

impl Into<Vec2> for TilemapSize {
    fn into(self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32)
    }
}

impl From<UVec2> for TilemapSize {
    fn from(vec: UVec2) -> Self {
        TilemapSize { x: vec.x, y: vec.y }
    }
}

/// A bevy asset handle linking to the tilemap atlas image file.
#[derive(Component, Clone, Default, Debug, Hash)]
pub struct TilemapTexture(pub Handle<Image>);

/// Size of the tiles in pixels
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct TilemapTileSize {
    pub x: f32,
    pub y: f32,
}

impl Into<Vec2> for TilemapTileSize {
    fn into(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

impl Into<TilemapGridSize> for TilemapTileSize {
    fn into(self) -> TilemapGridSize {
        TilemapGridSize {
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
pub struct TilemapGridSize {
    pub x: f32,
    pub y: f32,
}

impl Into<Vec2> for TilemapGridSize {
    fn into(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

/// Spacing between tiles inside of the texture atlas.
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct TilemapSpacing {
    pub x: f32,
    pub y: f32,
}

impl Into<Vec2> for TilemapSpacing {
    fn into(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

impl TilemapSpacing {
    pub fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

/// Size of the atlas texture in pixels.
#[derive(Component, Default, Clone, Copy, Debug)]
pub(crate) struct TilemapTextureSize {
    pub x: f32,
    pub y: f32,
}

impl Into<Vec2> for TilemapTextureSize {
    fn into(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }
}

impl From<Vec2> for TilemapTextureSize {
    fn from(size: Vec2) -> Self {
        TilemapTextureSize {
            x: size.x,
            y: size.y,
        }
    }
}

/// Different hex coordinate systems. You can find out more at this link: <https://www.redblobgames.com/grids/hexagons/>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HexCoordSystem {
    RowEven,
    RowOdd,
    ColumnEven,
    ColumnOdd,
    Row,
    Column,
}

/// Different square coordinate systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IsoCoordSystem {
    Diamond,
    Staggered,
}

/// The type of tile to be rendered, currently we support: Square, Hex, and Isometric.
#[derive(Debug, Component, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TilemapType {
    /// A tilemap with square tiles.
    Square { neighbors_include_diagonals: bool },
    /// Used to specify rendering of tilemaps on hexagons. Note: The `HexCoordSystem` determines the coordinate system.
    Hexagon(HexCoordSystem),
    /// Used to change the rendering mode to Isometric. Note: The `IsoCoordSystem` determines the coordinate system.
    Isometric {
        neighbors_include_diagonals: bool,
        coord_system: IsoCoordSystem,
    },
}

impl TilemapType {
    pub fn square(neighbors_include_diagonals: bool) -> TilemapType {
        TilemapType::Square {
            neighbors_include_diagonals,
        }
    }

    pub fn isometric_diamond(neighbors_include_diagonals: bool) -> TilemapType {
        TilemapType::Isometric {
            neighbors_include_diagonals,
            coord_system: IsoCoordSystem::Diamond,
        }
    }

    pub fn isometric_staggered(neighbors_include_diagonals: bool) -> TilemapType {
        TilemapType::Isometric {
            neighbors_include_diagonals,
            coord_system: IsoCoordSystem::Staggered,
        }
    }
}

impl Default for TilemapType {
    fn default() -> Self {
        Self::Square {
            neighbors_include_diagonals: false,
        }
    }
}
