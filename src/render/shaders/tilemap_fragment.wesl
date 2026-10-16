import bevy_ecs_tilemap::render::shaders::common::process_fragment;
import bevy_ecs_tilemap::render::shaders::tilemap_vertex_output::MeshVertexOutput;

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    return process_fragment(in);
}
