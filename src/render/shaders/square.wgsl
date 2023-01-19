#define_import_path bevy_ecs_tilemap::square

#import bevy_ecs_tilemap::mesh_output

fn get_mesh(v_index: u32, vertex_position: vec3<f32>) -> MeshOutput {
    var out: MeshOutput;

    let center = vertex_position.xy * tilemap_data.grid_size;
    let offset = 0.5 * tilemap_data.physical_tile_size;
    let bot_left = center - offset;
    let top_right = center + offset;

    var positions = array<vec2<f32>, 4>(
        bot_left,
        vec2<f32>(bot_left.x, top_right.y),
        top_right,
        vec2<f32>(top_right.x, bot_left.y)
    );

    out.world_position = mesh.model * vec4<f32>(positions[v_index % 4u], 0.0, 1.0);

    return out;
}
