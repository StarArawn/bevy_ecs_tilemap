#version 450

layout(location = 0) in vec3 Vertex_Position;
layout(location = 1) in vec3 Vertex_Normal;
layout(location = 2) in vec2 Vertex_Uv;

layout(location = 0) out vec2 v_Uv;

layout(set = 0, binding = 0) uniform CameraViewProj {
    mat4 ViewProj;
};

layout(set = 2, binding = 0) uniform Transform {
    mat4 Model;
};

void main() {
    vec2 uv = Vertex_Uv;

    v_Uv = uv;
    v_Uv += 1e-5;
    vec3 position = Vertex_Position;
    gl_Position = ViewProj * Model * vec4(position, 1.0);
}