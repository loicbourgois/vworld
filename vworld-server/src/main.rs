//#![deny(warnings)]
mod point;
mod vector;
mod particle;
use crate::particle::Particle;
use crate::particle::ParticleData;
use crate::particle::ParticleType;
use crate::particle::add_first_particle;
use crate::particle::add_second_particle;
use crate::particle::add_particle;
use crate::vector::Vector;
use crate::point::Point;
mod entity;
mod chunk;
mod compute;
use crate::compute::ComputeInputData;
use crate::compute::ComputeOutputData;
use crate::chunk::Chunk;
use crate::chunk::Stats;
use crate::entity::Entity;
use crate::entity::EntityType;
use crate::entity::get_next_gene;
use crate::entity::add_new_bloop;
use crate::entity::add_new_plant;
mod constants;
use crate::constants::Constants;
use std::net::TcpListener;
use std::thread;
use tungstenite::server::accept;
use std::env;
use std::time::Duration;
use std::sync::{Arc, RwLock};
use serde_json as json;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime};
use rand::prelude::*;
use ::uuid::Uuid as bob;
use std::collections::HashMap;
use std::collections::HashSet;
use crate::compute::compute;
use num_cpus;
use std::ptr;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
#[allow(non_camel_case_types)]
type uuid = u128;
#[allow(non_camel_case_types)]
type puuid = uuid;
#[allow(non_camel_case_types)]
type euuid = uuid;
#[allow(non_camel_case_types)]
type luuid = uuid;
#[derive(Serialize, Deserialize)]
struct ParticleConfiguration {
    x: f64,
    y: f64,
    type_: ParticleType,
    diameter: f64,
    mass: f64
}
#[derive(Serialize, Deserialize)]
pub struct Link {
    pub puuids: [puuid; 2],
    pub strengh: f64,
    pub puuids_str: [String; 2],
}
#[derive(Serialize, Deserialize)]
struct EntityConfiguration {
    particles: Vec<ParticleConfiguration>,
    links: Vec<Link>
}
#[derive(Clone, Copy, Serialize, Deserialize)]
pub enum ThreadCount {
    #[allow(non_camel_case_types)]
    auto,
    #[allow(non_camel_case_types)]
    value(usize)
}
#[derive(Serialize, Deserialize)]
struct ChunkConfiguration {
    x: u32,
    y: u32,
    entities: Vec<EntityConfiguration>,
    constants: Constants,
    thread_count: ThreadCount,
}
#[derive(Serialize, Deserialize)]
pub struct BestDna {
    pub age_in_ticks: u32,
    pub dna: Vec<f64>,
    pub distance_traveled: f64,
}
#[derive(Serialize, Deserialize)]
pub struct BestDnaStat {
    pub age_in_ticks: u32,
    pub distance_traveled: f64,
}
fn create_chunk_from_configuration_str(configuration_str: &str) -> Chunk {
    let configuration: ChunkConfiguration = json::from_str(&configuration_str).unwrap();
    let mut chunk = Chunk {
        total_energy: 0.0,
        width: 1.0,
        height: 1.0,
        step: 0,
        particles: HashMap::new(),
        links: HashMap::new(),
        x: configuration.x,
        y: configuration.y,
        constants: configuration.constants,
        entities: HashMap::new(),
        real_time_ms: 0,
        particles_count: 0,
        links_count: 0,
        entities_count: 0,
        best_dna_ever_by_age: BestDna {
            age_in_ticks: 0,
            dna: Vec::new(),
            distance_traveled: 0.0,
        },
        best_dna_alive_by_age: BestDna {
            age_in_ticks: 0,
            dna: Vec::new(),
            distance_traveled: 0.0,
        },
        best_dna_ever_by_distance_traveled: BestDna {
            age_in_ticks: 0,
            dna: Vec::new(),
            distance_traveled: 0.0,
        },
        best_dna_alive_by_distance_traveled: BestDna {
            age_in_ticks: 0,
            dna: Vec::new(),
            distance_traveled: 0.0,
        },
        stats: Vec::new(),
        thread_count: match configuration.thread_count {
            ThreadCount::auto => {
                (num_cpus::get() - 2).max(1)
            },
            ThreadCount::value(value) => value
        },
        json: "".to_string(),
    };
    for _ in 0..configuration.constants.bloop.min_count {
        add_new_bloop(&mut chunk)
    }
    for _ in 0..configuration.constants.plant.min_count {
        add_new_plant(&mut chunk, None, None);
    }
    return chunk
}
fn get_free_pairs(entity: &Entity) -> Vec<[puuid; 2]> {
    let mut v = Vec::new();
    for (puuid_a, hashet) in entity.pairs.iter() {
        for puuid_b in hashet {
            match entity.pairs_taken.get(puuid_a) {
                Some(hashset_taken) => {
                    match hashset_taken.get(puuid_b) {
                        Some(_) => (),
                        None => v.push([*puuid_a, *puuid_b])
                    }
                }, None => v.push([*puuid_a, *puuid_b])
            }
        }
    }
    return v;
}
fn main() {
    let address: String = env::var("vworld_address").unwrap();
    let port: String = match env::var("PORT") {
        Ok(port) => {
            println!("using env.PORT");
            port
        },
        Err(error) => {
            println!("error getting env.PORT: {:?}", error);
            println!("using vworld_port instead");
            env::var("vworld_port").unwrap()
        }
    };
    let chunk_configuration_str: String = env::var("vworld_chunk_configuration").unwrap().replace("\\\"", "\"");
    let host = format!("{}:{}", address, port);
    let server = TcpListener::bind(host.to_owned()).unwrap();
    let chunk = create_chunk_from_configuration_str(&chunk_configuration_str);
    let thread_count = chunk.thread_count;
    let chunk_lock = Arc::new(RwLock::new(chunk));
    let mut worker_threads = Vec::new();
    let mut input_channels: Vec<(Sender<ComputeInputData>, Receiver<ComputeInputData>)> = Vec::new();
    let output_channel: (Sender<ComputeOutputData>, Receiver<ComputeOutputData>) = mpsc::channel();
    let output_receiver = output_channel.1;
    for i in 0..thread_count {
        let chunk_clone = Arc::clone(&chunk_lock);
        input_channels.push(mpsc::channel());
        let thread_output_sender = output_channel.0.clone();
        let thread_input_receiver = unsafe { ptr::read(&input_channels[i].1) };
        worker_threads.push(thread::spawn(move || {
            loop {
                let input_data = thread_input_receiver.recv().unwrap();
                let mut particle_updates = HashMap::new();
                compute(&chunk_clone.read().unwrap(), &input_data.puuids, &mut particle_updates);
                let output_data = ComputeOutputData {
                    id: i,
                    particle_updates: particle_updates,
                };
                thread_output_sender.send(output_data).unwrap();
            }
        }));
    }

    //
    let chunk_json = "".to_string();
    let chunk_json_lock = Arc::new(RwLock::new(chunk_json));
    {
        let chunk_lock_clone = Arc::clone(&chunk_lock);
        let chunk_json_lock_clone = Arc::clone(&chunk_json_lock);
        thread::spawn(move || {
            let mut json = "".to_string();
            loop {
                {
                    json = serde_json::to_string(&*chunk_lock_clone.read().unwrap()).unwrap();
                }
                {
                    let mut chunk_json = chunk_json_lock_clone.write().unwrap();
                    *chunk_json = json;
                }
                thread::sleep(Duration::from_millis(50));
            }
        });
    }

    //
    println!("starting server...");
    {
        let chunk_clone = Arc::clone(&chunk_lock);
        thread::spawn(move || {
            let mut rng = rand::thread_rng();
            let start_time = SystemTime::now();
            // Tick loop
            loop {
                // Update chunk
                {
                    let mut chunk = chunk_clone.write().unwrap();
                    chunk.real_time_ms = SystemTime::now().duration_since(start_time).unwrap().as_millis();
                    let delta_time = chunk.constants.delta_time;
                    // Add entities
                    let mut entities_by_type: HashMap<EntityType, i32> = HashMap::new();
                    entities_by_type.insert(EntityType::Plant, 0);
                    entities_by_type.insert(EntityType::Bloop, 0);
                    for entity in chunk.entities.values() {
                        match &entity.type_ {
                            type_ => *entities_by_type.get_mut(&type_).unwrap() += 1
                        }
                    }
                    let a: i32 = (*entities_by_type.get(&EntityType::Plant).unwrap()) as i32;
                    let b: i32 = chunk.constants.plant.min_count as i32;
                    let plant_to_add_count: i32 = (b - a).max(0);
                    //println!("{}   {}     {}", a, b, plant_to_add_count);
                    if plant_to_add_count > 0 {
                        //println!("{}   {}     {}   {}", i, a, b, plant_to_add_count);
                        add_new_plant(&mut chunk, None, None);
                    }
                    let bloop_to_add_count = chunk.constants.bloop.min_count as i32 - entities_by_type.get(&EntityType::Bloop).unwrap();
                    for _ in 0..bloop_to_add_count {
                        add_new_bloop(&mut chunk);
                    }
                    // Update output
                    let mut new_outputs_by_puuid: HashMap<puuid, f64> = HashMap::new();
                    for (puuid, particle) in chunk.particles.iter() {
                        let mut output = particle.bias_weight;
                        let mut divisor = particle.bias_weight.abs();
                        for link in particle.links.values() {
                            output += link.weight * chunk.particles.get(&link.puuid_linked).unwrap().output;
                            divisor += link.weight.abs();
                        }
                        output /= divisor;
                        if output < 0.0 {
                            output = 0.0;
                        }
                        new_outputs_by_puuid.insert(*puuid, output);
                    }
                    let simulation_time_s = chunk.step as f64 * chunk.constants.delta_time;
                    for (puuid, particle) in &mut chunk.particles.iter_mut() {
                        particle.output = match particle.type_ {
                            ParticleType::Heart => {
                                (simulation_time_s * particle.frequency * 10.0 + particle.phase * 10.0).sin() * 0.5 + 0.5
                            },
                            ParticleType::Eye => {
                                if particle.is_colliding_other_entity {
                                    1.0
                                } else {
                                    0.0
                                }
                            }
                            _ => {
                                *new_outputs_by_puuid.get(puuid).unwrap()
                            }
                        };
                        if particle.output < 0.0 || particle.output > 1.0 {
                            println!("particle.output should be in [0.0, 1.0], not '{}'", particle.output)
                        }
                    }
                    // Muscle action
                    let diameter_muscle_change_rate = chunk.constants.diameter_muscle_change_rate;
                    let muscles_use_output = chunk.constants.muscles_use_output;
                    for particle in &mut chunk.particles.values_mut() {
                        match particle.type_ {
                            ParticleType::Muscle => {
                                particle.diameter = if muscles_use_output {
                                    particle.base_diameter * (1.0 - particle.output * 0.5)
                                } else {
                                    let sin = (simulation_time_s * particle.frequency * 10.0 + particle.phase * 10.0).sin();
                                    particle.base_diameter * (sin * 0.25 + 0.75)
                                }
                            },
                            ParticleType::MuscleInverted => {
                                particle.diameter = particle.base_diameter * ( (simulation_time_s * 10.0).sin()*-0.25 + 0.75 );
                            },
                            ParticleType::MuscleRandom => {
                                let delta_d: f64 = (rng.gen::<f64>() - 0.5) * diameter_muscle_change_rate * delta_time;
                                let new_d = particle.diameter + delta_d;
                                particle.diameter = new_d.max(particle.base_diameter*0.5).min(particle.base_diameter);
                            },
                            _ => {
                                // do nothing
                            }
                        }
                    }
                    // Energy
                    let mut energy_delta_by_puuid: HashMap<puuid, f64> = HashMap::new();
                    for puuid in chunk.particles.keys() {
                        energy_delta_by_puuid.insert(*puuid, 0.0);
                    }
                    let bloop_energy_drop_rate_per_tick = chunk.constants.bloop.energy_drop_rate_per_tick;
                    let plant_energy_drop_rate_per_tick = chunk.constants.plant.energy_drop_rate_per_tick;
                    let plant_energy_drop_rate_per_tick_circle = chunk.constants.plant.energy_drop_rate_per_tick_circle;
                    let energy_min = chunk.constants.energy_min;
                    let mut puuid_pairs: Vec<[puuid; 2]> = Vec::new();
                    for link in chunk.links.values() {
                        puuid_pairs.push(link.puuids);
                    }
                    // Energy transfer
                    for puuid_pair in puuid_pairs.iter() {
                        let puuid_a = puuid_pair[0];
                        let puuid_b = puuid_pair[1];
                        let energy = {
                            let p1 = chunk.particles.get(&puuid_a).unwrap();
                            let p2 = chunk.particles.get(&puuid_b).unwrap();
                            (p1.energy + p2.energy) * 0.5
                        };
                        chunk.particles.get_mut(&puuid_a).unwrap().energy = energy;
                        chunk.particles.get_mut(&puuid_b).unwrap().energy = energy;
                    }
                    // Energy drop
                    // Adjust plant size
                    let energy_max = chunk.constants.energy_max;
                    for particle in &mut chunk.particles.values_mut() {
                        match particle.type_ {
                            ParticleType::Plant => {
                                let plant_drop_rate = plant_energy_drop_rate_per_tick
                                    + plant_energy_drop_rate_per_tick_circle * Point::get_distance(particle.x, particle.y, 0.5, 0.5);
                                particle.energy -= plant_drop_rate;
                                if particle.x < 0.0 || particle.x > 1.0 || particle.y < 0.0 || particle.y > 1.0 {
                                    particle.energy = -1.0;
                                }
                                particle.diameter = particle.energy.max(0.000001) * particle.base_diameter;
                            },
                            _ => {
                                particle.energy -= bloop_energy_drop_rate_per_tick
                            }
                        }
                        particle.energy = particle.energy.min(energy_max)
                    }
                    // Kill
                    let mut entities_to_remove: HashSet<euuid> = HashSet::new();
                    let mut particles_to_remove: HashSet<puuid> = HashSet::new();
                    for (puuid, particle) in  chunk.particles.iter() {
                        // Kill particle
                        if particle.energy <= energy_min {
                            particles_to_remove.insert(*puuid);
                            match particle.type_ {
                                ParticleType::Heart => {
                                    for puuid_2 in chunk.entities.get(&particle.euuid).unwrap().puuids.iter() {
                                        particles_to_remove.insert(*puuid_2);
                                    }
                                }
                                _ => ()
                            }
                        }
                        if chunk.constants.destroy_unstable_entities == true && particle.links.len() > 6 {
                            entities_to_remove.insert(particle.euuid);
                            println!("destroy_unstable_entities");
                        }
                    }
                    // remove pairs
                    // TODO ?
                    // remove trinomes
                    // TODO ?
                    // Best dna
                    let mut new_best_dna_ever_by_age = BestDna {
                        age_in_ticks: chunk.best_dna_ever_by_age.age_in_ticks,
                        distance_traveled: chunk.best_dna_ever_by_age.distance_traveled,
                        dna: chunk.best_dna_ever_by_age.dna.to_vec(),
                    };
                    let mut new_best_dna_alive_by_age = BestDna {
                        age_in_ticks: 0,
                        distance_traveled: 0.0,
                        dna: Vec::new(),
                    };
                    let mut new_best_dna_ever_by_distance_traveled = BestDna {
                        age_in_ticks: chunk.best_dna_ever_by_distance_traveled.age_in_ticks,
                        distance_traveled: chunk.best_dna_ever_by_distance_traveled.distance_traveled,
                        dna: chunk.best_dna_ever_by_distance_traveled.dna.to_vec(),
                    };
                    let mut new_best_dna_alive_by_distance_traveled = BestDna {
                        age_in_ticks: 0,
                        distance_traveled: 0.0,
                        dna: Vec::new(),
                    };
                    for entity in chunk.entities.values() {
                        match entity.type_ {
                            EntityType::Plant => {
                                // Do nothing
                            },
                            EntityType::Bloop => {
                                let age_in_ticks = chunk.step - entity.tick_start;
                                let distance_traveled = Vector::new_2(
                                        entity.x_start, entity.y_start, entity.x, entity.y
                                    ).length();
                                if age_in_ticks > new_best_dna_ever_by_age.age_in_ticks {
                                    new_best_dna_ever_by_age = BestDna {
                                        age_in_ticks: age_in_ticks,
                                        distance_traveled: distance_traveled,
                                        dna: entity.dna.to_vec(),
                                    }
                                }
                                if age_in_ticks > new_best_dna_alive_by_age.age_in_ticks {
                                    new_best_dna_alive_by_age = BestDna {
                                        age_in_ticks: age_in_ticks,
                                        distance_traveled: distance_traveled,
                                        dna: entity.dna.to_vec(),
                                    }
                                }
                                if distance_traveled > new_best_dna_ever_by_distance_traveled.distance_traveled {
                                    new_best_dna_ever_by_distance_traveled = BestDna {
                                        age_in_ticks: age_in_ticks,
                                        distance_traveled: distance_traveled,
                                        dna: entity.dna.to_vec(),
                                    }
                                }
                                if distance_traveled > new_best_dna_alive_by_distance_traveled.distance_traveled {
                                    new_best_dna_alive_by_distance_traveled = BestDna {
                                        age_in_ticks: age_in_ticks,
                                        distance_traveled: distance_traveled,
                                        dna: entity.dna.to_vec(),
                                    }
                                }
                            }
                        }
                    }
                    chunk.best_dna_ever_by_age.age_in_ticks = new_best_dna_ever_by_age.age_in_ticks;
                    chunk.best_dna_ever_by_age.distance_traveled = new_best_dna_ever_by_age.distance_traveled.max(0.0);
                    chunk.best_dna_ever_by_age.dna = new_best_dna_ever_by_age.dna.to_vec();
                    chunk.best_dna_alive_by_age.age_in_ticks = new_best_dna_alive_by_age.age_in_ticks;
                    chunk.best_dna_alive_by_age.distance_traveled = new_best_dna_alive_by_age.distance_traveled.max(0.0);
                    chunk.best_dna_alive_by_age.dna = new_best_dna_alive_by_age.dna.to_vec();
                    chunk.best_dna_ever_by_distance_traveled.age_in_ticks = new_best_dna_ever_by_distance_traveled.age_in_ticks;
                    chunk.best_dna_ever_by_distance_traveled.distance_traveled = new_best_dna_ever_by_distance_traveled.distance_traveled.max(0.0);
                    chunk.best_dna_ever_by_distance_traveled.dna = new_best_dna_ever_by_distance_traveled.dna.to_vec();
                    chunk.best_dna_alive_by_distance_traveled.age_in_ticks = new_best_dna_alive_by_distance_traveled.age_in_ticks;
                    chunk.best_dna_alive_by_distance_traveled.distance_traveled = new_best_dna_alive_by_distance_traveled.distance_traveled.max(0.0);
                    chunk.best_dna_alive_by_distance_traveled.dna = new_best_dna_alive_by_distance_traveled.dna.to_vec();
                    // Remove entities
                    for (euuid, entity) in chunk.entities.iter() {
                        if entity.puuids.len() == 0 {
                            entities_to_remove.insert(*euuid);
                        }
                    }
                    for euuid in entities_to_remove.iter() {
                        for puuid in chunk.entities.get(euuid).unwrap().puuids.iter() {
                            particles_to_remove.insert(*puuid);
                        }
                    }
                    // Remove particle
                    let mut links_to_remove: HashSet<luuid> = HashSet::new();
                    let mut particle_links_to_remove: HashMap<puuid, HashSet<luuid>> = HashMap::new();
                    for puuid in particles_to_remove.iter() {
                        let mut links_to_remove_tmp: HashSet<luuid> = HashSet::new();
                        for luuid in chunk.particles.get(puuid).unwrap().links.keys() {
                            links_to_remove.insert(*luuid);
                            links_to_remove_tmp.insert(*luuid);
                        }
                        particle_links_to_remove.insert(*puuid, links_to_remove_tmp);
                        let euuid = chunk.particles.get(puuid).unwrap().euuid;
                        chunk.entities.get_mut(&euuid).unwrap().puuids.remove(puuid);
                    }
                    // Remove links
                    for luuid in links_to_remove.iter() {
                        for particle in chunk.particles.values_mut() {
                            particle.links.remove(luuid);
                        }
                        chunk.links.remove(&luuid);
                    }
                    // Remove particles
                    for puuid in particles_to_remove.iter() {
                        chunk.particles.remove(puuid);
                    }
                    // Remove entities
                    for euuid in entities_to_remove.iter() {
                        chunk.entities.remove(euuid);
                    }
                    // Add link forces
                    let mut forces_by_puuid: HashMap<puuid, Vector> = HashMap::new();
                    for puuid in chunk.particles.keys() {
                        forces_by_puuid.insert(*puuid, Vector::new(&Point{x:0.0, y:0.0}, &Point{x:0.0, y:0.0}));
                    }
                    for link in chunk.links.values() {
                        let p1 = &chunk.particles[&link.puuids[0]];
                        let p2 = &chunk.particles[&link.puuids[1]];
                        let length = (p1.diameter + p2.diameter) * 0.5 * chunk.constants.link_length_coefficient;
                        let force = get_link_force(p1, p2, length, link.strengh);
                        forces_by_puuid.get_mut(&link.puuids[0]).unwrap().add(&force);
                        forces_by_puuid.get_mut(&link.puuids[1]).unwrap().remove(&force);
                    }
                    // Add other forces
                    for (puuid, particle) in chunk.particles.iter() {
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
                    }
                    // Update positions from forces
                    for (puuid, force) in forces_by_puuid.iter() {
                        update_position_verlet(&mut chunk.particles.get_mut(puuid).unwrap(), &force, delta_time);
                    }

                }
                // Compute
                let mut outputs = Vec::new();
                {
                    let mut chunk = chunk_clone.read().unwrap();
                    let mut puuids_by_thread: Vec<Vec<puuid>> = Vec::new();
                    for i in 0..thread_count {
                        puuids_by_thread.push(Vec::new());
                    }
                    let mut i = 0;
                    for puuid in chunk.particles.keys() {
                        puuids_by_thread[i%thread_count].push(*puuid);
                        i += 1;
                    }
                    for i in 0..thread_count {
                        input_channels[i].0.send(ComputeInputData{
                            id: i,
                            puuids: puuids_by_thread[i].to_vec(),
                        }).unwrap();
                    }
                    for _ in 0..thread_count {
                        outputs.push(output_receiver.recv().unwrap());
                    }
                }
                {
                    let mut chunk = chunk_clone.write().unwrap();
                    // Treat collisions
                    //let mut collision_count = 0;
                    //for output in &outputs {
                    //    let collisions = &output.collisions;
                    //    collision_count += collisions.len();

                    //}
                    for output in &outputs {
                        for (puuid, particle_update) in &output.particle_updates {
                            let p = chunk.particles.get_mut(puuid).unwrap();
                            p.x += particle_update.x;
                            p.y += particle_update.y;
                            p.x_old += particle_update.x_old;
                            p.y_old += particle_update.y_old;
                            p.energy += particle_update.energy;
                            p.is_colliding_other_entity = particle_update.is_colliding_other_entity || p.is_colliding_other_entity;
                        }
                    }



                    // Update entity position
                    let mut entities_coord: HashMap<euuid, (f64, f64)> = HashMap::new();
                    for particle in chunk.particles.values() {
                        match particle.type_ {
                            ParticleType::Heart => {
                                entities_coord.insert(particle.euuid, (particle.x, particle.y));
                            },
                            ParticleType::Plant => {
                                entities_coord.insert(particle.euuid, (particle.x, particle.y));
                            },
                            _ => ()
                        }
                    }
                    for (euuid, coord) in entities_coord.iter() {
                        chunk.entities.get_mut(euuid).unwrap().x = coord.0;
                        chunk.entities.get_mut(euuid).unwrap().y = coord.1;
                    }
                    // Update cells data
                    let mut cells_data: HashMap<puuid, ParticleData> = HashMap::new();
                    for particle_a in chunk.particles.values() {
                        match particle_a.type_ {
                            ParticleType::Eye => {
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
                                let direction_normalized = direction.normalized();
                                // todo: check intersections with cells for eyesight
                                cells_data.insert(particle_a.puuid, ParticleData::EyeData{
                                    direction: direction_normalized,
                                    // color:
                                });
                            },
                            ParticleType::Mouth => {
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
                                let direction_normalized = direction.normalized();
                                cells_data.insert(particle_a.puuid, ParticleData::MouthData{
                                    direction: direction_normalized,
                                });

                            }, _ => ()
                        }
                    }
                    for (puuid, data) in cells_data.iter() {
                        match data {
                            ParticleData::EyeData {direction} => {
                                chunk.particles.get_mut(puuid).unwrap().data = ParticleData::EyeData{
                                    direction: *direction,
                                }
                            },
                            ParticleData::MouthData {direction} => {
                                chunk.particles.get_mut(puuid).unwrap().data = ParticleData::MouthData{
                                    direction: *direction,
                                }
                            },
                            _ => {},
                        }
                    }
                    // Stats
                    let real_time_s = chunk.real_time_ms as f64 * 0.001;
                    chunk.particles_count = chunk.particles.len() as u32;
                    chunk.links_count = chunk.links.len() as u32;
                    chunk.entities_count = chunk.entities.len() as u32;
                    let mut total_energy = 0.0;
                    for particle in chunk.particles.values() {
                        total_energy += particle.energy;
                    }
                    chunk.total_energy = total_energy;
                    let simulation_time_s = chunk.step as f64 * chunk.constants.delta_time;
                    if chunk.step % 100 == 0 {
                        let l = chunk.stats.len();
                        let mut real_time_s_delta = 0.0;
                        let mut simulation_time_s_delta = 0.0;
                        let mut steps_delta = 0;
                        if l >= 1 {
                            real_time_s_delta = real_time_s - chunk.stats[l-1].real_time_s;
                            simulation_time_s_delta = simulation_time_s - chunk.stats[l-1].simulation_time_s;
                            steps_delta = chunk.step - chunk.stats[l-1].step;
                        }
                        let current_steps_per_second = steps_delta as f64 / real_time_s_delta;
                        let steps_per_second = chunk.step as f64 / real_time_s;
                        let current_simulation_speed = simulation_time_s_delta / real_time_s_delta;
                        let stats = Stats {
                            step: chunk.step,
                            best_dna_ever_by_age: BestDnaStat {
                                age_in_ticks: chunk.best_dna_ever_by_age.age_in_ticks,
                                //dna: chunk.best_dna_ever_by_age.dna.to_vec(),
                                distance_traveled: chunk.best_dna_ever_by_age.distance_traveled,
                            },
                            best_dna_alive_by_age: BestDnaStat {
                                age_in_ticks: chunk.best_dna_alive_by_age.age_in_ticks,
                                //dna: chunk.best_dna_alive_by_age.dna.to_vec(),
                                distance_traveled: chunk.best_dna_alive_by_age.distance_traveled,
                            },
                            best_dna_ever_by_distance_traveled: BestDnaStat {
                                age_in_ticks: chunk.best_dna_ever_by_distance_traveled.age_in_ticks,
                                //dna: chunk.best_dna_ever_by_distance_traveled.dna.to_vec(),
                                distance_traveled: chunk.best_dna_ever_by_distance_traveled.distance_traveled,
                            },
                            best_dna_alive_by_distance_traveled: BestDnaStat {
                                age_in_ticks: chunk.best_dna_alive_by_distance_traveled.age_in_ticks,
                                //dna: chunk.best_dna_alive_by_distance_traveled.dna.to_vec(),
                                distance_traveled: chunk.best_dna_alive_by_distance_traveled.distance_traveled,
                            },
                            real_time_s: real_time_s,
                            simulation_time_s: simulation_time_s,
                            steps_per_second: steps_per_second,
                            simulation_speed: current_simulation_speed,
                        };
                        chunk.stats.push(stats);
                        let stats_to_drop_count: i32 = chunk.stats.len() as i32 - chunk.constants.max_stats_count;
                        for _ in 0..stats_to_drop_count {
                            let i = rng.gen::<f64>() * (chunk.stats.len() - 2) as f64 + 1.0;
                            chunk.stats.remove(i as usize);
                        }
                        if chunk.constants.display_simulation_logs {
                            let simulation_speed = simulation_time_s / real_time_s;
                            println!("step #{}", chunk.step);
                            println!("  average:              ");
                            println!("    steps / second:   {}", steps_per_second);
                            println!("    simulation_speed: {}", simulation_speed);
                            println!("  current:              ");
                            println!("    steps / second:   {}", current_steps_per_second);
                            println!("    simulation_speed: {}", current_simulation_speed);
                            // println!("  collisions:         {}", collision_count);
                            println!("  entities:           {}", chunk.entities.len());
                            println!("  particles:          {}", chunk.particles.len());
                            println!("  energy:             {}", chunk.total_energy);
                            println!("  best dna ever by age:        ");
                            println!("    age:              {}", chunk.best_dna_ever_by_age.age_in_ticks);
                            println!("    distance:         {}", chunk.best_dna_ever_by_age.distance_traveled);
                            println!("  best dna alive by age:       ");
                            println!("    age:              {}", chunk.best_dna_alive_by_age.age_in_ticks);
                            println!("    distance:         {}", chunk.best_dna_alive_by_age.distance_traveled);
                            println!("  stats:              {}", chunk.stats.len());
                        }
                    }
                    chunk.step += 1;
                }
            }
        });
    }
    println!("server started");
    println!("  host:           {}", host);
    println!("  engine threads: {}", thread_count);
    println!("  configuration:  {}", chunk_configuration_str);
    for stream in server.incoming() {
        println!("incoming");
        let chunk_clone = Arc::clone(&chunk_lock);
        let chunk_json_clone = Arc::clone(&chunk_json_lock);
        thread::spawn (move || {
            let mut websocket = accept(stream.unwrap()).unwrap();
            loop {
                let msg = websocket.read_message().unwrap();
                if msg.is_binary() || msg.is_text() {
                    println!("message: {}", msg);
                    if msg == tungstenite::Message::Text("Hello Server!".to_string()) {
                        loop {
                            {
                                let message = tungstenite::Message::Text(format!("{}", *chunk_json_clone.read().unwrap()));
                                websocket.write_message(message).unwrap();
                            }
                            thread::sleep(Duration::from_millis(50));
                        }
                    } else if msg == tungstenite::Message::Text("use_distance_traveled_as_fitness_function".to_string()) {
                        {
                            let mut chunk = chunk_clone.write().unwrap();
                            chunk.constants.use_distance_traveled_as_fitness_function = true;
                        }
                    } else if msg == tungstenite::Message::Text("use_distance_traveled_as_fitness_function_false".to_string()) {
                        {
                            let mut chunk = chunk_clone.write().unwrap();
                            chunk.constants.use_distance_traveled_as_fitness_function = false;
                        }
                    }
                }
            }
        });
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
fn update_position_verlet(p: &mut Particle, forces: &Vector, delta_time: f64) {
    let current_x = p.x;
    let current_y = p.y;
    let acceleration_x = forces.x / p.mass;
    let acceleration_y = forces.y / p.mass;
    p.x = 2.0 * current_x - p.x_old + acceleration_x * delta_time * delta_time;
    p.y = 2.0 * current_y - p.y_old + acceleration_y * delta_time * delta_time;
    p.x_old = current_x;
    p.y_old = current_y;
}
