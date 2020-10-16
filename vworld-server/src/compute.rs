//use std::marker::Send;
use crate::puuid;
use std::collections::HashMap;
use crate::Point;
use crate::Vector;
use crate::Chunk;
use crate::Particle;
use crate::particle::ParticleType;
#[derive(Copy, Clone)]
pub struct Collision {
    pub puuids: [puuid; 2],
    pub delta_vector: Vector,
    pub collision_x: f64,
}
pub struct ComputeInputData {
    pub id: usize,
    pub puuids: Vec<puuid>,
}
pub struct ParticleUpdate {
    pub x: f64,
    pub y: f64,
    pub x_old: f64,
    pub y_old: f64,
    pub energy: f64,
    pub is_colliding_other_entity: bool,
}
pub struct ComputeOutputData {
    pub id: usize,
    pub particle_updates: HashMap<puuid, ParticleUpdate>,
}
pub fn compute(
    chunk: &Chunk,
    puuids: &Vec<puuid>,
    particle_updates: &mut HashMap<puuid, ParticleUpdate>
) {
    // Reset particle_updates
    reset_particle_updates(chunk, particle_updates);
    // Compute collisions
    let mut collisions = Vec::new();
    compute_collisions(chunk, puuids, &mut collisions);



    //

    for collision in collisions.iter() {
        let p1_is_mouth: bool = {
            match chunk.particles.get(&collision.puuids[0]).unwrap().type_ {
                ParticleType::Mouth => true,
                _ => false
            }
        };
        let p2_is_mouth: bool = {
            match chunk.particles.get(&collision.puuids[1]).unwrap().type_ {
                ParticleType::Mouth => true,
                _ => false
            }
        };
        let e1_is_not_e2: bool = {
            chunk.particles.get(&collision.puuids[0]).unwrap().euuid != chunk.particles.get(&collision.puuids[1]).unwrap().euuid
        };
        let collision_push_rate = chunk.constants.collision_push_rate;
        let mouth_energy_eating_rate: f64 = chunk.constants.mouth_energy_eating_rate_per_second * chunk.constants.delta_time;
        let p1 = particle_updates.get_mut(&collision.puuids[0]).unwrap();
        p1.x -= collision.delta_vector.x * collision_push_rate;
        p1.y -= collision.delta_vector.y * collision_push_rate;
        p1.x_old -= collision.delta_vector.x * collision_push_rate;
        p1.y_old -= collision.delta_vector.y * collision_push_rate;
        if e1_is_not_e2 {
            if p1_is_mouth {
                p1.energy += mouth_energy_eating_rate;
            }
            if p2_is_mouth {
                p1.energy -= mouth_energy_eating_rate;
            }
            p1.is_colliding_other_entity = true;
        }
        let p2 = particle_updates.get_mut(&collision.puuids[1]).unwrap();
        p2.x += collision.delta_vector.x * collision_push_rate;
        p2.y += collision.delta_vector.y * collision_push_rate;
        p2.x_old += collision.delta_vector.x * collision_push_rate;
        p2.y_old += collision.delta_vector.y * collision_push_rate;
        if e1_is_not_e2 {
            if p1_is_mouth {
                p2.energy -= mouth_energy_eating_rate;
            }
            if p2_is_mouth {
                p2.energy += mouth_energy_eating_rate;
            }
            p2.is_colliding_other_entity = true;
        }
    }
}
fn reset_particle_updates (
    chunk: &Chunk,
    particle_updates: &mut HashMap<puuid, ParticleUpdate>
) {
    for (puuid, p) in chunk.particles.iter() {
        particle_updates.insert(*puuid, ParticleUpdate {
            x: 0.0,
            y: 0.0,
            x_old: 0.0,
            y_old: 0.0,
            energy: 0.0,
            is_colliding_other_entity: false,
        });
    }
}
fn compute_collisions (
    chunk: &Chunk,
    puuids: &Vec<puuid>,
    collisions: &mut Vec<Collision>
) {
    let collision_sections_count = 50;
    let mut divisions: Vec<Vec<Vec<puuid>>> = vec![vec![Vec::new(); collision_sections_count]; collision_sections_count];
    let mut divisions_by_puuid: HashMap<puuid, [usize; 2]> = HashMap::new();
    for (puuid, p) in chunk.particles.iter() {
        let p = chunk.particles.get(puuid).unwrap();
        let x = if p.x < 0.0 {
            0
        } else if p.x >= 1.0 {
            collision_sections_count - 1
        } else {
            (p.x * collision_sections_count as f64).abs() as usize
        };
        let y = if p.y < 0.0 {
            0
        } else if p.y >= 1.0 {
            collision_sections_count - 1
        } else {
            (p.y * collision_sections_count as f64).abs() as usize
        };
        divisions[x][y].push(*puuid);
        divisions_by_puuid.insert(*puuid, [x, y]);
    }
    for puuid_1 in puuids {
        let division_coords = divisions_by_puuid.get(puuid_1).unwrap();
        let i = division_coords[0];
        let j = division_coords[1];
        let targets_x_start = if i == 0 { 0 } else {i-1};
        let targets_x_end = if i + 2 > collision_sections_count { collision_sections_count } else { i + 2 };
        let targets_y_start = if j == 0 { 0 } else {j-1};
        let targets_y_end = if j + 2 > collision_sections_count { collision_sections_count } else { j + 2 };
        for x in targets_x_start..targets_x_end {
            for y in targets_y_start..targets_y_end {
                let targets = &divisions[x][y];
                for puuid_2 in targets.iter() {
                    if puuid_1 < puuid_2 {
                        let p1 = chunk.particles.get(puuid_1).unwrap();
                        let p2 = chunk.particles.get(puuid_2).unwrap();
                        let collision = detect_collision(p1, p2);
                        if collision {
                            let distance_centers = Point::get_distance(p1.x, p1.y, p2.x, p2.y);
                            let radiuses = (p1.diameter * 0.5) + (p2.diameter * 0.5);
                            let v = Vector::new_2(p1.x, p1.y, p2.x, p2.y);
                            let delta = radiuses - distance_centers;
                            let delta_vector = v.normalized().multiplied(delta);
                            collisions.push(Collision{
                                puuids: [*puuid_1,*puuid_2],
                                delta_vector: delta_vector,
                                collision_x: delta_vector.x * chunk.constants.collision_push_rate
                            });
                        } else {
                            // Do nothing
                        }
                    }
                }
            }
        }
    }
}
fn detect_collision(p1: & Particle, p2: & Particle) -> bool {
    let distance_squared_centers = Point::get_distance_squared(p1.x, p1.y, p2.x, p2.y);
    let radiuses = (p1.diameter * 0.5) + (p2.diameter * 0.5);
    let radiuses_squared = radiuses * radiuses;
    distance_squared_centers < radiuses_squared
}
