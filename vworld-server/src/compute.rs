use crate::puuid;
use crate::luuid;
use std::collections::HashMap;
use crate::Point;
use crate::Vector;
use crate::Chunk;
use crate::Particle;
use rand::prelude::*;
use crate::particle::ParticleType;
use crate::particle::ParticleData;
use crate::client::VisionData;
//
// A segment is defined by two pair of xy coordinates
//
#[derive(Copy, Clone)]
pub struct Segment {
    pub x1: f64,
    pub y1: f64,
    pub x2: f64,
    pub y2: f64
}
#[derive(Copy, Clone)]
pub struct Collision {
    pub puuids: [puuid; 2],
    pub delta_vector: Vector,
    pub collision_x: f64,
}
pub struct ComputeInputData {
    pub id: usize,
    pub puuids: Vec<puuid>,
    pub luuids: Vec<luuid>,
}
pub struct SingleParticleUpdate {
    pub output: f64,
    pub diameter: f64,
    pub particle_data: Option<ParticleData>,
}
pub struct MultipleParticleUpdate {
    pub x: f64,
    pub y: f64,
    pub x_old: f64,
    pub y_old: f64,
    pub energy: f64,
    pub is_colliding_other_entity: bool,
}
pub struct ComputeOutputData {
    pub id: usize,
    pub multiple_particle_updates: HashMap<puuid, MultipleParticleUpdate>,
    pub single_particle_updates: HashMap<puuid, SingleParticleUpdate>,
    pub vision_data: Vec<VisionData>,
}
pub fn compute(
    _id: usize,
    chunk: &Chunk,
    puuids: &Vec<puuid>,
    luuids: &Vec<luuid>,
    single_particle_updates: &mut HashMap<puuid, SingleParticleUpdate>,
    mut multiple_particle_updates: &mut HashMap<puuid, MultipleParticleUpdate>,
    vision_data: &mut Vec<VisionData>,
) {
    let mut collisions: Vec<Collision> = Vec::new();
    let mut forces_by_puuid: HashMap<puuid, Vector> = HashMap::new();
    reset_single_particle_updates(puuids, single_particle_updates);
    reset_multiple_particle_updates(chunk, multiple_particle_updates);
    compute_collisions(chunk, puuids, &mut collisions);
    update_from_collisions(chunk, &collisions, &mut multiple_particle_updates);
    update_outputs(chunk, single_particle_updates, vision_data);
    setup_forces(&mut forces_by_puuid, chunk);
    compute_forces(&mut forces_by_puuid, chunk, puuids, luuids);
    update_from_forces(&forces_by_puuid, &mut multiple_particle_updates, chunk);
    update_diameter(puuids, single_particle_updates, chunk);
    update_particle_data(puuids, single_particle_updates, chunk);
}
fn reset_single_particle_updates (
    puuids: &Vec<puuid>,
    single_particle_updates: &mut HashMap<puuid, SingleParticleUpdate>
) {
    for puuid in puuids {
        single_particle_updates.insert(*puuid, SingleParticleUpdate {
            output: 0.0,
            diameter: 0.0,
            particle_data: None,
        });
    }
}
fn reset_multiple_particle_updates (
    chunk: &Chunk,
    multiple_particle_updates: &mut HashMap<puuid, MultipleParticleUpdate>
) {
    for puuid in chunk.particles.keys() {
        multiple_particle_updates.insert(*puuid, MultipleParticleUpdate {
            x: 0.0,
            y: 0.0,
            x_old: 0.0,
            y_old: 0.0,
            energy: 0.0,
            is_colliding_other_entity: false,
        });
    }
}
fn get_direction(chunk: &Chunk, particle_a: &Particle) -> Vector {
    let mut direction = Vector {
        x: 0.0,
        y: 0.0,
    };
    for particle_link in particle_a.links.values() {
        let particle_b = chunk.particles.get(&particle_link.puuid_linked).unwrap();
        let direction_tmp = Vector::new_2(
            particle_b.x, particle_b.y,
            particle_a.x, particle_a.y
        );
        direction.add(&direction_tmp.normalized());
    }
    return direction.normalized();
}
fn update_particle_data(
    puuids: &Vec<puuid>,
    single_particle_updates: &mut HashMap<puuid, SingleParticleUpdate>,
    chunk: &Chunk,
) {
    for puuid in puuids {
        let particle_a = chunk.particles.get(puuid).unwrap();
        match particle_a.type_ {
            ParticleType::Eye => {
                single_particle_updates.get_mut(puuid).unwrap().particle_data = Some(ParticleData::EyeData{
                    direction: get_direction(chunk, particle_a)
                });
            },
            ParticleType::Mouth => {
                single_particle_updates.get_mut(puuid).unwrap().particle_data = Some(ParticleData::MouthData{
                    direction: get_direction(chunk, particle_a)
                });
            },
            ParticleType::Turbo => {
                single_particle_updates.get_mut(puuid).unwrap().particle_data = Some(ParticleData::TurboData{
                    direction: get_direction(chunk, particle_a)
                });
            },
            _ => ()
        }
    }
}
fn update_diameter(
    puuids: &Vec<puuid>,
    single_particle_updates: &mut HashMap<puuid, SingleParticleUpdate>,
    chunk: &Chunk,
) {
    let mut rng = rand::thread_rng();
    let simulation_time_s = chunk.step as f64 * chunk.constants.delta_time;
    let diameter_muscle_change_rate = chunk.constants.diameter_muscle_change_rate;
    let muscles_use_output = chunk.constants.muscles_use_output;
    let max_frequency: f64 = 10.0;//chunk.constants.max_frequency;
    for puuid in puuids {
        let spu = single_particle_updates.get_mut(puuid).unwrap();
        let particle = chunk.particles.get(puuid).unwrap();
        match particle.type_ {
            ParticleType::Muscle => {
                spu.diameter = if muscles_use_output {
                    particle.base_diameter * (1.0 - particle.output * 0.5)
                } else {
                    let sin_0_1 = (simulation_time_s * particle.frequency * max_frequency + particle.phase * 2.0 * std::f64::consts::PI).sin() * 0.5 + 0.5;
                    particle.base_diameter * (1.0 - sin_0_1 * particle.max_contraction)
                }
            },
            ParticleType::MuscleInverted => {
                spu.diameter = particle.base_diameter * ( (simulation_time_s * 10.0).sin()*-0.25 + 0.75 );
            },
            ParticleType::MuscleRandom => {
                let delta_d: f64 = (rng.gen::<f64>() - 0.5) * diameter_muscle_change_rate * chunk.constants.delta_time;
                let new_d = particle.diameter + delta_d;
                spu.diameter = new_d.max(particle.base_diameter*0.5).min(particle.base_diameter);
            },
            ParticleType::Plant => {
                spu.diameter = particle.energy.max(0.000001) * particle.base_diameter;
            },
            _ => {
                spu.diameter = particle.base_diameter
            }
        }
    }
}
fn get_eye_output (
    chunk: &Chunk,
    puuid: &puuid
) -> Option<(f64, VisionData)> {
    let p = chunk.particles.get(puuid).unwrap();
    let direction = match p.data {
        ParticleData::EyeData {direction} => {
            direction
        },
        _ => {
            //println!("this is not an eye");
            //Vector {x: 0.0, y: 0.0}
            return None
        },
    };
    let p_center = Point{x:p.x, y:p.y};
    let mut puuid_target = 0;
    let segment = Segment {
        x1: p.x,
        y1: p.y,
        x2: p.x + direction.x * chunk.constants.eye_sight_length,
        y2: p.y + direction.y * chunk.constants.eye_sight_length
    };
    let mut best_ratio_distance_to_intersection = 2.0;
    let mut inter_point = Point{x:0.0, y:0.0};
    for (puuid_b, p_b) in &chunk.particles {
        if p_b.euuid != p.euuid {
            let circle_center = Point {
                x: p_b.x,
                y: p_b.y
            };
            let circle_radius = p_b.diameter * 0.5;
            let intersection = approximate_segment_circle_intersection(&segment, circle_center, circle_radius);
            match intersection {
                Some(intersection_point) => {
                    let ratio_distance_to_intersection = Point::get_distance_2(&p_center, &intersection_point) / chunk.constants.eye_sight_length;
                    if ratio_distance_to_intersection < best_ratio_distance_to_intersection {
                        best_ratio_distance_to_intersection = ratio_distance_to_intersection;
                        inter_point = intersection_point;
                        puuid_target = *puuid_b;
                    }
                },
                None => {}
            }
        }
    }
    if best_ratio_distance_to_intersection > 1.0 {
        return None;
    } else {
        return Some((1.0-best_ratio_distance_to_intersection, VisionData {
            puuid_eye: *puuid,
            puuid_target: puuid_target,
            origin: p_center,
            target: inter_point,
        }));
    }
}
//
// Returns an approximation of the intersection between a segment and a circle.
//
fn approximate_segment_circle_intersection(
        segment: & Segment,
        circle_center: Point,
        circle_radius: f64
) -> Option<Point> {
    let closest_point_option = get_closest_point_on_segment(&circle_center, &segment);
    match closest_point_option {
        Some(closest_point) => {
            let distance = Point::get_distance_2(&closest_point, &circle_center);
            if distance <= circle_radius {
                Some(closest_point)
            } else {
                None
            }
        },
        None => {
            None
        }
    }
}
//
// Returns the point closest to point p.
// This point belongs to the segment s.
//
fn get_closest_point_on_segment(p: & Point, s: & Segment) -> Option<Point> {
    let x_delta = s.x2 - s.x1;
    let y_delta = s.y2 - s.y1;
    if x_delta == 0.0 && y_delta == 0.0 {
        return None;
    } else {
        // Do nothing
    }
    let u = ((p.x - s.x1) * x_delta + (p.y - s.y1) * y_delta) / (x_delta * x_delta + y_delta * y_delta);
    let closest_point = if u < 0.0 {
        Point {
            x: s.x1,
            y: s.y1
        }
    } else if u > 1.0 {
        Point {
            x: s.x2,
            y: s.y2
        }
    } else {
        Point {
            x: s.x1 + u * x_delta,
            y: s.y1 + u * y_delta
        }
    };
    return Some(closest_point);
}
fn update_outputs (
    chunk: &Chunk,
    single_particle_updates: &mut HashMap<puuid, SingleParticleUpdate>,
    vision_data: &mut Vec<VisionData>
) {
    let simulation_time_s = chunk.step as f64 * chunk.constants.delta_time;
    // TODO: refactor using constants
    let max_frequency = 10.0;
    for (puuid, spu) in single_particle_updates.iter_mut() {
        let particle = chunk.particles.get(puuid).unwrap();
        spu.output = match particle.type_ {
            ParticleType::Heart => {
                1.0
            },
            ParticleType::Clock => {
                (simulation_time_s * particle.frequency * max_frequency + particle.phase * 2.0 * std::f64::consts::PI).sin() * 0.5 + 0.5
            },
            ParticleType::Constant => {
                particle.bias_weight
            },
            ParticleType::Stomach => {
                1.0 - particle.energy
            },
            ParticleType::Eye => {
                match get_eye_output(chunk, puuid) {
                    Some((output, vision_data_point)) => {
                        vision_data.push(vision_data_point);
                        output
                    },
                    None => {
                        0.0
                    }
                }
            },
            ParticleType::Mouth => {
                if particle.is_colliding_other_entity {
                    1.0
                } else {
                    0.0
                }
            },
            _ => {
                let mut output = 0.0;
                let mut divisor = 0.0;
                for link in particle.links.values() {
                    output += link.weight * chunk.particles.get(&link.puuid_linked).unwrap().output;
                    divisor += link.weight.abs();
                }
                output /= divisor;
                if output < 0.0 {
                    output = 0.0;
                }
                output
            }
        };
        if spu.output > 1.0 || spu.output < 0.0 {
            println!("bad output: {}", spu.output);
        }
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
fn update_from_collisions(
    chunk: &Chunk,
    collisions: &Vec<Collision>,
    multiple_particle_updates: &mut HashMap<puuid, MultipleParticleUpdate>
) {
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
        let p1 = multiple_particle_updates.get_mut(&collision.puuids[0]).unwrap();
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
        let p2 = multiple_particle_updates.get_mut(&collision.puuids[1]).unwrap();
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
fn detect_collision(p1: & Particle, p2: & Particle) -> bool {
    let distance_squared_centers = Point::get_distance_squared(p1.x, p1.y, p2.x, p2.y);
    let radiuses = (p1.diameter * 0.5) + (p2.diameter * 0.5);
    let radiuses_squared = radiuses * radiuses;
    distance_squared_centers < radiuses_squared
}
fn setup_forces(
    forces_by_puuid: &mut HashMap<puuid, Vector>,
    chunk: &Chunk,
) {
    for puuid in chunk.particles.keys() {
        forces_by_puuid.insert(*puuid, Vector::new(&Point{x:0.0, y:0.0}, &Point{x:0.0, y:0.0}));
    }
}
fn compute_forces(
    forces_by_puuid: &mut HashMap<puuid, Vector>,
    chunk: &Chunk,
    puuids: &Vec<puuid>,
    luuids: &Vec<luuid>,
) {
    let delta_time = chunk.constants.delta_time;
    for luuid in luuids {
        let link = chunk.links.get(luuid).unwrap();
        let puuid_a = &link.puuids[0];
        let puuid_b = &link.puuids[1];
        let p1 = &chunk.particles[puuid_a];
        let p2 = &chunk.particles[puuid_b];
        let length = (p1.diameter + p2.diameter) * 0.5 * chunk.constants.link_length_coefficient;
        let force = get_link_force(p1, p2, length, link.strengh);
        forces_by_puuid.get_mut(puuid_a).unwrap().add(&force);
        forces_by_puuid.get_mut(puuid_b).unwrap().remove(&force);
    }
    for puuid in puuids {
        let particle = chunk.particles.get(puuid).unwrap();
        let force_by_puuid = forces_by_puuid.get_mut(puuid).unwrap();
        // Drag force
        let speed_x = (particle.x - particle.x_old ) / delta_time;
        let speed_y = (particle.y - particle.y_old ) / delta_time;
        let drag_force = Vector {
            x: chunk.constants.drag_coefficient * speed_x * speed_x.abs(),
            y: chunk.constants.drag_coefficient * speed_y * speed_y.abs()
        };
        force_by_puuid.remove(&drag_force);
        // Main gravity force
        let gravity_force = Vector {
            x: chunk.constants.gravity.x * particle.mass * delta_time,
            y: chunk.constants.gravity.y * particle.mass * delta_time
        };
        force_by_puuid.add(&gravity_force);
        // Turbo force
        match particle.type_ {
            ParticleType::Turbo => {
                let direction = match particle.data {
                    ParticleData::TurboData {direction} => {
                        direction
                    },
                    _ => {
                        Vector {x: 0.0, y: 0.0}
                    },
                };
                let turbo_force = Vector {
                    x: - delta_time * particle.output * direction.x * chunk.constants.turbo_max,
                    y: - delta_time * particle.output * direction.y * chunk.constants.turbo_max
                };
                force_by_puuid.add(&turbo_force);
            }, _ => ()
        }
    }
}
fn update_from_forces(
    forces_by_puuid: &HashMap<puuid, Vector>,
    multiple_particle_updates: &mut HashMap<puuid, MultipleParticleUpdate>,
    chunk: &Chunk,
) {
    let delta_time = chunk.constants.delta_time;
    for (puuid, forces) in forces_by_puuid {
        let mpu = multiple_particle_updates.get_mut(puuid).unwrap();
        let p = chunk.particles.get(puuid).unwrap();
        let acceleration_x = forces.x / p.mass;
        let acceleration_y = forces.y / p.mass;
        mpu.x += acceleration_x * delta_time * delta_time;
        mpu.y += acceleration_y * delta_time * delta_time;
        //mpu.x_old += p.x - p.x_old;
        //mpu.y_old += p.y - p.y_old;
    }
}
fn get_link_force(p1: &Particle, p2: &Particle, length: f64, strengh: f64) -> Vector {
    let x1 = p1.x;
    let y1 = p1.y;
    let x2 = p2.x;
    let y2 = p2.y;
    let delta_length = Point::get_distance(x1, y1, x2, y2) - length;
    let unit_vector_option = Vector::get_normalized_vector(
        x1, y1,
        x2, y2
    );
    let a = if delta_length < 0.0 {
        0.000001
    } else {
        1.0
    };
    let force_x;
    let force_y;
    match unit_vector_option {
        Some(unit_vector) => {
            force_x = unit_vector.x * delta_length * strengh * a;
            force_y = unit_vector.y * delta_length * strengh * a;
        },
        None => {
            force_x = 0.0;
            force_y = 0.0;
        }
    }
    Vector {
        x: force_x,
        y: force_y
    }
}
