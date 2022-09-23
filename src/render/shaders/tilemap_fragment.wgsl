#import bevy_ecs_tilemap::common

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    #import bevy_ecs_tilemap::vertex_output
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    #ifdef ATLAS
    var half_texture_pixel_size_u = 0.5 / tilemap_data.texture_size.x;
    var half_texture_pixel_size_v = 0.5 / tilemap_data.texture_size.y;
    let half_tile_pixel_size_u = 0.5 / tilemap_data.tile_size.x;
    let half_tile_pixel_size_v = 0.5 / tilemap_data.tile_size.y;

        // Offset the UV 1/2 pixel from the sides of the tile, so that the sampler doesn't bleed onto
        // adjacent tiles at the edges.
    var uv_offset: vec2<f32> = vec2<f32>(0.0, 0.0);
    if (in.uv.z < half_tile_pixel_size_u) {
        uv_offset.x = half_texture_pixel_size_u;
    } else if (in.uv.z > (1.0 - half_tile_pixel_size_u)) {
        uv_offset.x = - half_texture_pixel_size_u;
    }
    if (in.uv.w < half_tile_pixel_size_v) {
        uv_offset.y = half_texture_pixel_size_v;
    } else if (in.uv.w > (1.0 - half_tile_pixel_size_v)) {
        uv_offset.y = - half_texture_pixel_size_v;
    }

    var color = textureSample(sprite_texture, sprite_sampler, in.uv.xy + uv_offset) * in.color;
    if (color.a < 0.001) {
            discard;
    }
    return color;
    #else
    var color = textureSample(sprite_texture, sprite_sampler, in.uv.xy, in.tile_id) * in.color;
    if (color.a < 0.001) {
            discard;
    }
    return color;
    #endif
}