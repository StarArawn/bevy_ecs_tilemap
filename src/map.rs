use bevy::asset::Assets;
use bevy::ecs::entity::{EntityMapper, MapEntities};
use bevy::ecs::reflect::ReflectMapEntities;
use bevy::prelude::{ReflectComponent, Res, ResMut};
use bevy::render::render_resource::TextureUsages;
use bevy::{
    math::{UVec2, Vec2, Vec3},
    prelude::{Component, Deref, DerefMut, Entity, Handle, Image, Reflect},
};

/// The default chunk_size (in tiles) used per mesh.
pub const CHUNK_SIZE_2D: UVec2 = UVec2::from_array([64, 64]);

/// The order in which we want to perform the render
#[derive(Clone, Debug, Copy, Default)]
pub enum RenderOrder {
    #[default]
    None,
    XThenY,
    XReverseThenY,
    XThenYReverse,
    XReverseThenYReverse,
    YThenX,
    YReverseThenX,
    YThenXReverse,
    YReverseThenXReverse,
}

impl RenderOrder {
    /// Compute a new Z translation value based upon the selected render order
    /// Returned Z value will have an offset between 0 and 11
    pub fn compute_z_translation(
        &self,
        translation: &Vec3,
        tilemap_size: TilemapSize,
        tile_size: TilemapTileSize,
    ) -> f32 {
        let scaling_factor = 10.;
        let map_size_x = tilemap_size.x as f32 * tile_size.x;
        let map_size_y = tilemap_size.y as f32 * tile_size.y;
        let mut z_value = translation.z;
        match self {
            Self::XThenY => {
                z_value += scaling_factor * (translation.x / map_size_x);
                z_value += translation.y / map_size_y;
            }
            Self::XReverseThenY => {
                z_value += scaling_factor * (1. - (translation.x / map_size_x));
                z_value += translation.y / map_size_y;
            }
            Self::XThenYReverse => {
                z_value += scaling_factor * (translation.x / map_size_x);
                z_value += 1. - (translation.y / map_size_y);
            }
            Self::XReverseThenYReverse => {
                z_value += scaling_factor * (1. - (translation.x / map_size_x));
                z_value += 1. - (translation.y / map_size_y);
            }
            Self::YThenX => {
                z_value += translation.x / map_size_x;
                z_value += scaling_factor * (translation.y / map_size_y);
            }
            Self::YReverseThenX => {
                z_value += translation.x / map_size_x;
                z_value += scaling_factor * (1. - (translation.y / map_size_y));
            }
            Self::YThenXReverse => {
                z_value += 1. - (translation.x / map_size_x);
                z_value += scaling_factor * (translation.y / map_size_y);
            }
            Self::YReverseThenXReverse => {
                z_value += 1. - (translation.x / map_size_x);
                z_value += scaling_factor * (1. - (translation.y / map_size_y));
            }
            _ => {}
        };
        z_value
    }
}

/// Custom parameters for the render pipeline.
///
/// It must be added as a component to the tilemap entity.
#[derive(Component, Debug, Copy, Clone)]
pub struct TilemapRenderSettings {
    /// Dimensions of a "chunk" in tiles. Chunks are grouping of tiles combined and rendered as a
    /// single mesh by the render pipeline.
    ///
    /// Larger chunk sizes are better for tilemaps which change infrequently.
    ///
    /// Smaller chunk sizes will benefit tilemaps which change frequently.
    pub render_chunk_size: UVec2,
    /// If true, uses the chunk's `z` and `y` values when sorting during rendering.
    ///
    /// When using this option with layered tilemaps, `z` values for layers should be separated by
    /// at least `1.0` units.
    ///
    /// `render_chunk_size`'s `z` value should be `1` when using this for 3d isometric tilemaps.
    pub y_sort: bool,
    /// The order in which we will render each chunk relative to each other
    pub render_chunk_order: RenderOrder,
}

impl Default for TilemapRenderSettings {
    fn default() -> Self {
        Self {
            render_chunk_size: CHUNK_SIZE_2D,
            y_sort: false,
            render_chunk_order: RenderOrder::None,
        }
    }
}

/// A component which stores a reference to the tilemap entity.
#[derive(Component, Reflect, Clone, Copy, Debug, Hash, Deref, DerefMut, PartialEq, Eq)]
#[reflect(Component, MapEntities)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TilemapId(pub Entity);

impl MapEntities for TilemapId {
    fn map_entities<M: EntityMapper>(&mut self, entity_mapper: &mut M) {
        self.0 = entity_mapper.map_entity(self.0);
    }
}

impl Default for TilemapId {
    fn default() -> Self {
        Self(Entity::from_raw(0))
    }
}

/// Size of the tilemap in tiles.
#[derive(Component, Reflect, Default, Clone, Copy, Debug, Hash)]
#[reflect(Component)]
pub struct TilemapSize {
    pub x: u32,
    pub y: u32,
}

impl TilemapSize {
    pub const fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    pub const fn count(&self) -> usize {
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

impl From<TilemapSize> for UVec2 {
    fn from(size: TilemapSize) -> Self {
        UVec2::new(size.x, size.y)
    }
}

impl From<UVec2> for TilemapSize {
    fn from(vec: UVec2) -> Self {
        TilemapSize { x: vec.x, y: vec.y }
    }
}

#[derive(Component, Reflect, Clone, Debug, Hash, PartialEq, Eq)]
#[reflect(Component)]
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
    /// The tiles are provided as array layers inside a KTX2 or DDS container.
    ///
    /// This only makes sense to use when the `"atlas"` feature is NOT enabled, as texture arrays
    /// are required to handle storing an array of textures. Therefore, this variant is only
    /// available when `"atlas"` is not enabled.
    #[cfg(not(feature = "atlas"))]
    TextureContainer(Handle<Image>),
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
            #[cfg(not(feature = "atlas"))]
            TilemapTexture::TextureContainer(handle) => vec![handle],
        }
    }

    pub fn verify_ready(&self, images: &Res<Assets<Image>>) -> bool {
        #[cfg(feature = "atlas")]
        {
            images.get(self.image_handle()).is_some()
        }

        #[cfg(not(feature = "atlas"))]
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
            // NOTE: We retrieve it non-mutably first to avoid triggering an `AssetEvent::Modified`
            // if we didn't actually need to modify it
            if let Some(image) = images.get(handle) {
                if !image
                    .texture_descriptor
                    .usage
                    .contains(TextureUsages::COPY_SRC)
                {
                    if let Some(image) = images.get_mut(handle) {
                        image.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
                            | TextureUsages::COPY_SRC
                            | TextureUsages::COPY_DST;
                    };
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
            #[cfg(not(feature = "atlas"))]
            TilemapTexture::TextureContainer(handle) => {
                TilemapTexture::TextureContainer(handle.clone_weak())
            }
        }
    }
}

/// Size of the tiles in pixels
#[derive(Component, Reflect, Default, Clone, Copy, Debug, PartialOrd, PartialEq)]
#[reflect(Component)]
pub struct TilemapTileSize {
    pub x: f32,
    pub y: f32,
}

impl TilemapTileSize {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
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
#[derive(Component, Reflect, Default, Clone, Copy, Debug, PartialOrd, PartialEq)]
#[reflect(Component)]
pub struct TilemapGridSize {
    pub x: f32,
    pub y: f32,
}

impl TilemapGridSize {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
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

/// Spacing between tiles in pixels inside of the texture atlas.
/// Defaults to 0.0
#[derive(Component, Reflect, Default, Clone, Copy, Debug)]
#[reflect(Component)]
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
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const fn zero() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

/// Size of the atlas texture in pixels.
#[derive(Component, Reflect, Default, Clone, Copy, Debug)]
#[reflect(Component)]
pub struct TilemapTextureSize {
    pub x: f32,
    pub y: f32,
}

impl TilemapTextureSize {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
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

impl From<TilemapTileSize> for TilemapTextureSize {
    fn from(tile_size: TilemapTileSize) -> Self {
        let TilemapTileSize { x, y } = tile_size;
        TilemapTextureSize { x, y }
    }
}

/// Different hex grid coordinate systems. You can find out more at this link: <https://www.redblobgames.com/grids/hexagons/>
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum HexCoordSystem {
    RowEven,
    RowOdd,
    ColumnEven,
    ColumnOdd,
    Row,
    Column,
}

/// Different isometric coordinate systems.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Reflect)]
pub enum IsoCoordSystem {
    Diamond,
    Staggered,
}

/// The type of tile to be rendered, currently we support: Square, Hex, and Isometric.
#[derive(Component, Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[reflect(Component)]
pub enum TilemapType {
    /// A tilemap with rectangular tiles.
    Square,
    /// Used to specify rendering of tilemaps on hexagons.
    ///
    /// The `HexCoordSystem` determines the coordinate system.
    Hexagon(HexCoordSystem),
    /// Used to change the rendering mode to Isometric.
    ///
    /// The `IsoCoordSystem` determines the coordinate system.
    Isometric(IsoCoordSystem),
}

impl Default for TilemapType {
    fn default() -> Self {
        Self::Square
    }
}
