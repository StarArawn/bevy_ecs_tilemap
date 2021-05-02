#version 300 es

precision highp float;

in vec3 Vertex_Position;
in vec3 Vertex_Normal;
in vec2 Vertex_Uv;

out vec2 v_Uv;

layout(std140) uniform CameraViewProj { // set = 0, binding = 0
    mat4 ViewProj;
};

layout(std140) uniform Transform { // set = 2, binding = 0
    mat4 Model;
};

void main() {
    vec2 uv = Vertex_Uv;

    v_Uv = uv;
    v_Uv += 1e-5;
    vec3 position = Vertex_Position;
    gl_Position = ViewProj * Model * vec4(position, 1.0);
}