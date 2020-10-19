use serde::{Deserialize, Serialize};
use crate::BestDnaStat;
use crate::BestDna;
use crate::Constants;
use crate::euuid;
use crate::puuid;
use crate::luuid;
use crate::Link;
use crate::Entity;
use crate::Particle;
use std::collections::HashMap;
use crate::add_new_plant;
use crate::add_new_bloop;
use serde_json as json;
use crate::ParticleType;
#[derive(Serialize, Deserialize)]
pub struct Chunk {
    pub width: f64,
    pub height: f64,
    pub step: u32,
    pub particles: HashMap<puuid, Particle>,
    pub x: u32,
    pub y: u32,
    pub links: HashMap<luuid, Link>,
    pub constants: Constants,
    pub entities: HashMap<euuid, Entity>,
    pub real_time_ms: u128,
    pub particles_count: u32,
    pub entities_count: u32,
    pub links_count: u32,
    pub total_energy: f64,
    pub best_dna_ever_by_age: BestDna,
    pub best_dna_alive_by_age: BestDna,
    pub best_dna_ever_by_distance_traveled: BestDna,
    pub best_dna_alive_by_distance_traveled: BestDna,
    pub stats: Vec<Stats>,
    pub thread_count: usize,
    pub json: String,
}
#[derive(Serialize, Deserialize)]
struct EntityConfiguration {
    particles: Vec<ParticleConfiguration>,
    links: Vec<Link>
}
#[derive(Serialize, Deserialize)]
struct ParticleConfiguration {
    x: f64,
    y: f64,
    type_: ParticleType,
    diameter: f64,
    mass: f64
}
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Stats {
    pub step: u32,
    pub real_time_s: f64,
    pub simulation_time_s: f64,
    pub steps_per_second: f64,
    pub simulation_speed: f64,
    pub best_dna_ever_by_age: BestDnaStat,
    pub best_dna_alive_by_age: BestDnaStat,
    pub best_dna_ever_by_distance_traveled: BestDnaStat,
    pub best_dna_alive_by_distance_traveled: BestDnaStat,
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
pub fn create_chunk_from_configuration_str(configuration_str: &str) -> Chunk {
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
