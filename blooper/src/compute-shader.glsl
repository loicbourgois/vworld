#version 450
layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;
struct Particle {
  uint is_active;
  float d;
  float x;
  float y;
  float x_before;
  float y_before;
  float mass;
  uint grid_x;
  uint grid_y;
  uint collisions_count;
};
#define max_collision 1024
#define max_particles_count 1024*64
#define max_grid_size 128
struct CollisionCell {
  uint count;
  uint collision_pids [max_collision];
};
layout(set = 0, binding = 0) buffer Data {
    Particle particles[max_particles_count][2];
    CollisionCell collision_grid[max_grid_size][max_grid_size];
} data;
layout(push_constant) uniform pushConstants {
  vec2 gravity;
  uint i_source;
  uint i_target;
  float width;
  float height;
  uint grid_size;
  float delta_time_s;
  float collision_push_rate;
} push_constants;
float get_distance_squared(vec2 pa, vec2 pb) {
  float delta_x = pa.x - pb.x;
  float delta_y = pa.y - pb.y;
  return delta_x * delta_x + delta_y * delta_y;
}
bool particles_are_colliding(Particle pa, Particle pb) {
  float distance_squared_centers = get_distance_squared(vec2(pa.x, pa.y), vec2(pb.x, pb.y));
  float radiuses = (pa.d * 0.5) + (pb.d * 0.5);
  float radiuses_squared = radiuses * radiuses;
  return distance_squared_centers < radiuses_squared;
}
vec2 delta_vector(vec2 va, vec2 vb) {
  return vec2(vb.x - va.x, vb.y - va.y);
}
#define pat data.particles[pid_a][i_target]
#define pas data.particles[pid_a][i_source]
//#define pb_t data.particles[pid_b][i_target]
#define pbs data.particles[pid_b][i_source]
#define pas_xy vec2(pas.x, pas.y)
#define pbs_xy vec2(pbs.x, pbs.y)
#define consts push_constants
void main() {
  uint pid_a = gl_GlobalInvocationID.x;
  uint i_source = push_constants.i_source;
  if (pas.is_active == 0) {
    return;
  }
  uint i_target = push_constants.i_target;
  uint grid_size = push_constants.grid_size;
  // Init target position
  pat.x = pas.x;
  pat.y = pas.y;
  pat.x_before = pas.x;
  pat.y_before = pas.y;
  // Collision response
  vec2 force = vec2(0.0, 0.0);

  uint gi_min = max(0, pas.grid_x-1);
  uint gi_max = min(grid_size-1, pas.grid_x+1);
  uint gj_min = max(0, pas.grid_y-1);
  uint gj_max = min(grid_size-1, pas.grid_y+1);

  for (uint gi=gi_min ; gi <= gi_max ; gi++) {
    for (uint gj=gj_min ; gj <= gj_max ; gj++) {
      for(int i=0 ; i < data.collision_grid[gi][gj].count ; ++i) {
        uint pid_b = data.collision_grid[gi][gj].collision_pids[i];
        //if (pbs.is_active == 0) {
        //  continue;
        //}
        if (pid_a != pid_b) {
          if (particles_are_colliding(pas, pbs)) {
            float distance_centers = distance(pas_xy, pbs_xy);
            float radiuses = (pas.d * 0.5) + (pbs.d * 0.5);
            float delta = radiuses - distance_centers;
            vec2  v_ = delta_vector(pas_xy, pbs_xy);
            vec2 delta_vector = normalize(v_) * delta;
            pat.x -= delta_vector.x * consts.collision_push_rate;
            pat.y -= delta_vector.y * consts.collision_push_rate;
            //pat.x_before -= delta_vector.x * consts.collision_push_rate;
            //pat.y_before -= delta_vector.y * consts.collision_push_rate;
          }
        }
      }
    }
  }


  // Compute gravity force
  vec2 gravity_force = vec2(
    consts.gravity.x * pas.mass * consts.delta_time_s,
    consts.gravity.y * pas.mass * consts.delta_time_s
  );
  // Move target
  vec2 acceleration = gravity_force / pas.mass;
  pat.x += acceleration.x * consts.delta_time_s * consts.delta_time_s;
  pat.y += acceleration.y * consts.delta_time_s * consts.delta_time_s;
  float move_ratio = 1.0;
  pat.x += (pas.x - pas.x_before) * move_ratio;
  pat.y += (pas.y - pas.y_before) * move_ratio;
  // Ground response
  if (pat.y < 0.0) {
    float dy = pat.y - pat.y_before;
    pat.y = 0.0;
    pat.y_before = pat.y + dy;
  }
  if (pat.y > consts.height) {
    float dy = pat.y - pat.y_before;
    pat.y = consts.height;
    pat.y_before = pat.y + dy;
  }
  // Walls response
  if (pat.x < 0.0) {
    float dx = pat.x - pat.x_before;
    pat.x = 0.0;
    pat.x_before = pat.x + dx;
  }
  if (pat.x > consts.width) {
    float dx = pat.x - pat.x_before;
    pat.x = consts.width;
    pat.x_before = pat.x + dx;
  }
  // Update grid
  pat.grid_x = min(max(uint(floor((pat.x * grid_size) / push_constants.width)), 0), consts.grid_size-1);
  pat.grid_y = min(max(uint(floor((pat.y * grid_size) / push_constants.height)), 0), consts.grid_size-1);
}
