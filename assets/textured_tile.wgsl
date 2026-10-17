#import bevy_ecs_tilemap::common::process_fragment
#import bevy_ecs_tilemap::vertex_output::MeshVertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals

@group(3) @binding(2) var overlay_texture: texture_2d<f32>;
@group(3) @binding(3) var overlay_sampler: sampler;

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    let color = process_fragment(in);
    let overlay_color = textureSample(overlay_texture, overlay_sampler, fract(in.world_position.xy * 0.0069444444));
    return mix(color, overlay_color, color.g);
}