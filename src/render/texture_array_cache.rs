use std::num::NonZeroU32;

use crate::map::TilemapTextureSize;
use crate::{TilemapSpacing, TilemapTexture, TilemapTileSize};
use bevy::{
    math::Vec2,
    prelude::{Handle, Image, Res},
    render::{
        render_asset::RenderAssets,
        render_resource::{
            AddressMode, CommandEncoderDescriptor, Extent3d, FilterMode, ImageCopyTexture,
            Origin3d, SamplerDescriptor, TextureAspect, TextureDescriptor, TextureDimension,
            TextureFormat, TextureUsages, TextureViewDescriptor, TextureViewDimension,
        },
        renderer::{RenderDevice, RenderQueue},
        texture::{BevyDefault, GpuImage},
    },
    utils::{HashMap, HashSet},
};

#[derive(Default, Debug, Clone)]
pub struct TextureArrayCache {
    textures: HashMap<TilemapTexture, GpuImage>,
    sizes: HashMap<
        TilemapTexture,
        (
            u32,
            TilemapTileSize,
            TilemapTextureSize,
            TilemapSpacing,
            FilterMode,
        ),
    >,
    prepare_queue: HashSet<TilemapTexture>,
    queue_queue: HashSet<TilemapTexture>,
    bad_flag_queue: HashSet<Handle<Image>>,
}

impl TextureArrayCache {
    /// Adds a `TilemapTexture` to the texture array cache.
    pub fn add(
        &mut self,
        texture: TilemapTexture,
        tile_size: TilemapTileSize,
        texture_size: TilemapTextureSize,
        tile_spacing: TilemapSpacing,
        filter: FilterMode,
    ) {
        if !self.sizes.contains_key(&texture) {
            let count = match &texture {
                TilemapTexture::Atlas(_) => {
                    let tile_count_x = ((texture_size.x as f32 + tile_spacing.x)
                        / (tile_size.x + tile_spacing.x))
                        .floor();
                    let tile_count_y = ((texture_size.y as f32 + tile_spacing.y)
                        / (tile_size.y + tile_spacing.y))
                        .floor();
                    (tile_count_x * tile_count_y) as u32
                }
                TilemapTexture::Vector(tile_handles) => tile_handles.len() as u32,
            };

            self.sizes.insert(
                texture.clone_weak(),
                (count, tile_size, texture_size, tile_spacing, filter),
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
    pub fn prepare(&mut self, render_device: &RenderDevice) {
        let prepare_queue = self.prepare_queue.drain().collect::<Vec<_>>();
        for texture in prepare_queue {
            let (count, tile_size, atlas_size, spacing, filter) = self.sizes.get(&texture).unwrap();

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
                format: TextureFormat::Rgba8UnormSrgb,
                usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
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
                texture_format: TextureFormat::bevy_default(),
                texture: gpu_texture,
                sampler,
                texture_view,
                size: Vec2::new(tile_size.x, tile_size.y),
            };

            self.textures.insert(texture.clone_weak(), gpu_image);
            self.queue_queue.insert(texture.clone_weak());
        }
    }

    pub fn queue(
        &mut self,
        render_device: &RenderDevice,
        render_queue: &RenderQueue,
        render_images: &Res<RenderAssets<Image>>,
    ) {
        let queue_queue = self.queue_queue.drain().collect::<Vec<_>>();

        for texture in queue_queue {
            let gpu_image = match &texture {
                TilemapTexture::Atlas(atlas_handle) => {
                    let gpu_image = if let Some(gpu_image) = render_images.get(&atlas_handle) {
                        gpu_image
                    } else {
                        self.prepare_queue.insert(texture);
                        continue;
                    };
                }
                TilemapTexture::Vector(_) => {}
            }


            let (count, tile_size, atlas_size, spacing, _) = self.sizes.get(&texture).unwrap();
            let array_gpu_image = self.textures.get(&texture).unwrap();
            let count = (tile_count_x * tile_count_y) as u32;

            let mut command_encoder =
                render_device.create_command_encoder(&CommandEncoderDescriptor {
                    label: Some("create_texture_array_from_atlas"),
                });

            for i in 0..count {
                let columns = (atlas_size.x as f32 + spacing.x) / (tile_size.x + spacing.x);
                let sprite_sheet_x: f32 = (i as f32 % columns).floor() * (tile_size.x + spacing.x);
                let sprite_sheet_y: f32 = (i as f32 / columns).floor() * (tile_size.y + spacing.y);

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
                        origin: Origin3d {
                            x: 0,
                            y: 0,
                            z: i as u32,
                        },
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
    }
}
