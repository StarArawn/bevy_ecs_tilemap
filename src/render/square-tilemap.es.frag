#version 300 es

precision highp float;

in vec2 v_Uv;

out vec4 o_Target;

layout(std140) uniform ColorMaterial_color { // set = 1, binding = 1
    vec4 Color;
};

# ifdef COLORMATERIAL_TEXTURE
uniform sampler2D ColorMaterial_texture;  // set = 1, binding = 1
# endif

void main() {
    vec4 color = Color;
# ifdef COLORMATERIAL_TEXTURE
    color *= texture(
        ColorMaterial_texture,
        v_Uv);
# endif
    o_Target = color;
}