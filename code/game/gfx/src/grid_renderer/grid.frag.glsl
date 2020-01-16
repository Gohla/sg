#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_EXT_nonuniform_qualifier : require

#define GRID_LENGTH 8
#define GRID_COUNT 64
#define GRID_COUNT_DIV_4 GRID_COUNT / 4

// Inputs
/// Builtin fragment coordinates
layout(location = 0) in vec2 tex;
/// Dynamic inform data
layout(set = 0, binding = 0) uniform sampler2DArray samplerArray;
layout(std140, set = 1, binding = 0) uniform FragmentUniformData {
  // Need to use uvec4 (instead of uint) because GLSL is fucking retarded and always aligns array elements to 16 bytes.
  uvec4 textureIdxs[GRID_COUNT_DIV_4];
} ud;

// Outputs
/// Color
layout(location = 0) out vec4 outCol;

void main() {
  vec2 uv = tex;
  uv *= GRID_LENGTH;
  uvec2 id = uvec2(uv);
  uv = fract(uv);
  float idx = ud.textureIdxs[id.x/4 + id.y*2][id.x%4];
  //outCol = vec4((id.x/4 + id.y*2) / 16.0, (id.x%4) / 4.0, 0.0, 1.0);
  outCol = texture(samplerArray, vec3(uv, idx));
}
