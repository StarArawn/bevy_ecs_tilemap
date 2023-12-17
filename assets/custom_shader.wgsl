#import bevy_ecs_tilemap::common::process_fragment
#import bevy_ecs_tilemap::vertex_output::MeshVertexOutput
#import bevy_sprite::mesh2d_view_bindings::globals

struct MyMaterial {
    brightness: f32,
    _padding: vec3<f32>
};

@group(3) @binding(0)
var<uniform> material: MyMaterial;

fn hsv2rgb(c: vec3<f32>) -> vec3<f32>
{
    let K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    let p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, vec3(0.0), vec3(1.0)), c.y);
}

@fragment
fn fragment(in: MeshVertexOutput) -> @location(0) vec4<f32> {
    let color = process_fragment(in);
    
    let hsv = vec3(abs(sin(globals.time)), 1.0, 1.0);
    return vec4((color.rgb + hsv2rgb(hsv)) * material.brightness, color.a);
}