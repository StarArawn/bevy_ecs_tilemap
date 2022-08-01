use std::num::NonZeroU32;

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
        texture::GpuImage,
    },
    utils::{HashMap, HashSet},
};

#[derive(Default, Debug, Clone)]
pub struct TextureArrayCache {
    textures: HashMap<Handle<Image>, GpuImage>,
    sizes: HashMap<Handle<Image>, (Vec2, Vec2, Vec2, FilterMode)>,
    prepare_queue: HashSet<Handle<Image>>,
    queue_queue: HashSet<Handle<Image>>,
    bad_flag_queue: HashSet<Handle<Image>>,
}

impl TextureArrayCache {
    /// Adds an atlas to the texture array cache.
    pub fn add(
        &mut self,
        atlas_texture: &Handle<Image>,
        tile_size: Vec2,
        texture_size: Vec2,
        tile_spacing: Vec2,
        filter: FilterMode,
    ) {
        if !self.sizes.contains_key(&atlas_texture) {
            self.sizes.insert(
                atlas_texture.clone_weak(),
                (tile_size, texture_size, tile_spacing, filter),
            );
            self.prepare_queue.insert(atlas_texture.clone_weak());
        }
    }

    pub fn get(&self, image_handle: &Handle<Image>) -> &GpuImage {
        self.textures.get(image_handle).unwrap()
    }

    pub fn contains(&self, image_handle: &Handle<Image>) -> bool {
        self.textures.contains_key(image_handle)
    }

    /// Prepares each texture array texture
    pub fn prepare(&mut self, render_device: &RenderDevice) {
        let prepare_queue = self.prepare_queue.drain().collect::<Vec<_>>();
        for item in prepare_queue {
            let (tile_size, atlas_size, spacing, filter) = self.sizes.get(&item).unwrap();
            let tile_count_x =
                ((atlas_size.x as f32 + spacing.x) / (tile_size.x + spacing.x)).floor();
            let tile_count_y =
                ((atlas_size.y as f32 + spacing.y) / (tile_size.y + spacing.y)).floor();
            let mut count = (tile_count_x * tile_count_y) as u32;

            // Fixes weird cubemap bug.
            if count == 6 {
                count += 1;
            }

            let texture = render_device.create_texture(&TextureDescriptor {
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
                lod_max_clamp: std::f32::MAX,
                compare: None,
                anisotropy_clamp: None,
                border_color: None,
            });

            let texture_view = texture.create_view(&TextureViewDescriptor {
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
                texture,
                sampler,
                texture_view,
                size: Vec2::new(tile_size.x, tile_size.y),
                texture_format: TextureFormat::Rgba8UnormSrgb,
            };

            self.textures.insert(item.clone_weak(), gpu_image);
            self.queue_queue.insert(item.clone_weak());
        }
    }

    pub fn queue(
        &mut self,
        render_device: &RenderDevice,
        render_queue: &RenderQueue,
        gpu_images: &Res<RenderAssets<Image>>,
    ) {
        let queue_queue = self.queue_queue.drain().collect::<Vec<_>>();

        for item in queue_queue {
            let atlas_image = if let Some(atlas_image) = gpu_images.get(&item) {
                atlas_image
            } else {
                self.prepare_queue.insert(item);
                continue;
            };

            let (tile_size, atlas_size, spacing, _) = self.sizes.get(&item).unwrap();
            let array_gpu_image = self.textures.get(&item).unwrap();
            let tile_count_x =
                ((atlas_size.x as f32 + spacing.x) / (tile_size.x + spacing.x)).floor();
            let tile_count_y =
                ((atlas_size.y as f32 + spacing.y) / (tile_size.y + spacing.y)).floor();
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
                        texture: &atlas_image.texture,
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
