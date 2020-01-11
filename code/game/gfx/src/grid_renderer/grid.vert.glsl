#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

// Inputs
/// Dynamic vertex data
layout(location = 0) in vec2 pos;
layout(location = 1) in vec3 col;
/// Dynamic uniform data
layout(set = 0, binding = 0) uniform UniformData { mat4 mvp; } ud;

// Outputs
/// Vertex position
out gl_PerVertex { vec4 gl_Position; };
/// Color
layout(location = 0) out vec3 frgCol;

void main() {
  gl_Position = ud.mvp * vec4(pos, 0.0, 1.0);
  frgCol = col;
}
