use crate::render::extract::ExtractedTilemapTexture;
use crate::{TilemapSpacing, TilemapTexture, TilemapTextureSize, TilemapTileSize};
use bevy::asset::Assets;
use bevy::prelude::Resource;
use bevy::{
    prelude::{Image, Res},
    render::{
        render_asset::RenderAssets,
        render_resource::{
            AddressMode, CommandEncoderDescriptor, Extent3d, FilterMode, ImageCopyTexture,
            Origin3d, SamplerDescriptor, TextureAspect, TextureDescriptor, TextureDimension,
            TextureFormat, TextureUsages, TextureViewDescriptor, TextureViewDimension,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::GpuImage,
    },
    utils::{HashMap, HashSet},
};
use std::num::NonZeroU32;

#[derive(Resource, Default, Debug, Clone)]
pub struct TextureArrayCache {
    textures: HashMap<TilemapTexture, GpuImage>,
    meta_data: HashMap<
        TilemapTexture,
        (
            u32,
            TilemapTileSize,
            TilemapTextureSize,
            TilemapSpacing,
            FilterMode,
            TextureFormat,
        ),
    >,
    prepare_queue: HashSet<TilemapTexture>,
    queue_queue: HashSet<TilemapTexture>,
    bad_flag_queue: HashSet<TilemapTexture>,
}

impl TextureArrayCache {
    /// Adds an `ExtractedTilemapTexture` to the texture array cache.
    ///
    /// Unlike [`add_texture`](TextureArrayCache::add_texture) it does not perform any verification
    /// checks, as this is assumed to have been done during [`ExtractedTilemapTexture::new`].
    pub(crate) fn add_extracted_texture(&mut self, extracted_texture: &ExtractedTilemapTexture) {
        if !self.meta_data.contains_key(&extracted_texture.texture) {
            self.meta_data.insert(
                extracted_texture.texture.clone_weak(),
                (
                    extracted_texture.tile_count,
                    extracted_texture.tile_size,
                    extracted_texture.texture_size,
                    extracted_texture.tile_spacing,
                    extracted_texture.filtering,
                    extracted_texture.format,
                ),
            );
            self.prepare_queue
                .insert(extracted_texture.texture.clone_weak());
        }
    }

    /// Adds a `TilemapTexture` to the texture array cache.
    pub fn add_texture(
        &mut self,
        texture: TilemapTexture,
        tile_size: TilemapTileSize,
        tile_spacing: TilemapSpacing,
        filtering: FilterMode,
        format: TextureFormat,
        image_assets: &Res<Assets<Image>>,
    ) {
        let (tile_count, texture_size) = match &texture {
            TilemapTexture::Single(handle) => {
                let image = image_assets.get(handle).expect(
                    "Expected image to have finished loading if \
                    it is being extracted as a texture!",
                );
                let texture_size: TilemapTextureSize = image.size().into();
                let tile_count_x = ((texture_size.x) / (tile_size.x + tile_spacing.x)).floor();
                let tile_count_y = ((texture_size.y) / (tile_size.y + tile_spacing.y)).floor();
                ((tile_count_x * tile_count_y) as u32, texture_size)
            }
            TilemapTexture::Vector(handles) => {
                for handle in handles {
                    let image = image_assets.get(handle).expect(
                        "Expected image to have finished loading if \
                        it is being extracted as a texture!",
                    );
                    let this_tile_size: TilemapTileSize = image.size().try_into().unwrap();
                    if this_tile_size != tile_size {
                        panic!(
                            "Expected all provided image assets to have size {:?}, \
                                    but found image with size: {:?}",
                            tile_size, this_tile_size
                        );
                    }
                }
                (handles.len() as u32, tile_size.into())
            }
            TilemapTexture::TextureContainer(handle) => {
                let image = image_assets.get(handle).expect(
                    "Expected image to have finished loading if \
                        it is being extracted as a texture!",
                );
                let tile_size: TilemapTileSize = image.size().into();
                (
                    image.texture_descriptor.array_layer_count(),
                    tile_size.into(),
                )
            }
        };

        if !self.meta_data.contains_key(&texture) {
            self.meta_data.insert(
                texture.clone_weak(),
                (
                    tile_count,
                    tile_size,
                    texture_size,
                    tile_spacing,
                    filtering,
                    format,
                ),
            );
            self.prepare_queue.insert(texture.clone_weak());
        }
    }

    pub fn get(&self, texture: &TilemapTexture) -> &GpuImage {
        self.textures.get(texture).unwrap()
    }

    pub fn contains(&self, texture: &TilemapTexture) -> bool {
        self.textures.contains_key(texture)
    }

    /// Prepares each texture array texture
    pub fn prepare(
        &mut self,
        render_device: &RenderDevice,
        render_images: &Res<RenderAssets<Image>>,
    ) {
        let prepare_queue = self.prepare_queue.drain().collect::<Vec<_>>();
        for texture in prepare_queue.iter() {
            match texture {
                TilemapTexture::Single(_) | TilemapTexture::Vector(_) => {
                    let (count, tile_size, _, _, filter, format) =
                        self.meta_data.get(texture).unwrap();

                    // Fixes weird cubemap bug.
                    let count = if *count == 6 { count + 1 } else { *count };

                    let gpu_texture = render_device.create_texture(&TextureDescriptor {
                        label: Some("texture_array"),
                        size: Extent3d {
                            width: tile_size.x as u32,
                            height: tile_size.y as u32,
                            depth_or_array_layers: count,
                        },
                        mip_level_count: 1,
                        sample_count: 1,
                        dimension: TextureDimension::D2,
                        format: *format,
                        usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
                        view_formats: &[],
                    });

                    let sampler = render_device.create_sampler(&SamplerDescriptor {
                        label: Some("texture_array_sampler"),
                        address_mode_u: AddressMode::ClampToEdge,
                        address_mode_v: AddressMode::ClampToEdge,
                        address_mode_w: AddressMode::ClampToEdge,
                        mag_filter: *filter,
                        min_filter: *filter,
                        mipmap_filter: *filter,
                        lod_min_clamp: 0.0,
                        lod_max_clamp: f32::MAX,
                        compare: None,
                        anisotropy_clamp: None,
                        border_color: None,
                    });

                    let texture_view = gpu_texture.create_view(&TextureViewDescriptor {
                        label: Some("texture_array_view"),
                        format: None,
                        dimension: Some(TextureViewDimension::D2Array),
                        aspect: TextureAspect::All,
                        base_mip_level: 0,
                        mip_level_count: None,
                        base_array_layer: 0,
                        array_layer_count: NonZeroU32::new(count),
                    });

                    let gpu_image = GpuImage {
                        texture_format: *format,
                        texture: gpu_texture,
                        sampler,
                        texture_view,
                        size: tile_size.into(),
                    };

                    self.textures.insert(texture.clone_weak(), gpu_image);
                    self.queue_queue.insert(texture.clone_weak());
                }
                TilemapTexture::TextureContainer(handle) => {
                    if let Some(gpu_image) = render_images.get(handle) {
                        self.textures
                            .insert(texture.clone_weak(), gpu_image.clone());
                    } else {
                        self.prepare_queue.insert(texture.clone_weak());
                    }
                }
            }
        }
    }

    pub fn queue(
        &mut self,
        render_device: &RenderDevice,
        render_queue: &RenderQueue,
        render_images: &Res<RenderAssets<Image>>,
    ) {
        let queue_queue = self.queue_queue.drain().collect::<Vec<_>>();

        for texture in queue_queue.iter() {
            match &texture {
                TilemapTexture::Single(handle) => {
                    let gpu_image = if let Some(gpu_image) = render_images.get(handle) {
                        gpu_image
                    } else {
                        self.prepare_queue.insert(texture.clone_weak());
                        continue;
                    };

                    let (count, tile_size, texture_size, spacing, _, _) =
                        self.meta_data.get(texture).unwrap();
                    let array_gpu_image = self.textures.get(texture).unwrap();
                    let count = *count;

                    let mut command_encoder =
                        render_device.create_command_encoder(&CommandEncoderDescriptor {
                            label: Some("create_texture_array_from_atlas"),
                        });

                    for i in 0..count {
                        let columns = (texture_size.x / (tile_size.x + spacing.x)).floor();
                        let sprite_sheet_x: f32 =
                            (i as f32 % columns).floor() * (tile_size.x + spacing.x) + spacing.x;
                        let sprite_sheet_y: f32 =
                            (i as f32 / columns).floor() * (tile_size.y + spacing.y) + spacing.y;

                        command_encoder.copy_texture_to_texture(
                            ImageCopyTexture {
                                texture: &gpu_image.texture,
                                mip_level: 0,
                                origin: Origin3d {
                                    x: sprite_sheet_x as u32,
                                    y: sprite_sheet_y as u32,
                                    z: 0,
                                },
                                aspect: TextureAspect::All,
                            },
                            ImageCopyTexture {
                                texture: &array_gpu_image.texture,
                                mip_level: 0,
                                origin: Origin3d { x: 0, y: 0, z: i },
                                aspect: TextureAspect::All,
                            },
                            Extent3d {
                                width: tile_size.x as u32,
                                height: tile_size.y as u32,
                                depth_or_array_layers: 1,
                            },
                        );
                    }

                    let command_buffer = command_encoder.finish();
                    render_queue.submit(vec![command_buffer]);
                }
                TilemapTexture::Vector(handles) => {
                    let mut gpu_images = Vec::with_capacity(handles.len());
                    for handle in handles {
                        if let Some(gpu_image) = render_images.get(handle) {
                            gpu_images.push(gpu_image)
                        } else {
                            self.prepare_queue.insert(texture.clone_weak());
                            continue;
                        }
                    }

                    let (count, tile_size, _, _, _, _) = self.meta_data.get(texture).unwrap();
                    let array_gpu_image = self.textures.get(texture).unwrap();
                    let count = *count;

                    let mut command_encoder =
                        render_device.create_command_encoder(&CommandEncoderDescriptor {
                            label: Some("create_texture_array_from_handles_vec"),
                        });

                    for i in 0..count {
                        command_encoder.copy_texture_to_texture(
                            ImageCopyTexture {
                                texture: &gpu_images[i as usize].texture,
                                mip_level: 0,
                                origin: Origin3d { x: 0, y: 0, z: 0 },
                                aspect: TextureAspect::All,
                            },
                            ImageCopyTexture {
                                texture: &array_gpu_image.texture,
                                mip_level: 0,
                                origin: Origin3d { x: 0, y: 0, z: i },
                                aspect: TextureAspect::All,
                            },
                            Extent3d {
                                width: tile_size.x as u32,
                                height: tile_size.y as u32,
                                depth_or_array_layers: 1,
                            },
                        );
                    }

                    let command_buffer = command_encoder.finish();
                    render_queue.submit(vec![command_buffer]);
                }
                TilemapTexture::TextureContainer(_) => {
                    // do nothing, we already have the necessary GPU image
                }
            }
        }
    }
}
