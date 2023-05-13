#import bevy_ecs_tilemap::common

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    return process_fragment(in);
}