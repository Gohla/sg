#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_EXT_nonuniform_qualifier : require

#define GRID_LENGTH 16
#define GRID_COUNT GRID_LENGTH * GRID_LENGTH
#define GRID_COUNT_DIV_4 16 // No integer division in GLSL: manually calculate.

// Inputs
/// Builtin fragment coordinates
in vec4 gl_FragCoord;
/// Dynamic inform data
layout(set = 0, binding = 0) uniform sampler2D textures[];
layout(std140, set = 1, binding = 0) uniform FragmentUniformData {
  // Need to use uvec4 (instead of uint) because GLSL is fucking retarded and always aligns array elements to 16 bytes.
  uvec4 textureIdxs[GRID_COUNT_DIV_4];
  vec2 viewport;
} ud;

// Outputs
/// Color
layout(location = 0) out vec4 outCol;

void main() {
  vec2 uv = gl_FragCoord.xy / ud.viewport;
  uv *= GRID_LENGTH;
  uv = fract(uv);
  uvec2 id = uvec2(uv);
  uint idx = ud.textureIdxs[id.x + id.y][0]; // TODO: fix indexing
  outCol = texture(textures[idx], uv);
}
