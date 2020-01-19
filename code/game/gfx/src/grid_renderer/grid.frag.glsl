#version 450
#extension GL_ARB_separate_shader_objects : enable

// Inputs
/// Builtin fragment coordinates
layout(location = 0) in vec3 tex;
/// Dynamic inform data
layout(set = 0, binding = 0) uniform sampler2DArray samplerArray;

// Outputs
/// Color
layout(location = 0) out vec4 outCol;

void main() {
  outCol = texture(samplerArray, tex);
}
