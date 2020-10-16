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
