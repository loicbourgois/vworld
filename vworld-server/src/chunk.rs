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
use serde_json as json;
use std::collections::HashSet;
use crate::ParticleType;
use crate::stat::AverageStat;
use crate::entity::add_new_bloop_from_dna_at;
use crate::particle::get_genes_first_particle_from_conf;
use crate::particle::get_genes_second_particle_from_conf;
use crate::particle::get_genes_particle_from_conf;
use crate::client::VisionData;
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
    pub vision_data: Vec<VisionData>,
}
#[derive(Serialize, Deserialize)]
struct EntityConfiguration {
    x: f64,
    y: f64,
    particles: Vec<ParticleConfiguration>,
    trinomes: Vec<[usize;3]>,
    particle_closers: HashSet<usize>,
}
#[derive(Serialize, Deserialize)]
pub struct ParticleConfiguration {
    pub type_: ParticleType,
    pub phase: Option<f64>,
    pub frequency: Option<f64>,
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
    pub averages: AverageStat,
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
    human_entities: Vec<EntityConfiguration>,
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
        vision_data: Vec::new(),
    };
    for (i, human_entity_config) in configuration.human_entities.iter().enumerate() {
        println!("loading entity #{}", i);
        let dna = get_dna_from_human_entity_conf(&configuration.constants, human_entity_config);
        println!("  pre-computed dna len: {}", dna.len());
        //println!("{:?}", dna);
        let eeuid = add_new_bloop_from_dna_at(&mut chunk, dna, human_entity_config.x, human_entity_config.y);
        println!("  real dna len:         {}", chunk.entities.get(&eeuid).unwrap().dna.len() );
    }
    for _ in 0..configuration.constants.plant.min_count {
        add_new_plant(&mut chunk, None, None);
    }
    return chunk
}
fn get_dna_from_human_entity_conf(constants: &Constants, human_entity_config: &EntityConfiguration) -> Vec<f64> {
    let mut dna = Vec::new();
    let debug_entity_conf = false;
    dna.append(&mut get_genes_first_particle_from_conf(&human_entity_config.particles[0]).to_vec());
    dna.append(&mut get_genes_second_particle_from_conf(&human_entity_config.particles[1]).to_vec());
    let particle_to_add_count = human_entity_config.particles.len();
    let link_close_gene_count = 4;
    for i in 2..particle_to_add_count {
        dna.append(&mut get_genes_particle_from_conf(&human_entity_config.particles[i]).to_vec());
        match human_entity_config.particle_closers.get(&i) {
            Some(_) => {
                for _ in 0..link_close_gene_count {
                    dna.push(0.2);
                }
            },
            None => {}
        }
    }
    let duplication_coefficient_minus = (1.0 - constants.min_duplication_coefficient) / (particle_to_add_count as f64);
    let mut duplication_coefficient = 1.0 - duplication_coefficient_minus;
    let entity_genes_count = 0;
    let first_particle_genes_count = 5;
    let second_particle_genes_count = 13;
    let other_particle_genes_count = 17;
    let count_before_gene_duplication_coefficient = 11;
    let count_before_gene_duplication_coefficient_2 = 13;
    for i in 0..particle_to_add_count-2 {
        let trinome = human_entity_config.trinomes[i];
        let id_a = trinome[0];
        let id_b = trinome[1];
        let id_c = trinome[2];
        let source_p_id = id_a.max(id_b);
        let mut close_count = 0;
        for j in 0..source_p_id {
            match human_entity_config.particle_closers.get(&j) {
                Some(_) => {
                    close_count += 1;
                }, None => {}
            }
        }
        let a_to_b = if id_a == 0 && id_b == 1 {
            0
        } else if id_a == 1 && id_b == 0 {
            1
        } else if id_a < id_b {
            0
        } else {
            3
        };
        let dna_id = if id_a + id_b == 1 {
            entity_genes_count + first_particle_genes_count + count_before_gene_duplication_coefficient + a_to_b
        } else {
            entity_genes_count + first_particle_genes_count
            + second_particle_genes_count
            + other_particle_genes_count * (source_p_id-2)
            + count_before_gene_duplication_coefficient_2
            + a_to_b
            + close_count * link_close_gene_count
        };
        dna[dna_id] = duplication_coefficient;
        if debug_entity_conf {
            println!("id_c: {}", id_c);
            println!("  dna_id: {}", dna_id);
            println!("  close_count: {}", close_count);
            println!("  source_p_id: {}", source_p_id);
            println!("  duplication_coefficient: {}", duplication_coefficient);
        }
        duplication_coefficient -= duplication_coefficient_minus;
    }
    let theorical_dna_size = entity_genes_count
        + first_particle_genes_count
        + second_particle_genes_count
        + (particle_to_add_count-2) * other_particle_genes_count
        + human_entity_config.particle_closers.len() * link_close_gene_count;
    println!("  theorical dna len:    {}", theorical_dna_size);
    return dna;
}
