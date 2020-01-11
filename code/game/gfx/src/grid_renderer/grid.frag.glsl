#version 450
#extension GL_ARB_separate_shader_objects : enable
#extension GL_EXT_nonuniform_qualifier : require

// Inputs
/// Builtin fragment coordinates
in vec4 gl_FragCoord;
/// Dynamic vertex data
layout(location = 0) in vec3 vrtCol;
/// Dynamic inform data
//layout (set = 0, binding = 1) uniform sampler2D textures[];

// Outputs
/// Color
layout(location = 0) out vec4 outCol;

#define PI 3.14159265358979323846

vec2 rotate2D(vec2 st, float angle){
  st -= 0.5;
  st =  mat2(cos(angle), -sin(angle), sin(angle), cos(angle)) * st;
  st += 0.5;
  return st;
}

void main() {
  vec2 st = gl_FragCoord.xy/vec2(500.0, 500.0);
  st = rotate2D(st, PI*0.25);
  st *= 16.0;
  st = fract(st);
  st = rotate2D(st, PI*0.25);
  outCol = vec4(0.5 * vrtCol + 0.5 * vec3(st, 0.0), 1.0);
}
