#import bevy_ecs_tilemap::common

fn hsv2rgb(c: vec3<f32>) -> vec3<f32>
{
    let K = vec4(1.0, 2.0 / 3.0, 1.0 / 3.0, 3.0);
    let p = abs(fract(c.xxx + K.xyz) * 6.0 - K.www);
    return c.z * mix(K.xxx, clamp(p - K.xxx, vec3(0.0), vec3(1.0)), c.y);
}

@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = process_fragment(in);
    
    let hsv = vec3(abs(sin(globals.time)), 1.0, 1.0);
    return vec4(color.rgb + hsv2rgb(hsv), color.a);
}