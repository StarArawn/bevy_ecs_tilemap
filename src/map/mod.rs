use bevy::prelude::{Res, ResMut};
use bevy::render::render_resource::TextureUsages;
use bevy::{
    math::{UVec2, Vec2},
    prelude::{Assets, Component, Entity, Handle, Image},
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
    /// All textures for tiles are inside a single image asset.
    Single(Handle<Image>),
    /// Each tile's texture has its own image asset (each asset must have the same size), so there
    /// is a vector of image assets.
    ///
    /// Each image should have the same size, identical to the provided `TilemapTileSize`. If this
    /// is not the case, a panic will be thrown during the verification when images are being
    /// extracted to the render world.
    ///
    /// This only makes sense to use when the `"atlas"` feature is NOT enabled, as texture arrays
    /// are required to handle storing an array of textures. Therefore, this variant is only
    /// available when `"atlas"` is not enabled.
    #[cfg(not(feature = "atlas"))]
    Vector(Vec<Handle<Image>>),
}

impl Default for TilemapTexture {
    fn default() -> Self {
        TilemapTexture::Single(Default::default())
    }
}

impl TilemapTexture {
    #[cfg(feature = "atlas")]
    pub fn image_handle(&self) -> &Handle<Image> {
        match &self {
            TilemapTexture::Single(handle) => handle,
        }
    }

    pub fn image_handles(&self) -> Vec<&Handle<Image>> {
        match &self {
            TilemapTexture::Single(handle) => vec![handle],
            #[cfg(not(feature = "atlas"))]
            TilemapTexture::Vector(handles) => handles.iter().collect(),
        }
    }

    pub fn verify_ready(&self, images: &Res<Assets<Image>>) -> bool {
        self.image_handles().into_iter().all(|h| {
            if let Some(image) = images.get(h) {
                image
                    .texture_descriptor
                    .usage
                    .contains(TextureUsages::COPY_SRC)
            } else {
                false
            }
        })
    }

    /// Sets images with the `COPY_SRC` flag.
    pub fn set_images_to_copy_src(&self, images: &mut ResMut<Assets<Image>>) {
        for handle in self.image_handles() {
            if let Some(mut image) = images.get_mut(handle) {
                if !image
                    .texture_descriptor
                    .usage
                    .contains(TextureUsages::COPY_SRC)
                {
                    image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
                        | TextureUsages::COPY_SRC
                        | TextureUsages::COPY_DST;
                }
            }
        }
    }

    pub fn clone_weak(&self) -> Self {
        match self {
            TilemapTexture::Single(handle) => TilemapTexture::Single(handle.clone_weak()),
            #[cfg(not(feature = "atlas"))]
            TilemapTexture::Vector(handles) => {
                TilemapTexture::Vector(handles.iter().map(|h| h.clone_weak()).collect())
            }
        }
    }
}

/// Size of the tiles in pixels
#[derive(Component, Default, Clone, Copy, Debug, Hash, PartialOrd, PartialEq, Eq)]
pub struct TilemapTileSize {
    pub x: u32,
    pub y: u32,
}

impl From<TilemapTileSize> for TilemapGridSize {
    fn from(tile_size: TilemapTileSize) -> Self {
        TilemapGridSize {
            x: tile_size.x as f32,
            y: tile_size.y as f32,
        }
    }
}

impl From<TilemapTileSize> for Vec2 {
    fn from(tile_size: TilemapTileSize) -> Self {
        Vec2::new(tile_size.x as f32, tile_size.y as f32)
    }
}

impl From<&TilemapTileSize> for Vec2 {
    fn from(tile_size: &TilemapTileSize) -> Self {
        Vec2::new(tile_size.x as f32, tile_size.y as f32)
    }
}

impl TryFrom<Vec2> for TilemapTileSize {
    type Error = String;

    fn try_from(v: Vec2) -> Result<Self, Self::Error> {
        if v.min_element() < 0.0 {
            Err(format!(
                "Vec2 to TilemapTileSize conversion error: input {v:?} has a negative component."
            ))
        } else {
            let Vec2 { x, y } = v;
            Ok(TilemapTileSize {
                x: x.round() as u32,
                y: y.round() as u32,
            })
        }
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
#[derive(Component, Default, Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct TilemapSpacing {
    pub x: u32,
    pub y: u32,
}

impl From<TilemapSpacing> for Vec2 {
    fn from(spacing: TilemapSpacing) -> Self {
        Vec2::new(spacing.x as f32, spacing.y as f32)
    }
}

impl TilemapSpacing {
    pub fn zero() -> Self {
        Self { x: 0, y: 0 }
    }
}

/// Size of tilemap textures in pixels.
#[derive(Component, Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
pub struct TilemapTextureSize {
    pub x: u32,
    pub y: u32,
}

impl From<Vec2> for TilemapTextureSize {
    fn from(v: Vec2) -> Self {
        let Vec2 { x, y } = v;
        // texture sizes should always be positive and round numbers...
        TilemapTextureSize {
            x: x as u32,
            y: y as u32,
        }
    }
}

impl From<TilemapTextureSize> for Vec2 {
    fn from(size: TilemapTextureSize) -> Self {
        let TilemapTextureSize { x, y } = size;
        Vec2::new(x as f32, y as f32)
    }
}

impl From<TilemapTileSize> for TilemapTextureSize {
    fn from(tile_size: TilemapTileSize) -> Self {
        let TilemapTileSize { x, y } = tile_size;
        TilemapTextureSize { x, y }
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
