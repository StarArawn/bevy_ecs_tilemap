import package::render::shaders::common::process_fragment;
import package::render::shaders::tilemap_vertex_output::MeshVertexOutput;

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    return process_fragment(in);
}