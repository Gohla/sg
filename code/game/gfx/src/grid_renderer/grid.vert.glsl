#version 450
#extension GL_ARB_separate_shader_objects : enable

// Inputs
/// Dynamic vertex data
layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 tex;
/// Dynamic uniform data
layout(push_constant) uniform VertexUniformData { mat4 mvp; } ud;

// Outputs
/// Builtin vertex position
out gl_PerVertex { vec4 gl_Position; };
layout(location = 0) out vec2 frgTex;

void main() {
  gl_Position = ud.mvp * vec4(pos, 0.0, 1.0);
  frgTex = tex;
}
