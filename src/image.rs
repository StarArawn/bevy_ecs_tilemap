use std::num::NonZeroU32;

use bevy::{
    ecs::system::{
        lifetimeless::{SRes, SResMut},
        SystemParamItem,
    },
    prelude::{Assets, Handle, Image, Local},
    reflect::TypeUuid,
    render::{
        render_asset::{ExtractAssetError, PrepareAssetError, RenderAsset, RenderAssets},
        render_resource::{
            AddressMode, CommandEncoderDescriptor, Extent3d, FilterMode, ImageCopyTexture,
            Origin3d, Sampler, SamplerDescriptor, Texture, TextureAspect, TextureDescriptor,
            TextureDimension, TextureFormat, TextureUsages, TextureView, TextureViewDescriptor,
            TextureViewDimension,
        },
        renderer::{RenderDevice, RenderQueue},
    },
    utils::{HashMap, HashSet},
};

use crate::TileSize;

#[derive(Debug, Clone, TypeUuid)]
#[uuid = "a759744d-6923-48dd-95ca-1f8e2d9ded42"]
pub enum LayerImage {
    Array {
        images: Vec<Handle<Image>>,
        tile_size: TileSize,
    },
    Atlas {
        image_handle: Handle<Image>,
        tile_size: TileSize,
    },
}

impl LayerImage {
    pub fn new_array(images: Vec<Handle<Image>>, tile_size: TileSize) -> Self {
        Self::Array { images, tile_size }
    }

    pub fn new_from_atlas_image(image_handle: &Handle<Image>, tile_size: TileSize) -> Self {
        Self::Atlas {
            image_handle: image_handle.clone(),
            tile_size,
        }
    }
}

#[derive(Debug, Clone)]
pub struct GpuLayerImage {
    pub texture: Texture,
    pub texture_view: TextureView,
    pub sampler: Sampler,
}

impl RenderAsset for LayerImage {
    type ExtractedAsset = (LayerImage, (u32, u32), TextureFormat);
    type PreparedAsset = GpuLayerImage;
    type ExtractParam = (
        SResMut<Assets<Image>>,
        Local<'static, HashSet<Handle<Image>>>,
    );
    type PrepareParam = (
        SRes<RenderDevice>,
        SRes<RenderQueue>,
        SRes<RenderAssets<Image>>,
    );

    /// Clones the LayerImage.
    fn extract_asset(
        &self,
        (images, last_usages): &mut SystemParamItem<Self::ExtractParam>,
    ) -> Result<Self::ExtractedAsset, ExtractAssetError> {
        match self {
            LayerImage::Array { .. } => panic!("Not supported yet!"),
            LayerImage::Atlas { image_handle, .. } => {
                if let Some(image) = images.get_mut(image_handle) {
                    if image
                        .texture_descriptor
                        .usage
                        .contains(TextureUsages::COPY_SRC)
                    {
                        // We need to wait one frame because the current image is wrong.
                        if last_usages.contains(&image_handle) {
                            last_usages.remove(&image_handle);
                            return Ok((
                                self.clone(),
                                (
                                    image.texture_descriptor.size.width,
                                    image.texture_descriptor.size.height,
                                ),
                                image.texture_descriptor.format,
                            ));
                        } else {
                            last_usages.insert(image_handle.clone_weak());
                        }
                    }
                }
            }
        }
        Err(ExtractAssetError::RetryNextUpdate)
    }

    /// Converts the extracted image into a [`GpuLayerImage`].
    fn prepare_asset(
        extracted_asset: Self::ExtractedAsset,
        (render_device, render_queue, gpu_images): &mut SystemParamItem<Self::PrepareParam>,
    ) -> Result<Self::PreparedAsset, PrepareAssetError<Self::ExtractedAsset>> {
        let gpu_image = match &extracted_asset.0 {
            LayerImage::Array { .. } => panic!("Not supported yet!"),
            LayerImage::Atlas {
                image_handle,
                tile_size,
            } => {
                let atlas_size = extracted_asset.1;
                let tile_count_x = atlas_size.0 as f32 / tile_size.0;
                let tile_count_y = atlas_size.1 as f32 / tile_size.1;
                let count = (tile_count_x * tile_count_y).floor() as u32;
                let atlas_image = if let Some(atlas_image) = gpu_images.get(&image_handle) {
                    atlas_image
                } else {
                    return Err(PrepareAssetError::RetryNextUpdate(extracted_asset));
                };

                let texture = render_device.create_texture(&TextureDescriptor {
                    label: Some("texture_array"),
                    size: Extent3d {
                        width: tile_size.0 as u32,
                        height: tile_size.1 as u32,
                        depth_or_array_layers: count,
                    },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: TextureDimension::D2,
                    format: extracted_asset.2,
                    usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
                });

                let sampler = render_device.create_sampler(&SamplerDescriptor {
                    label: Some("texture_array_sampler"),
                    address_mode_u: AddressMode::ClampToEdge,
                    address_mode_v: AddressMode::ClampToEdge,
                    address_mode_w: AddressMode::ClampToEdge,
                    mag_filter: FilterMode::Linear,
                    min_filter: FilterMode::Linear,
                    mipmap_filter: FilterMode::Nearest,
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

                let mut command_encoder =
                    render_device.create_command_encoder(&CommandEncoderDescriptor {
                        label: Some("create_texture_array_from_atlas"),
                    });

                for i in 0..count {
                    let columns = atlas_size.0 as f32 / tile_size.0;
                    let sprite_sheet_x: f32 = (i as f32 % columns).floor() * (tile_size.0);
                    let sprite_sheet_y: f32 = (i as f32 / columns).floor() * (tile_size.1);

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
                            texture: &texture,
                            mip_level: 0,
                            origin: Origin3d {
                                x: 0,
                                y: 0,
                                z: i as u32,
                            },
                            aspect: TextureAspect::All,
                        },
                        Extent3d {
                            width: tile_size.0 as u32,
                            height: tile_size.1 as u32,
                            depth_or_array_layers: 1,
                        },
                    );
                }

                let command_buffer = command_encoder.finish();
                render_queue.submit(vec![command_buffer]);

                GpuLayerImage {
                    texture,
                    sampler,
                    texture_view,
                }
            }
        };

        Ok(gpu_image)
    }
}
