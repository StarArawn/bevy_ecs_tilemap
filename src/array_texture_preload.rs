use std::sync::{Arc, RwLock};

use bevy::{
    prelude::{Assets, Handle, Image, Res, ResMut},
    render::{render_resource::FilterMode, texture::ImageSettings, Extract},
};

use crate::{
    prelude::{TilemapSpacing, TilemapTileSize},
    render::TextureArrayCache,
};

#[derive(Default, Debug, Clone)]
pub struct TilemapArrayTexture {
    pub atlas_texture: Handle<Image>,
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
            return textures.drain(..).collect::<Vec<_>>();
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
    for texture in array_texture_loader.drain() {
        if let Some(image) = images.get(&texture.atlas_texture) {
            texture_array_cache.add(
                &texture.atlas_texture,
                texture.tile_size.into(),
                image.size(),
                texture.tile_spacing.into(),
                if let Some(filter) = texture.filter {
                    filter
                } else {
                    default_image_settings.default_sampler.mag_filter
                },
            );
        } else {
            // Image hasn't loaded yet punt to next frame.
            array_texture_loader.add(texture);
        }
    }
}
