#define_import_path bevy_ecs_tilemap::row_hex

#import bevy_ecs_tilemap::common::{VertexInput, tilemap_data, mesh}
#import bevy_ecs_tilemap::mesh_output::MeshOutput

// Gets the screen space coordinates of the bottom left of an isometric tile position.
fn hex_row_tile_pos_to_world_pos(pos: vec2<f32>, grid_width: f32, grid_height: f32) -> vec2<f32> {
    return vec2<f32>(grid_width * (pos.x + pos.y / 2.0), grid_height * pos.y * 0.75);
}

fn get_mesh(v_index: u32, vertex_position: vec3<f32>) -> MeshOutput {
    var out: MeshOutput;

    let center = hex_row_tile_pos_to_world_pos(vertex_position.xy, tilemap_data.grid_size.x, tilemap_data.grid_size.y);
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
