use std::collections::HashMap;
use bevy::prelude::{Assets, Handle, Texture, TextureAtlas, Vec2};
use bevy::render::texture::{Extent3d, TextureDimension, TextureFormat};
use bevy::sprite::{Rect, TextureAtlasBuilderError};
use bevy::log::{debug, error, warn};

/// Used to build a `TextureAtlas` that maintains the order of the textures added.
pub struct TileAtlasBuilder {
    /// The size of _all_ tiles in the atlas
    tile_size: Vec2,
    /// The maximum number of columns to allow before wrapping
    ///
    /// If `None`, then there's no wrapping
    columns: Option<usize>,
    /// The ordered collection of texture handles in this atlas
    handles: Vec<Handle<Texture>>,
    /// The texture format for the textures that will be loaded in the atlas.
    format: TextureFormat,
    /// Enable automatic format conversion for textures if they are not in the atlas format.
    auto_format_conversion: bool,
}

impl Default for TileAtlasBuilder {
    fn default() -> Self {
        Self {
            tile_size: Vec2::new(32.0, 32.0),
            columns: None,
            handles: Vec::default(),
            format: TextureFormat::Rgba8UnormSrgb,
            auto_format_conversion: true,
        }
    }
}

impl TileAtlasBuilder {
    pub fn new(tile_size: Vec2) -> Self {
        Self {
            tile_size,
            ..Default::default()
        }
    }

    /// Sets the tile size.
    pub fn tile_size(mut self, size: Vec2) -> Self {
        self.tile_size = size;
        self
    }

    /// Sets the texture format for textures in the atlas.
    pub fn format(mut self, format: TextureFormat) -> Self {
        self.format = format;
        self
    }

    /// Control whether the added texture should be converted to the atlas format, if different.
    pub fn auto_format_conversion(mut self, auto_format_conversion: bool) -> Self {
        self.auto_format_conversion = auto_format_conversion;
        self
    }

    /// Reverses the order of the textures.
    pub fn reverse(&mut self) {
        self.handles.reverse();
    }

    /// Adds a texture to be copied to the texture atlas.
    pub fn add_texture(
        &mut self,
        texture_handle: Handle<Texture>,
        texture: &Texture,
    ) -> Result<usize, ()> {
        if texture.size.width > self.tile_size.x as u32
            || texture.size.height > self.tile_size.y as u32
        {
            error!(
				"Error: Given texture does not fit into specified tile size.\n
				Expected {:?}\n
				Received: {:?}",
				self.tile_size,
				texture.size.as_vec3().truncate(),
			);
            return Err(());
        }

        self.handles.push(texture_handle);
        Ok(self.handles.len() - 1usize)
    }

    /// Build the final `TextureAtlas`
    pub fn finish(
        self,
        textures: &mut Assets<Texture>,
    ) -> Result<TextureAtlas, TextureAtlasBuilderError> {
        let size = &self.tile_size;
        let total = self.handles.len();
        let total_rows = ((total as f32) / self.columns_f32()).ceil() as usize;

        let mut atlas_texture = Texture::new_fill(
            Extent3d::new(
                (self.columns_f32() * size.x) as u32,
                ((total_rows as f32) * size.y) as u32,
                1,
            ),
            TextureDimension::D2,
            &[0, 0, 0, 0],
            self.format,
        );

        let mut col_idx = 0usize;
        let mut row_idx = 0usize;
        let mut texture_handles = HashMap::default();
        let mut texture_rects = Vec::with_capacity(total);
        for (index, handle) in self.handles.iter().enumerate() {
            let texture = textures.get(handle).unwrap();
            let x = (row_idx as f32) * size.x;
            let y = (col_idx as f32) * size.y;
            let min = Vec2::new(x, y);
            let max = min + Vec2::new(x + size.x, y + size.y);

            texture_handles.insert(handle.clone_weak(), index);
            texture_rects.push(Rect { min, max });
            if texture.format != self.format && !self.auto_format_conversion {
                warn!(
					"Loading a texture of format '{:?}' in an atlas with format '{:?}'",
					texture.format, self.format
				);
                return Err(TextureAtlasBuilderError::WrongFormat);
            }
            self.copy_converted_texture(&mut atlas_texture, texture, row_idx, col_idx);

            if (index + 1usize).wrapping_rem(self.columns()) == 0usize {
                col_idx += 1usize;
                row_idx = 0usize;
            } else {
                row_idx += 1usize;
            }
        }

        Ok(TextureAtlas {
            size: atlas_texture.size.as_vec3().truncate(),
            texture: textures.add(atlas_texture),
            textures: texture_rects,
            texture_handles: Some(texture_handles),
        })
    }

    fn columns(&self) -> usize {
        self.columns.unwrap_or(self.handles.len())
    }

    fn columns_f32(&self) -> f32 {
        self.columns.unwrap_or(self.handles.len()) as f32
    }

    fn copy_converted_texture(
        &self,
        atlas_texture: &mut Texture,
        texture: &Texture,
        row_index: usize,
        column_index: usize,
    ) {
        if self.format == texture.format {
            self.copy_texture_to_atlas(atlas_texture, texture, row_index, column_index);
        } else if let Some(converted_texture) = texture.convert(self.format) {
            debug!(
				"Converting texture from '{:?}' to '{:?}'",
				texture.format, self.format
			);
            self.copy_texture_to_atlas(atlas_texture, &converted_texture, row_index, column_index);
        } else {
            error!(
				"Error converting texture from '{:?}' to '{:?}', ignoring",
				texture.format, self.format
			);
        }
    }

    fn copy_texture_to_atlas(
        &self,
        atlas_texture: &mut Texture,
        texture: &Texture,
        row_index: usize,
        column_index: usize,
    ) {
        let rect_width = self.tile_size.x as usize;
        let rect_height = self.tile_size.y as usize;
        let rect_x = row_index * self.tile_size.x as usize;
        let rect_y = column_index * self.tile_size.y as usize;
        let atlas_width = atlas_texture.size.width as usize;
        let format_size = atlas_texture.format.pixel_size();

        for (texture_y, bound_y) in (rect_y..rect_y + rect_height).enumerate() {
            let begin = (bound_y * atlas_width + rect_x) * format_size;
            let end = begin + rect_width * format_size;
            let texture_begin = texture_y * rect_width * format_size;
            let texture_end = texture_begin + rect_width * format_size;

            atlas_texture.data[begin..end]
                .copy_from_slice(&texture.data[texture_begin..texture_end]);
        }
    }
}
