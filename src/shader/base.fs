#version 330

in vec4 cam_pos;
in vec3 cam_norm;

out vec4 o_color;

void main() {
  vec3 to_cam = normalize(- cam_pos.xyz);
  float incidence_level = dot(to_cam, normalize(cam_norm));
  float grey_level = clamp(incidence_level, 0.0, 1.0) * 0.5 + 0.15;
  o_color = vec4(grey_level, grey_level, grey_level, 1.0);
}
