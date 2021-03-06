#![deny(warnings)]
mod point;
mod vector;
mod particle;
mod stat;
use crate::particle::Particle;
use crate::particle::ParticleData;
use crate::particle::ParticleType;
use crate::particle::add_first_particle;
use crate::particle::add_second_particle;
use crate::particle::add_particle;
use crate::vector::Vector;
use crate::point::Point;
use crate::entity::add_new_bloop_from_dna_at;
mod entity;
mod chunk;
mod compute;
use crate::particle::get_age;
mod client;
use crate::client::VisionData;
use crate::particle::Color;
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
use serde::{Deserialize, Serialize};
use std::time::{SystemTime};
use rand::prelude::*;
use ::uuid::Uuid as bob;
use std::collections::HashMap;
use std::collections::HashSet;
use crate::compute::compute;
use std::ptr;
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use crate::chunk::create_chunk_from_configuration_str;
#[allow(non_camel_case_types)]
type uuid = u128;
#[allow(non_camel_case_types)]
type puuid = uuid;
#[allow(non_camel_case_types)]
type euuid = uuid;
#[allow(non_camel_case_types)]
type luuid = uuid;
#[derive(Serialize, Deserialize)]
pub struct Link {
    pub puuids: [puuid; 2],
    pub strengh: f64,
    pub puuids_str: [String; 2],
}
#[derive(Serialize, Deserialize)]
pub struct BestDna {
    pub age_in_ticks: u32,
    pub dna: Vec<f64>,
    pub distance_traveled: f64,
}
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct BestDnaStat {
    pub age_in_ticks: u32,
    pub distance_traveled: f64,
}
#[derive(Serialize, Deserialize)]
struct ParticleClientData {
    x: f64,
    y: f64,
    diameter: f64,
    color: Color,
    type_: ParticleType,
    direction: Vector,
    energy: f64,
    output: f64,
}
#[derive(Serialize, Deserialize)]
struct Data {
    step: u32,
    stats: Vec<Stats>,
    particles: Vec<ParticleClientData>,
    constants: Constants,
    real_time_ms: u128,
    best_dna_ever_by_age: BestDnaStat,
    best_dna_alive_by_age: BestDnaStat,
    best_dna_ever_by_distance_traveled: BestDnaStat,
    best_dna_alive_by_distance_traveled: BestDnaStat,
    vision_data: Vec<VisionData>,
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
    // Compute threads
    for i in 0..thread_count {
        let chunk_lock_clone = Arc::clone(&chunk_lock);
        input_channels.push(mpsc::channel());
        let thread_output_sender = output_channel.0.clone();
        let thread_input_receiver = unsafe { ptr::read(&input_channels[i].1) };
        worker_threads.push(thread::spawn(move || {
            loop {
                let input_data = thread_input_receiver.recv().unwrap();
                let mut single_particle_updates = HashMap::new();
                let mut multiple_particle_updates = HashMap::new();
                let mut vision_data = Vec::new();
                compute(
                    i,
                    &chunk_lock_clone.read().unwrap(),
                    &input_data.puuids,
                    &input_data.luuids,
                    &mut single_particle_updates,
                    &mut multiple_particle_updates,
                    &mut vision_data,
                );
                let output_data = ComputeOutputData {
                    id: i,
                    single_particle_updates: single_particle_updates,
                    multiple_particle_updates: multiple_particle_updates,
                    vision_data: vision_data,
                };
                thread_output_sender.send(output_data).unwrap();
            }
        }));
    }
    //
    // Client data thread
    //
    let client_data = "".to_string();
    let client_data_lock = Arc::new(RwLock::new(client_data));
    {
        let chunk_lock_clone = Arc::clone(&chunk_lock);
        let client_data_lock_clone = Arc::clone(&client_data_lock);
        thread::spawn(move || {
            loop {
                let data = {
                    let chunk_read = &*chunk_lock_clone.read().unwrap();
                    let mut data = Data {
                        step: chunk_read.step,
                        particles: Vec::new(),
                        stats: chunk_read.stats.to_vec(),
                        constants: chunk_read.constants,
                        real_time_ms: chunk_read.real_time_ms,
                        best_dna_ever_by_age: BestDnaStat{
                            age_in_ticks: chunk_read.best_dna_ever_by_age.age_in_ticks,
                            distance_traveled: chunk_read.best_dna_ever_by_age.distance_traveled,
                        },
                        best_dna_alive_by_age: BestDnaStat{
                            age_in_ticks: chunk_read.best_dna_alive_by_age.age_in_ticks,
                            distance_traveled: chunk_read.best_dna_alive_by_age.distance_traveled,
                        },
                        best_dna_ever_by_distance_traveled: BestDnaStat{
                            age_in_ticks: chunk_read.best_dna_ever_by_distance_traveled.age_in_ticks,
                            distance_traveled: chunk_read.best_dna_ever_by_distance_traveled.distance_traveled,
                        },
                        best_dna_alive_by_distance_traveled: BestDnaStat{
                            age_in_ticks: chunk_read.best_dna_alive_by_distance_traveled.age_in_ticks,
                            distance_traveled: chunk_read.best_dna_alive_by_distance_traveled.distance_traveled,
                        },
                        vision_data: chunk_read.vision_data.to_vec(),
                    };
                    for p in chunk_read.particles.values() {
                        let color = match p.data {
                            ParticleData::PlantData {color} => {
                                color
                            },
                            _ => {
                                Color {
                                    r: 0.5,
                                    g: 0.5,
                                    b: 0.5
                                }
                            }
                        };
                        let direction = match p.data {
                            ParticleData::EyeData {direction} => {
                                direction
                            },
                            ParticleData::MouthData {direction} => {
                                direction
                            },
                            ParticleData::TurboData {direction} => {
                                direction
                            },
                            _ => {
                                Vector {x: 0.0, y: 0.0}
                            },
                        };
                        data.particles.push(ParticleClientData{
                            x: p.x,
                            y: p.y,
                            diameter: p.diameter,
                            color: color,
                            type_: p.type_,
                            direction: direction,
                            energy: p.energy,
                            output: p.output
                        });
                    }
                    data
                };
                *(client_data_lock_clone.write().unwrap()) = serde_json::to_string(&data).unwrap().to_string();
                thread::sleep(Duration::from_millis(10));
            }
        });
    }
    //
    // Main loop
    //
    {
        println!("starting server...");
        let chunk_lock_clone = Arc::clone(&chunk_lock);
        thread::spawn(move || {
            let mut rng = rand::thread_rng();
            let start_time = SystemTime::now();
            // Tick loop
            loop {
                //
                // Compute
                //
                let mut outputs = Vec::new();
                let mut puuids_by_thread: Vec<Vec<puuid>> = Vec::new();
                let mut luuids_by_thread: Vec<Vec<luuid>> = Vec::new();
                {
                    let chunk_read = chunk_lock_clone.read().unwrap();
                    for _ in 0..thread_count {
                        puuids_by_thread.push(Vec::new());
                        luuids_by_thread.push(Vec::new());
                    }
                    let mut i = 0;
                    for puuid in chunk_read.particles.keys() {
                        puuids_by_thread[i%thread_count].push(*puuid);
                        i += 1;
                    }
                    let mut i = 0;
                    for luuid in chunk_read.links.keys() {
                        luuids_by_thread[i%thread_count].push(*luuid);
                        i += 1;
                    }
                }
                for i in 0..thread_count {
                    input_channels[i].0.send(ComputeInputData{
                        id: i,
                        puuids: puuids_by_thread[i].to_vec(),
                        luuids: luuids_by_thread[i].to_vec(),
                    }).unwrap();
                }
                for _ in 0..thread_count {
                    outputs.push(output_receiver.recv().unwrap());
                }
                //
                // Apply
                //
                {
                    let mut chunk = chunk_lock_clone.write().unwrap();
                    for p in chunk.particles.values_mut() {
                        p.is_colliding_other_entity = false;
                        let dx = p.x - p.x_old;
                        let dy = p.y - p.y_old;
                        p.x_old = p.x;
                        p.y_old = p.y;
                        p.x += dx;
                        p.y += dy;
                    }
                    chunk.vision_data = Vec::new();
                    for output in &mut outputs {
                        for (puuid, spu) in &output.single_particle_updates {
                            let p = chunk.particles.get_mut(puuid).unwrap();
                            p.output = spu.output;
                            p.diameter = spu.diameter;
                            match &spu.particle_data {
                                Some(particle_data) => {
                                    match particle_data {
                                        ParticleData::EyeData {direction} => {
                                            p.data = ParticleData::EyeData{
                                                direction: *direction,
                                            }
                                        },
                                        ParticleData::MouthData {direction} => {
                                            p.data = ParticleData::MouthData{
                                                direction: *direction,
                                            }
                                        },
                                        ParticleData::TurboData {direction} => {
                                            p.data = ParticleData::TurboData{
                                                direction: *direction,
                                            }
                                        },
                                        _ => {},
                                    }
                                },
                                None => {}
                            }
                            if p.output < 0.0 || p.output > 1.0 {
                                println!("particle.output should be in [0.0, 1.0], not '{}'", p.output)
                            }
                        }
                        for (puuid, multiple_particle_update) in &output.multiple_particle_updates {
                            let p = chunk.particles.get_mut(puuid).unwrap();
                            p.x_old += multiple_particle_update.x_old;
                            p.y_old += multiple_particle_update.y_old;
                            p.x += multiple_particle_update.x;
                            p.y += multiple_particle_update.y;
                            p.energy += multiple_particle_update.energy;
                            p.is_colliding_other_entity = multiple_particle_update.is_colliding_other_entity || p.is_colliding_other_entity;
                        }
                        chunk.vision_data.append(&mut output.vision_data);
                    }
                }
                //
                // Update chunk
                //
                {
                    let mut chunk = chunk_lock_clone.write().unwrap();
                    chunk.real_time_ms = SystemTime::now().duration_since(start_time).unwrap().as_millis();
                    // Add entities
                    let mut entities_by_type: HashMap<EntityType, i32> = HashMap::new();
                    entities_by_type.insert(EntityType::Plant, 0);
                    entities_by_type.insert(EntityType::Bloop, 0);
                    entities_by_type.insert(EntityType::Egg, 0);
                    for entity in chunk.entities.values() {
                        match &entity.type_ {
                            type_ => *entities_by_type.get_mut(&type_).unwrap() += 1
                        }
                    }
                    let a: i32 = (*entities_by_type.get(&EntityType::Plant).unwrap()) as i32;
                    let b: i32 = chunk.constants.plant.min_count as i32;
                    let plant_to_add_count: i32 = (b - a).max(0);
                    if plant_to_add_count > 0 {
                        add_new_plant(&mut chunk, None, None);
                    }
                    let bloop_to_add_count = chunk.constants.bloop.min_count as i32
                        - entities_by_type.get(&EntityType::Bloop).unwrap()
                        - entities_by_type.get(&EntityType::Egg).unwrap();
                    if bloop_to_add_count > 0 {
                        add_new_bloop(&mut chunk);
                    }
                    // Energy drop
                    let plant_energy_drop_rate_per_tick = chunk.constants.plant.energy_drop_rate_per_tick;
                    let plant_energy_drop_rate_per_tick_circle = chunk.constants.plant.energy_drop_rate_per_tick_circle;
                    let energy_min = chunk.constants.energy_min;
                    let energy_max = chunk.constants.energy_max;
                    let bloop_energy_drop_rate_per_tick = chunk.constants.bloop.energy_drop_rate_per_tick;
                    let mouth_energy_consumption_rate_per_tick = chunk.constants.mouth_energy_consumption_rate_per_second * chunk.constants.delta_time;
                    for particle in &mut chunk.particles.values_mut() {
                        match particle.type_ {
                            ParticleType::Plant => {
                                let plant_drop_rate = plant_energy_drop_rate_per_tick
                                    + plant_energy_drop_rate_per_tick_circle * Point::get_distance(particle.x, particle.y, 0.5, 0.5);
                                particle.energy -= plant_drop_rate;
                                if particle.x < 0.0 || particle.x > 1.0 || particle.y < 0.0 || particle.y > 1.0 {
                                    particle.energy = -1.0;
                                }
                            },
                            ParticleType::Mouth => {
                                particle.energy -= bloop_energy_drop_rate_per_tick + mouth_energy_consumption_rate_per_tick;
                            },
                            _ => {
                                particle.energy -= bloop_energy_drop_rate_per_tick;
                            }
                        }
                        particle.energy = particle.energy.min(energy_max);
                    }
                    // Energy transfer
                    let mut puuid_pairs: Vec<[puuid; 2]> = Vec::new();
                    for link in chunk.links.values() {
                        puuid_pairs.push(link.puuids);
                    }
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
                    //
                    let mut particles_to_remove: HashSet<puuid> = HashSet::new();
                    struct EggToAdd {
                        x: f64,
                        y: f64,
                        dna: Vec<f64>,
                    };
                    let mut eggs_to_hatch = Vec::new();
                    for (puuid, particle) in  chunk.particles.iter() {
                        match particle.type_ {
                            ParticleType::Egg => {
                                if get_age(&chunk, particle) > chunk.constants.hatch_egg_age_ticks {
                                    particles_to_remove.insert(*puuid);
                                    eggs_to_hatch.push(EggToAdd {
                                        x: particle.x,
                                        y: particle.y,
                                        dna: chunk.entities.get(&particle.euuid).unwrap().dna.to_vec(),
                                    });
                                }
                            },
                            _ => {}
                        }
                    }
                    // Prepare remove
                    let mut entities_to_remove: HashSet<euuid> = HashSet::new();
                    for (puuid, particle) in  chunk.particles.iter() {
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
                    //
                    for egg in eggs_to_hatch {
                        add_new_bloop_from_dna_at(&mut chunk, egg.dna, egg.x, egg.y);
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
                    let mut age_in_ticks_vec = Vec::new();
                    let mut distance_traveled_vec = Vec::new();
                    for entity in chunk.entities.values() {
                        match entity.type_ {
                            EntityType::Egg => {
                                // Do nothing
                            },
                            EntityType::Plant => {
                                // Do nothing
                            },
                            EntityType::Bloop => {
                                let age_in_ticks = chunk.step - entity.tick_start;
                                age_in_ticks_vec.push(age_in_ticks);
                                let distance_traveled = Vector::new_2(
                                        entity.x_start, entity.y_start, entity.x, entity.y
                                    ).length();
                                distance_traveled_vec.push(distance_traveled);
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
                        let average_age_in_ticks = mean_u32(&age_in_ticks_vec);
                        let average_distance_traveled = mean(&distance_traveled_vec);
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
                            averages: stat::AverageStat {
                                age_in_ticks: average_age_in_ticks,
                                distance_traveled: average_distance_traveled,
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
    // println!("  configuration:  {}", chunk_configuration_str);
    for stream in server.incoming() {
        println!("incoming");
        let chunk_lock_clone = Arc::clone(&chunk_lock);
        let client_data_lock_clone = Arc::clone(&client_data_lock);
        thread::spawn (move || {
            let mut websocket = accept(stream.unwrap()).unwrap();
            let message = websocket.read_message().unwrap();
            println!("message: {}", message);
            if message == tungstenite::Message::Text("Hello Server!".to_string()) {
                loop {
                    {
                        let message_write = tungstenite::Message::Text(client_data_lock_clone.read().unwrap().to_string());
                        match websocket.write_message(message_write) {
                            Ok(_) => {
                                // Do nothing
                            },
                            Err(error) => {
                                println!("error writer socket: {}", error);
                                break;
                            }
                        }
                    }
                    thread::sleep(Duration::from_millis(10));
                }
            } else if message == tungstenite::Message::Text("latency_checker".to_string()) {
                loop {
                    match websocket.read_message() {
                        Ok(message) => {
                            if message == tungstenite::Message::Text("check".to_string()) {
                                websocket.write_message(tungstenite::Message::Text("check_back".to_string())).unwrap();
                            } else {
                                println!("message not handled: {}", message);
                            }
                        },
                        Err(error) => {
                            println!("error: {}", error);
                            break;
                        }
                    }
                }
            } else if message == tungstenite::Message::Text("writer".to_string()) {
                loop {
                    match websocket.read_message() {
                        Ok(message) => {
                            println!("message: {}", message);
                            if message == tungstenite::Message::Text("use_distance_traveled_as_fitness_function".to_string()) {
                                {
                                    let mut chunk = chunk_lock_clone.write().unwrap();
                                    chunk.constants.use_distance_traveled_as_fitness_function = true;
                                }
                            } else if message == tungstenite::Message::Text("use_distance_traveled_as_fitness_function_false".to_string()) {
                                {
                                    let mut chunk = chunk_lock_clone.write().unwrap();
                                    chunk.constants.use_distance_traveled_as_fitness_function = false;
                                }
                            } else {
                                println!("message not handled: {}", message);
                            }
                        },
                        Err(error) => {
                            println!("error: {}", error);
                            break;
                        }
                    }
                }
            } else {
                println!("starting message not handled: {}", message);
            }
        });
    }
}
fn mean(v: &Vec<f64>) -> f64 {
    let sum: f64 = Iterator::sum(v.iter());
    sum / (v.len() as f64)
}
fn mean_u32(v: &Vec<u32>) -> u32 {
    let sum: u32 = Iterator::sum(v.iter());
    if v.len() > 0 {
        sum / (v.len() as u32)
    } else {
        0
    }
}
