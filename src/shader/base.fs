#version 330

in vec4 cam_pos;
in vec3 model_norm;
in vec3 world_norm;
in vec3 cam_norm;

out vec4 o_color;

void main() {
  float incidence_level = dot(normalize(- cam_pos.xyz), normalize(cam_norm));
  // vec3 modifier = sin(normalize(cam_pos) * 20.0);
  // vec3 modifier = sin(normalize(cam_pos.xyz * cam_norm) * 200.0) * cos(cam_pos.xyz);
  vec3 modifier = sin(incidence_level * 40) * cam_pos.xxx * cos(cam_norm * 2.0);
  float grey_level = clamp(incidence_level, 0.0, 1.0) * 0.15;
  // o_color = cam_pos;
  // o_color = vec4(normalize(world_norm), 1.0);
  // o_color = vec4(normalize(cam_norm), 1.0);
  // o_color = vec4(grey_level, grey_level, grey_level, 1.0);
  o_color = vec4(grey_level, grey_level, grey_level, 1.0) + vec4(modifier, 1.0);
}
