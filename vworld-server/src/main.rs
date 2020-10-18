#![deny(warnings)]
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
#[derive(Serialize, Deserialize)]
pub struct BestDnaStat {
    pub age_in_ticks: u32,
    pub distance_traveled: f64,
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
                compute(
                    i,
                    &chunk_lock_clone.read().unwrap(),
                    &input_data.puuids,
                    &input_data.luuids,
                    &mut single_particle_updates,
                    &mut multiple_particle_updates,
                );
                let output_data = ComputeOutputData {
                    id: i,
                    single_particle_updates: single_particle_updates,
                    multiple_particle_updates: multiple_particle_updates,
                };
                thread_output_sender.send(output_data).unwrap();
            }
        }));
    }
    // Wesocket write thread
    let sockets: Vec<tungstenite::WebSocket<_>> = Vec::new();
    let sockets_lock = Arc::new(RwLock::new(sockets));
    {
        let chunk_lock_clone = Arc::clone(&chunk_lock);
        let sockets_lock_clone = Arc::clone(&sockets_lock);
        thread::spawn(move || {
            loop {
                let json = serde_json::to_string(&*chunk_lock_clone.read().unwrap()).unwrap().to_string();
                let msg = tungstenite::Message::Text(json);
                let sockets: &mut Vec<tungstenite::WebSocket<_>> = &mut *sockets_lock_clone.write().unwrap();
                let sockets_len = sockets.len();
                for websocket in sockets {
                    match websocket.write_message(msg.clone()) {
                        _ => {}
                    }
                }
                if sockets_len == 0 {
                    thread::sleep(Duration::from_millis(1000));
                } else {
                    thread::sleep(Duration::from_millis(60));
                }
            }
        });
    }
    // Main loop
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
                    for output in &outputs {
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
                    let bloop_to_add_count = chunk.constants.bloop.min_count as i32 - entities_by_type.get(&EntityType::Bloop).unwrap();
                    for _ in 0..bloop_to_add_count {
                        add_new_bloop(&mut chunk);
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
        let chunk_lock_clone = Arc::clone(&chunk_lock);
        let sockets_lock_clone = Arc::clone(&sockets_lock);
        thread::spawn (move || {
            let mut websocket = accept(stream.unwrap()).unwrap();
            let msg = websocket.read_message().unwrap();
            println!("message: {}", msg);
            if msg == tungstenite::Message::Text("Hello Server!".to_string()) {
                let mut sockets = sockets_lock_clone.write().unwrap();
                (*sockets).push(websocket);
            } else {
                loop {
                    let msg = websocket.read_message().unwrap();
                    if msg.is_binary() || msg.is_text() {
                        if msg == tungstenite::Message::Text("use_distance_traveled_as_fitness_function".to_string()) {
                            {
                                let mut chunk = chunk_lock_clone.write().unwrap();
                                chunk.constants.use_distance_traveled_as_fitness_function = true;
                            }
                        } else if msg == tungstenite::Message::Text("use_distance_traveled_as_fitness_function_false".to_string()) {
                            {
                                let mut chunk = chunk_lock_clone.write().unwrap();
                                chunk.constants.use_distance_traveled_as_fitness_function = false;
                            }
                        }
                    }
                }
            }
        });
    }
}
