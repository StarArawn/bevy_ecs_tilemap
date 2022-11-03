#[cfg(feature = "render")]
use crate::render::TextureArrayCache;
use crate::{
    prelude::{TilemapSpacing, TilemapTileSize},
    TilemapTexture,
};
use bevy::render::render_resource::{FilterMode, TextureFormat};
#[cfg(feature = "render")]
use bevy::{
    prelude::{Assets, Image, Res, ResMut},
    render::{texture::ImageSettings, Extract},
};
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone)]
pub struct TilemapArrayTexture {
    pub texture: TilemapTexture,
    pub tile_size: TilemapTileSize,
    pub tile_spacing: TilemapSpacing,
    /// Defaults to ImageSettings.
    pub filter: Option<FilterMode>,
    pub format: TextureFormat,
}

/// A bevy world resource that allows you to add atlas textures for
/// loading into the array texture cache.
#[derive(Default, Debug, Clone)]
pub struct ArrayTextureLoader {
    // Arc and RwLock just let us get around Extract's read only rules.
    textures: Arc<RwLock<Vec<TilemapArrayTexture>>>,
}

impl ArrayTextureLoader {
    pub fn add(&self, texture: TilemapArrayTexture) {
        if let Ok(mut textures) = self.textures.try_write() {
            textures.push(texture);
        }
    }

    pub(crate) fn drain(&self) -> Vec<TilemapArrayTexture> {
        if let Ok(mut textures) = self.textures.try_write() {
            return std::mem::take(&mut *textures);
        }
        vec![]
    }
}

#[cfg(feature = "render")]
pub(crate) fn extract(
    images: Extract<Res<Assets<Image>>>,
    default_image_settings: Extract<Res<ImageSettings>>,
    array_texture_loader: Extract<Res<ArrayTextureLoader>>,
    mut texture_array_cache: ResMut<TextureArrayCache>,
) {
    for mut array_texture in array_texture_loader.drain() {
        if array_texture.filter.is_none() {
            array_texture
                .filter
                .replace(default_image_settings.default_sampler.mag_filter);
        }
        if array_texture.texture.verify_ready(&images) {
            texture_array_cache.add_texture(
                array_texture.texture,
                array_texture.tile_size,
                array_texture.tile_spacing,
                default_image_settings.default_sampler.min_filter,
                array_texture.format,
                &images,
            );
        } else {
            // Image hasn't loaded yet punt to next frame.
            array_texture_loader.add(array_texture);
        }
    }
}
