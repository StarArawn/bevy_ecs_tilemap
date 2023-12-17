#define_import_path bevy_ecs_tilemap::vertex_output

struct MeshVertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec4<f32>,
    @location(1) color: vec4<f32>,
    @location(2) @interpolate(flat) tile_id: i32,
    @location(3) storage_position: vec2<u32>,
}
