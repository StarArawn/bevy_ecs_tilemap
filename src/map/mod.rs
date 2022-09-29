use bevy::render::render_resource::ShaderType;
use bevy::{
    math::{UVec2, Vec2},
    prelude::{Assets, Bundle, Component, Entity, Handle, Image, Res},
};

/// Custom parameters for the render pipeline.
///
/// It must be added as a resource before [`TilemapPlugin`](crate::TilemapPlugin). For example:
/// ```ignore
/// App::new()
///     .insert_resource(WindowDescriptor {
///         width: 1270.0,
///         height: 720.0,
///     })
///     .insert_resource(TilemapRenderSettings {
///         render_chunk_size: UVec2::new(32, 32),
///     })
///     .add_plugin(TilemapPlugin)
///     .run();
/// ```
#[derive(Debug, Default, Copy, Clone)]
pub struct TilemapRenderSettings {
    /// Dimensions of a "chunk" in tiles. Chunks are grouping of tiles combined and rendered as a
    /// single mesh by the render pipeline.
    ///
    /// Larger chunk sizes are better for tilemaps which change infrequently.
    ///
    /// Smaller chunk sizes will benefit tilemaps which change frequently.
    pub render_chunk_size: UVec2,
}

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

impl From<TilemapSize> for Vec2 {
    fn from(tilemap_size: TilemapSize) -> Self {
        Vec2::new(tilemap_size.x as f32, tilemap_size.y as f32)
    }
}

impl From<&TilemapSize> for Vec2 {
    fn from(tilemap_size: &TilemapSize) -> Self {
        Vec2::new(tilemap_size.x as f32, tilemap_size.y as f32)
    }
}

impl From<UVec2> for TilemapSize {
    fn from(vec: UVec2) -> Self {
        TilemapSize { x: vec.x, y: vec.y }
    }
}

#[derive(Component, Clone, Debug, Hash, PartialEq, Eq)]
pub enum TilemapTexture {
    Atlas(Handle<Image>),
    Vector(Vec<Handle<Image>>),
}

impl Default for TilemapTexture {
    fn default() -> Self {
        TilemapTexture::Atlas(Handle::default())
    }
}

impl TilemapTexture {
    /// Create from a vector of image handles, each representing one tile.
    ///
    /// The images must already have been loaded by the asset server.
    ///
    /// Each image must have the same size, and this function will check to make sure this is the
    /// case, panicking if images of different sizes than the `expected_tile_size` are encountered.
    pub fn from_image_handles(
        image_handles: Vec<Handle<Image>>,
        image_assets: Res<Assets<Image>>,
        expected_tile_size: TilemapTileSize,
    ) -> TilemapTexture {
        if image_handles.is_empty() {
            panic!("Image handles vector is empty.");
        }
        let expected_size = Vec2::from(expected_tile_size);
        for handle in image_handles.iter() {
            let image_size = image_assets
                .get(handle)
                .expect("Assets<Image> does not contain image with given handle.")
                .size();
            if !(expected_size == image_size) {
                panic!(
                    "Found an image of size {image_size:?} \
                which is different from expected {expected_size:?}"
                );
            }
        }
        TilemapTexture::Vector(image_handles)
    }

    /// Create from a handle to an image that contains multiple tiles in a single size.
    ///
    /// Each image must have the same size, and this function will check to make sure this is the
    /// case, panicking if images of different sizes than the `expected_tile_size` are encountered.
    pub fn from_atlas(
        image_handle: Handle<Image>,
        image_assets: Res<Assets<Image>>,
        expected_tile_size: TilemapTileSize,
    ) -> TilemapTexture {
        let image_size = image_assets
            .get(&image_handle)
            .expect("Assets<Image> does not contain image with given handle.")
            .size();
        let num_x_tiles = image_size.x / expected_tile_size.x;
        let num_y_tiles = image_size.y / expected_tile_size.y;

        let double_epsilon = 2.0 * f32::EPSILON;
        if num_x_tiles.fract() > double_epsilon || num_y_tiles.fract() > double_epsilon {
            panic!(
                "Expected tile size was: {expected_tile_size:?}, \
            and image size was: {image_size:?}, which does not cleanly divide expected tile size. \
            (num_x_tiles, num_y_tiles): ({num_x_tiles:?}, {num_y_tiles:?})"
            );
        }
        TilemapTexture::Atlas(image_handle)
    }

    pub fn clone_weak(&self) -> Self {
        match self {
            TilemapTexture::Atlas(handle) => TilemapTexture::Atlas(handle.clone_weak()),
            TilemapTexture::Vector(handles) => {
                TilemapTexture::Vector(handles.iter().map(|h| h.clone_weak()).collect())
            }
        }
    }
}

/// Size of the tiles in pixels
#[derive(Component, Default, Clone, Copy, Debug, PartialOrd, PartialEq)]
pub struct TilemapTileSize {
    pub x: f32,
    pub y: f32,
}

impl From<TilemapTileSize> for TilemapGridSize {
    fn from(tile_size: TilemapTileSize) -> Self {
        TilemapGridSize {
            x: tile_size.x,
            y: tile_size.y,
        }
    }
}

impl From<TilemapTileSize> for Vec2 {
    fn from(tile_size: TilemapTileSize) -> Self {
        Vec2::new(tile_size.x, tile_size.y)
    }
}

impl From<&TilemapTileSize> for Vec2 {
    fn from(tile_size: &TilemapTileSize) -> Self {
        Vec2::new(tile_size.x, tile_size.y)
    }
}

impl From<Vec2> for TilemapTileSize {
    fn from(v: Vec2) -> Self {
        let Vec2 { x, y } = v;
        TilemapTileSize { x, y }
    }
}

/// Size of the tiles on the grid in pixels.
/// This can be used to overlay tiles on top of each other.
/// Ex. A 16x16 pixel tile can be overlapped by 8 pixels by using
/// a grid size of 16x8.
#[derive(Component, Default, Clone, Copy, Debug, PartialOrd, PartialEq)]
pub struct TilemapGridSize {
    pub x: f32,
    pub y: f32,
}

impl From<TilemapGridSize> for Vec2 {
    fn from(grid_size: TilemapGridSize) -> Self {
        Vec2::new(grid_size.x, grid_size.y)
    }
}

impl From<&TilemapGridSize> for Vec2 {
    fn from(grid_size: &TilemapGridSize) -> Self {
        Vec2::new(grid_size.x, grid_size.y)
    }
}

impl From<Vec2> for TilemapGridSize {
    fn from(v: Vec2) -> Self {
        TilemapGridSize { x: v.x, y: v.y }
    }
}

impl From<&Vec2> for TilemapGridSize {
    fn from(v: &Vec2) -> Self {
        TilemapGridSize { x: v.x, y: v.y }
    }
}

/// Spacing between tiles inside of the texture atlas.
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct TilemapSpacing {
    pub x: f32,
    pub y: f32,
}

impl From<TilemapSpacing> for Vec2 {
    fn from(spacing: TilemapSpacing) -> Self {
        Vec2::new(spacing.x, spacing.y)
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

impl From<TilemapTextureSize> for Vec2 {
    fn from(texture_size: TilemapTextureSize) -> Self {
        Vec2::new(texture_size.x, texture_size.y)
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

/// Different hex_grid coordinate systems. You can find out more at this link: <https://www.redblobgames.com/grids/hexagons/>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HexCoordSystem {
    RowEven,
    RowOdd,
    ColumnEven,
    ColumnOdd,
    Row,
    Column,
}

/// Different isometric coordinate systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IsoCoordSystem {
    Diamond,
    Staggered,
}

/// The type of tile to be rendered, currently we support: Square, Hex, and Isometric.
#[derive(Debug, Component, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TilemapType {
    /// A tilemap with isometric tiles.
    ///
    /// If `diagonal_neighbors` is `true`, then given a specified tile,
    /// any tiles diagonal to it are also considered neighbors by [`get_tile_neighbors`](crate::helpers::neighbors::get_tile_neighbors)
    /// and [`get_neighboring_pos`](crate::helpers::neighbors::get_neighboring_pos).
    Square { diagonal_neighbors: bool },
    /// Used to specify rendering of tilemaps on hexagons.
    ///
    /// The `HexCoordSystem` determines the coordinate system.
    Hexagon(HexCoordSystem),
    /// Used to change the rendering mode to Isometric.
    ///
    /// The `IsoCoordSystem` determines the coordinate system.
    ///
    /// If `diagonal_neighbors` is `true`, then given a specified tile,
    /// any tiles diagonal to it are also considered neighbors by [`get_tile_neighbors`](crate::helpers::neighbors::get_tile_neighbors)
    /// and [`get_neighboring_pos`](crate::helpers::neighbors::get_neighboring_pos).
    Isometric {
        diagonal_neighbors: bool,
        coord_system: IsoCoordSystem,
    },
}

impl TilemapType {
    pub fn square(neighbors_include_diagonals: bool) -> TilemapType {
        TilemapType::Square {
            diagonal_neighbors: neighbors_include_diagonals,
        }
    }

    pub fn isometric_diamond(neighbors_include_diagonals: bool) -> TilemapType {
        TilemapType::Isometric {
            diagonal_neighbors: neighbors_include_diagonals,
            coord_system: IsoCoordSystem::Diamond,
        }
    }

    pub fn isometric_staggered(neighbors_include_diagonals: bool) -> TilemapType {
        TilemapType::Isometric {
            diagonal_neighbors: neighbors_include_diagonals,
            coord_system: IsoCoordSystem::Staggered,
        }
    }
}

impl Default for TilemapType {
    fn default() -> Self {
        Self::Square {
            diagonal_neighbors: false,
        }
    }
}
