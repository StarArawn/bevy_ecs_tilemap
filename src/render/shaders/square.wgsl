#define_import_path bevy_ecs_tilemap::square
#import bevy_ecs_tilemap::common::{VertexInput, tilemap_data, mesh}
#import bevy_ecs_tilemap::mesh_output::MeshOutput

fn get_mesh(v_index: u32, vertex_position: vec3<f32>) -> MeshOutput {
    var out: MeshOutput;

    let center = vertex_position.xy * tilemap_data.grid_size;
    let bot_left = center - 0.5 * tilemap_data.tile_size;
    let top_right = bot_left + tilemap_data.tile_size;

    var positions = array<vec2<f32>, 4>(
        bot_left,
        vec2<f32>(bot_left.x, top_right.y),
        top_right,
        vec2<f32>(top_right.x, bot_left.y)
    );

    out.world_position = mesh.model * vec4<f32>(positions[v_index % 4u], 0.0, 1.0);

    return out;
}
