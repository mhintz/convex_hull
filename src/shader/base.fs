in vec4 cam_pos;
in vec3 cam_norm;

void main() {
  vec4 to_cam = normalize(- cam_pos);
  float incidence_level = dot(to_cam, normalize(cam_norm));
  float grey_level = clamp(incidence_level, 0.0, 1.0) * 0.5 + 0.15;
  gl_FragColor = vec4(grey_level, grey_level, grey_level, 1.0);
}
