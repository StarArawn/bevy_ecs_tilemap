use std::sync::{Arc, RwLock};

use crate::{
    prelude::{TilemapSpacing, TilemapTileSize},
    render::TextureArrayCache,
    TilemapTexture,
};
use bevy::{
    prelude::{Assets, Image, Res, ResMut},
    render::{render_resource::FilterMode, texture::ImageSettings, Extract},
};

#[derive(Default, Debug, Clone)]
pub struct TilemapArrayTexture {
    pub texture: TilemapTexture,
    pub tile_size: TilemapTileSize,
    pub tile_spacing: TilemapSpacing,
    /// Defaults to ImageSettings.
    pub filter: Option<FilterMode>,
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

pub(crate) fn extract(
    images: Extract<Res<Assets<Image>>>,
    default_image_settings: Extract<Res<ImageSettings>>,
    array_texture_loader: Extract<Res<ArrayTextureLoader>>,
    mut texture_array_cache: ResMut<TextureArrayCache>,
) {
    for array_texture in array_texture_loader.drain() {
        match &array_texture.texture {
            TilemapTexture::Atlas {
                handle: atlas_handle,
                ..
            } => {
                if images.get(&atlas_handle).is_some() {
                    texture_array_cache.add(
                        array_texture.texture,
                        array_texture.tile_size.into(),
                        array_texture.tile_spacing.into(),
                        if let Some(filter) = array_texture.filter {
                            filter
                        } else {
                            default_image_settings.default_sampler.mag_filter
                        },
                    );
                } else {
                    // Image hasn't loaded yet punt to next frame.
                    array_texture_loader.add(array_texture);
                }
            }
            #[cfg(not(feature = "atlas"))]
            TilemapTexture::Vector {
                handles: tile_handles,
                ..
            } => {
                if tile_handles
                    .iter()
                    .all(|tile_handle| images.get(&tile_handle).is_some())
                {
                    texture_array_cache.add(
                        array_texture.texture,
                        array_texture.tile_size.into(),
                        array_texture.tile_spacing.into(),
                        if let Some(filter) = array_texture.filter {
                            filter
                        } else {
                            default_image_settings.default_sampler.mag_filter
                        },
                    );
                } else {
                    // Image hasn't loaded yet punt to next frame.
                    array_texture_loader.add(array_texture);
                }
            }
        }
    }
}
