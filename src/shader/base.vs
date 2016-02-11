uniform mat4 u_model_world;
uniform mat4 u_world_cam;
uniform mat4 u_projection;
uniform mat4 u_normal_cam;

in vec3 a_pos;
in vec3 a_norm;

out vec4 cam_pos;
out vec3 cam_norm;

void main() {
  cam_pos = u_model_world * u_world_cam * vec4(a_pos, 1.0);
  gl_Position = u_projection * cam_pos;

  cam_norm = u_normal_cam * vec4(a_norm, 1.0);
}
