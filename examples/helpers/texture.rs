use bevy::{
    prelude::*,
    render::render_resource::{FilterMode, TextureUsages},
};

pub fn set_texture_filters_to_nearest(
    mut texture_events: EventReader<AssetEvent<Image>>,
    mut textures: ResMut<Assets<Image>>,
) {
    // quick and dirty, run this for all textures anytime a texture is created.
    for event in texture_events.iter() {
        match event {
            AssetEvent::Created { handle } => {
                if let Some(mut texture) = textures.get_mut(handle) {
                    texture.sampler_descriptor.min_filter = FilterMode::Linear;
                    // Important for texture array usage.
                    texture.texture_descriptor.usage = TextureUsages::TEXTURE_BINDING
                        | TextureUsages::COPY_DST
                        | TextureUsages::COPY_SRC;
                }
            }
            _ => (),
        }
    }
}
