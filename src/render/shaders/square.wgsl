#define_import_path bevy_ecs_tilemap::square

#import bevy_ecs_tilemap::mesh_output

fn get_mesh(v_index: u32, vertex_position: vec3<f32>) -> MeshOutput {
    var out: MeshOutput;

    var position = vertex_position.xy * tilemap_data.grid_size;
    var positions: array<vec2<f32>, 4> = array<vec2<f32>, 4>(
        vec2<f32>(position.x, position.y),
        vec2<f32>(position.x, position.y + tilemap_data.tile_size.y),
        vec2<f32>(position.x + tilemap_data.tile_size.x, position.y + tilemap_data.tile_size.y),
        vec2<f32>(position.x + tilemap_data.tile_size.x, position.y)
    );

    position = positions[v_index % 4u];
    out.world_position = mesh.model * vec4<f32>(position, 0.0, 1.0);

    return out;
}
