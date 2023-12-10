//use crate::Particle;
use crate::bob;
use crate::Chunk;
use crate::euuid;
use crate::puuid;
use crate::luuid;
use crate::Link;
use crate::Vector;
use crate::entity::Entity;
use crate::get_next_gene;
use std::collections::HashSet;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::chunk::ParticleConfiguration;
use crate::entity::PairInfo;
use rand::Rng;
const GENES_PER_PARTICLE_2: usize = 17;
const GENES_PER_PARTICLE: usize = 21;
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}
#[derive(Serialize, Deserialize)]
pub enum ParticleData {
    None,
    EyeData {
        direction: Vector,
    },
    MouthData {
        direction: Vector,
    },
    TurboData {
        direction: Vector,
    },
    PlantData {
        color: Color,
    }
}
#[derive(Copy, Clone, Serialize, Deserialize)]
pub enum ParticleType {
    Sun,
    Energy,
    Default,
    Muscle,
    MuscleInverted,
    MuscleRandom,
    Heart,
    Eye,
    Mouth,
    Plant,
    Turbo,
    Clock,
    Stomach,
    Constant,
    Egg
}
// Represents a link to another particle, from the current particle
#[derive(Serialize, Deserialize)]
pub struct ParticleLink {
    pub luuid: luuid,
    pub puuid_linked: puuid,
    pub weight: f64,
}
#[derive(Serialize, Deserialize)]
pub struct Particle {
    pub x: f64,
    pub y: f64,
    pub x_old: f64,
    pub y_old: f64,
    pub mass: f64,
    pub base_diameter: f64,
    pub diameter: f64,
    pub type_: ParticleType,
    pub puuid: puuid,
    pub energy: f64,
    pub links: HashMap<luuid, ParticleLink>,
    pub euuid: euuid,
    pub output: f64,
    pub bias_weight: f64,
    pub duplication_coefficient: f64,
    pub frequency: f64,
    pub phase: f64,
    pub data: ParticleData,
    pub is_colliding_other_entity: bool,
    pub max_contraction: f64,
}
pub fn get_genes_first_particle(entity: &mut Entity, mut rng: &mut rand::prelude::ThreadRng) -> [f64;5] {
    let mut genes = [0.0;5];
    for i in 0..genes.len() {
        genes[i] = get_next_gene(entity, &mut rng);
    }
    return genes;
}
pub fn get_genes_second_particle(entity: &mut Entity, mut rng: &mut rand::prelude::ThreadRng) -> [f64;GENES_PER_PARTICLE_2] {
    let mut genes = [0.0;GENES_PER_PARTICLE_2];
    for i in 0..genes.len() {
        genes[i] = get_next_gene(entity, &mut rng);
    }
    return genes;
}
pub fn get_genes_other_particle(entity: &mut Entity, mut rng: &mut rand::prelude::ThreadRng) -> [f64;GENES_PER_PARTICLE] {
    let mut genes = [0.0;GENES_PER_PARTICLE];
    for i in 0..genes.len() {
        genes[i] = get_next_gene(entity, &mut rng);
    }
    return genes;
}
pub fn get_genes_first_particle_from_conf(particle_conf: &ParticleConfiguration) -> [f64;5] {
    let mut genes = [0.5;5];
    genes[0] = 1.0;
    match particle_conf.frequency {
        Some(v) => genes[1] = v,
        None => {}
    }
    match particle_conf.phase {
        Some(v) => genes[2] = v,
        None => {}
    }
    return genes;
}
pub fn get_genes_second_particle_from_conf(particle_conf: &ParticleConfiguration) -> [f64;GENES_PER_PARTICLE_2] {
    let mut genes = [0.5;GENES_PER_PARTICLE_2];
    let genes_first =  get_genes_first_particle_from_conf(particle_conf);
    for i in 0..genes_first.len() {
        genes[i] = genes_first[i];
    }
    match particle_conf.type_ {
        ParticleType::Muscle =>     genes[5] = 0.75,
        ParticleType::Eye =>        genes[6] = 0.75,
        ParticleType::Mouth =>      genes[7] = 0.75,
        ParticleType::Default =>    genes[8] = 0.75,
        ParticleType::Turbo =>      genes[9] = 0.75,
        ParticleType::Clock =>      genes[10] = 0.75,
        ParticleType::Stomach =>    genes[11] = 0.75,
        ParticleType::Constant =>   genes[12] = 0.75,
        _ => {}
    };
    return genes;
}
pub fn get_genes_particle_from_conf(particle_conf: &ParticleConfiguration) -> [f64;GENES_PER_PARTICLE] {
    let mut genes = [0.5;GENES_PER_PARTICLE];
    let genes_second =  get_genes_second_particle_from_conf(particle_conf);
    for i in 0..genes_second.len() {
        genes[i] = genes_second[i];
    }
    return genes;
}

pub fn add_first_particle(chunk: &mut Chunk, euuid: &euuid, rng: &mut rand::prelude::ThreadRng) -> puuid {
    let entity = chunk.entities.get_mut(euuid).unwrap();
    // get genes
    let genes = get_genes_first_particle(entity, rng);
    let duplication_coefficient =   genes[0];
    let frequency =                 genes[1];
    let phase =                     genes[2];
    let bias_weight =               genes[3];
    let max_contraction =           genes[4] * chunk.constants.bloop.max_contraction;
    //
    let energy = chunk.constants.bloop.starting_energy;
    let puuid = bob::new_v4().as_u128();
    chunk.particles.insert(puuid, Particle{
        puuid: puuid,
        x: entity.x_start,
        y: entity.y_start,
        x_old: entity.x_start,
        y_old: entity.y_start,
        mass: chunk.constants.default_mass,
        base_diameter: chunk.constants.default_diameter,
        diameter: chunk.constants.default_diameter,
        type_: ParticleType::Heart,
        energy: energy,
        euuid: *euuid,
        links: HashMap::new(),
        duplication_coefficient: duplication_coefficient,
        frequency: frequency,
        phase: phase,
        data: ParticleData::None,
        output: 0.0,
        bias_weight: bias_weight,
        is_colliding_other_entity: false,
        max_contraction: max_contraction,
    });
    entity.puuids.insert(puuid);
    return puuid;
}
fn get_cell_type(genes: [f64; 8]) -> ParticleType {
    let mut max_gene = 0.0;
    let mut id = 0;
    for (i, v) in genes.iter().enumerate() {
        if *v > max_gene {
            id = i;
            max_gene = *v;
        }
    }
    match id {
        0 => ParticleType::Muscle,
        1 => ParticleType::Eye,
        2 => ParticleType::Mouth,
        3 => ParticleType::Default,
        4 => ParticleType::Turbo,
        5 => ParticleType::Clock,
        6 => ParticleType::Stomach,
        7 => ParticleType::Constant,
        _ => {
            println!("get_cell_type error: {}", id);
            ParticleType::Default
        }
    }
}
pub fn add_second_particle(chunk: &mut Chunk, euuid: &euuid, rng: &mut rand::prelude::ThreadRng, puuid_a: puuid) {
    let entity = chunk.entities.get_mut(euuid).unwrap();
    let energy = chunk.constants.bloop.starting_energy;
    let puuid_b = bob::new_v4().as_u128();
    let luuid = bob::new_v4().as_u128();
    // get genes
    let genes = get_genes_second_particle(entity, rng);
    let duplication_coefficient =   genes[0];
    let frequency =                 genes[1];
    let phase =                     genes[2];
    let bias_weight =               genes[3];
    let max_contraction =           genes[4] * chunk.constants.bloop.max_contraction;
    let cell_type_genes = [         genes[5],
                                    genes[6],
                                    genes[7],
                                    genes[8],
                                    genes[9],
                                    genes[10],
                                    genes[11],
                                    genes[12]
    ];
    let gene_weight_a_b =           genes[13];
    let gene_weight_b_a =           genes[14];
    let gene_duplication_coefficient_a_b = genes[15];
    let gene_duplication_coefficient_b_a = genes[16];
    //
    let type_ = get_cell_type(cell_type_genes);
    chunk.particles.insert(puuid_b, Particle{
        puuid: puuid_b,
        x: entity.x_start + chunk.constants.default_diameter,
        y: entity.y_start,
        x_old: entity.x_start + chunk.constants.default_diameter,
        y_old: entity.y_start,
        mass: chunk.constants.default_mass,
        base_diameter: chunk.constants.default_diameter,
        diameter: chunk.constants.default_diameter,
        type_: type_,
        energy: energy,
        euuid: *euuid,
        links: HashMap::new(),
        duplication_coefficient: duplication_coefficient,
        frequency: frequency,
        phase: phase,
        data: ParticleData::None,
        output: 0.0,
        bias_weight: bias_weight,
        is_colliding_other_entity: false,
        max_contraction: max_contraction,
    });
    entity.puuids.insert(puuid_b);
    add_link(chunk, *euuid, luuid, puuid_a, puuid_b,
        gene_weight_a_b,
        gene_weight_b_a,
        gene_duplication_coefficient_a_b,
        gene_duplication_coefficient_b_a
    );
}
pub fn add_particle(chunk: &mut Chunk, euuid: &euuid, puuid_pair: [puuid; 2], mut rng: &mut rand::prelude::ThreadRng) {
    //let entity = chunk.entities.get_mut(euuid).unwrap();
    let energy = chunk.constants.bloop.starting_energy;
    // get genes
    let genes = get_genes_other_particle(chunk.entities.get_mut(euuid).unwrap(), rng);
    let duplication_coefficient =   genes[0];
    let frequency =                 genes[1];
    let phase =                     genes[2];
    let bias_weight =               genes[3];
    let max_contraction =           genes[4] * chunk.constants.bloop.max_contraction;
    let cell_type_genes = [         genes[5],
                                    genes[6],
                                    genes[7],
                                    genes[8],
                                    genes[9],
                                    genes[10],
                                    genes[11],
                                    genes[12]
    ];
    let gene_weight_a_c =           genes[13];
    let gene_weight_c_a =           genes[14];
    let gene_weight_b_c =           genes[15];
    let gene_weight_c_b =           genes[16];
    let gene_duplication_coefficient_a_c = genes[17];
    let gene_duplication_coefficient_c_a = genes[18];
    let gene_duplication_coefficient_b_c = genes[19];
    let gene_duplication_coefficient_c_b = genes[20];
    //
    let type_ = get_cell_type(cell_type_genes);
    let puuid_a = puuid_pair[0];
    let puuid_b = puuid_pair[1];
    let puuid_c = bob::new_v4().as_u128();
    let luuid_a_c = bob::new_v4().as_u128();
    let luuid_b_c = bob::new_v4().as_u128();
    let p_a = &chunk.particles[&puuid_a];
    let p_b = &chunk.particles[&puuid_b];
    let (x, y) = match Vector::get_normalized_vector(p_a.x, p_a.y, p_b.x, p_b.y) {
        Some(normalized_vector) => {
            let x = (p_a.x + p_b.x) * 0.5 - normalized_vector.y * chunk.constants.default_diameter;
            let y = (p_a.y + p_b.y) * 0.5 + normalized_vector.x * chunk.constants.default_diameter;
            (x, y)
        },
        None => {
            let x = (p_a.x + p_b.x) * 0.5 - rng.gen::<f64>() * chunk.constants.default_diameter;
            let y = (p_a.y + p_b.y) * 0.5 + rng.gen::<f64>() * chunk.constants.default_diameter;
            (x, y)
        }
    };

    chunk.entities.get_mut(euuid).unwrap().puuids.insert(puuid_c);
    chunk.particles.insert(puuid_c, Particle{
        puuid: puuid_c,
        x: x,
        y: y,
        x_old: x,
        y_old: y,
        mass: chunk.constants.default_mass,
        base_diameter: chunk.constants.default_diameter,
        diameter: chunk.constants.default_diameter,
        type_:  type_,
        energy: energy,
        euuid: *euuid,
        links: HashMap::new(),
        duplication_coefficient: duplication_coefficient,
        frequency: frequency,
        phase: phase,
        data: ParticleData::None,
        output: 0.0,
        bias_weight: bias_weight,
        is_colliding_other_entity: false,
        max_contraction: max_contraction,
    });
    add_pair_taken(&mut chunk.entities.get_mut(euuid).unwrap().pairs_taken, puuid_a, puuid_b);
    add_pair_taken(&mut chunk.entities.get_mut(euuid).unwrap().pairs_taken, puuid_b, puuid_c);
    add_pair_taken(&mut chunk.entities.get_mut(euuid).unwrap().pairs_taken, puuid_c, puuid_a);
    add_link(chunk, *euuid, luuid_a_c, puuid_a, puuid_c,
        gene_weight_a_c,
        gene_weight_c_a,
        gene_duplication_coefficient_a_c,
        gene_duplication_coefficient_c_a,
    );
    add_link(chunk, *euuid, luuid_b_c, puuid_b, puuid_c,
        gene_weight_b_c,
        gene_weight_c_b,
        gene_duplication_coefficient_b_c,
        gene_duplication_coefficient_c_b
    );
    if chunk.constants.enable_auto_link_6
    {
        struct DataMember {
            euuid: euuid,
            luuid_a_b: luuid,
            puuid_a: puuid,
            puuid_b: puuid,
            puuid_c: puuid
        }
        let mut data = Vec::new();
        let entity = chunk.entities.get(euuid).unwrap();
        for puuid in &entity.puuids {
            let links_count = chunk.particles.get(&puuid).unwrap().links.len();
            if links_count < 6 {
                // Do nothing
            } else if links_count == 6 {
                let mut link_counts_per_neighbour: HashMap<puuid, u32> = HashMap::new();
                let links = &chunk.particles.get(&puuid).unwrap().links;
                for particle_link in links.values() {
                    link_counts_per_neighbour.insert(particle_link.puuid_linked, 0);
                }
                for particle_link_a in links.values() {
                    for particle_link_b in links.values() {
                        let puuid_a = particle_link_a.puuid_linked;
                        let puuid_b = particle_link_b.puuid_linked;
                        if puuid_a < puuid_b && particles_are_paired(chunk.entities.get(euuid).unwrap(), puuid_a, puuid_b) {
                            *link_counts_per_neighbour.get_mut(&puuid_a).unwrap() += 1;
                            *link_counts_per_neighbour.get_mut(&puuid_b).unwrap() += 1;
                        }
                    }
                }
                let mut monolinked_puuids = Vec::new();
                for (puuid, count) in link_counts_per_neighbour {
                    match count {
                        1 => {
                            monolinked_puuids.push(puuid)
                        },
                        2 => {
                            // Do nothing
                        },
                        value => {
                            println!("error: link count should be 1 or 2, not {}", value)
                        }
                    }
                }
                match monolinked_puuids.len() {
                    0 => {
                        // Do nothing
                    },
                    2 => {
                        let puuid_a = monolinked_puuids[0];
                        let puuid_b = monolinked_puuids[1];
                        let luuid_a_b = bob::new_v4().as_u128();

                        data.push(DataMember{
                            euuid: *euuid,
                            luuid_a_b: luuid_a_b,
                            puuid_a: puuid_a,
                            puuid_b: puuid_b,
                            puuid_c: *puuid
                        })
                    },
                    value => {
                        println!("error: should not have {} monolink", value)
                    }
                }
            } else {
                println!("error: too many neighbours: {}", links_count)
            }
        }
        for d in data {
            //println!("4 get_next_gene");
            let gene_weight_a_b = get_next_gene(chunk.entities.get_mut(euuid).unwrap(), &mut rng);
            let gene_weight_b_a = get_next_gene(chunk.entities.get_mut(euuid).unwrap(), &mut rng);
            let gene_duplication_coefficient_a_b = get_next_gene(chunk.entities.get_mut(euuid).unwrap(), &mut rng);
            let gene_duplication_coefficient_b_a = get_next_gene(chunk.entities.get_mut(euuid).unwrap(), &mut rng);
            add_link(chunk, d.euuid, d.luuid_a_b, d.puuid_a, d.puuid_b,
                gene_weight_a_b,
                gene_weight_b_a,
                gene_duplication_coefficient_a_b,
                gene_duplication_coefficient_b_a,
            );
            add_pair_taken(&mut chunk.entities.get_mut(&d.euuid).unwrap().pairs_taken, d.puuid_a, d.puuid_c);
            add_pair_taken(&mut chunk.entities.get_mut(&d.euuid).unwrap().pairs_taken, d.puuid_c, d.puuid_a);
            add_pair_taken(&mut chunk.entities.get_mut(&d.euuid).unwrap().pairs_taken, d.puuid_b, d.puuid_c);
            add_pair_taken(&mut chunk.entities.get_mut(&d.euuid).unwrap().pairs_taken, d.puuid_c, d.puuid_b);
            // Todo: only add pair taken in the correct direction
            add_pair_taken(&mut chunk.entities.get_mut(&d.euuid).unwrap().pairs_taken, d.puuid_a, d.puuid_b);
            add_pair_taken(&mut chunk.entities.get_mut(&d.euuid).unwrap().pairs_taken, d.puuid_b, d.puuid_a);
        }
    }
}
pub fn add_plant_particle(chunk: &mut Chunk, euuid: &euuid, mut rng: &mut rand::prelude::ThreadRng) -> puuid {
    let entity = chunk.entities.get_mut(euuid).unwrap();
    let energy = 1.0;
    let puuid = bob::new_v4().as_u128();
    let red = get_next_gene(entity, &mut rng) * 0.0;
    let green = get_next_gene(entity, &mut rng) * 0.5 + 0.5;
    let blue = get_next_gene(entity, &mut rng) * 0.5 + 0.5;
    chunk.particles.insert(puuid, Particle{
        puuid: puuid,
        x: entity.x_start,
        y: entity.y_start,
        x_old: entity.x_start,
        y_old: entity.y_start,
        mass: chunk.constants.default_mass,
        base_diameter: chunk.constants.default_diameter,
        diameter: chunk.constants.default_diameter,
        type_: ParticleType::Plant,
        energy: energy,
        euuid: *euuid,
        links: HashMap::new(),
        duplication_coefficient: get_next_gene(entity, &mut rng),
        frequency: get_next_gene(entity, &mut rng),
        phase: get_next_gene(entity, &mut rng),
        data: ParticleData::PlantData {
            color: Color {
                r: red,
                g: green,
                b: blue,
            }
        },
        output: 0.0,
        bias_weight: get_next_gene(entity, &mut rng),
        is_colliding_other_entity: false,
        max_contraction: 0.0,
    });
    entity.puuids.insert(puuid);
    return puuid;
}
pub fn add_egg_particle(chunk: &mut Chunk, euuid: &euuid) -> puuid {
    let entity = chunk.entities.get_mut(euuid).unwrap();
    let energy = 1.0;
    let puuid = bob::new_v4().as_u128();
    chunk.particles.insert(puuid, Particle{
        puuid: puuid,
        x: entity.x_start,
        y: entity.y_start,
        x_old: entity.x_start,
        y_old: entity.y_start,
        mass: chunk.constants.default_mass,
        base_diameter: chunk.constants.default_diameter,
        diameter: chunk.constants.default_diameter,
        type_: ParticleType::Egg,
        energy: energy,
        euuid: *euuid,
        links: HashMap::new(),
        duplication_coefficient: 0.0,
        frequency: 0.0,
        phase: 0.0,
        data: ParticleData::None,
        output: 0.0,
        bias_weight: 0.0,
        is_colliding_other_entity: false,
        max_contraction: 0.0,
    });
    entity.puuids.insert(puuid);
    return puuid;
}
fn particles_are_paired(entity: &Entity, puuid_a: puuid, puuid_b: puuid) -> bool {
    match entity.pairs.get(&puuid_a) {
        Some(hashset) => {
            match hashset.get(&puuid_b) {
                Some(_) =>  return true,
                None =>  return false
            }
        },
        None => return false
    }
}
fn add_link(
    chunk: &mut Chunk,
    euuid: euuid,
    luuid_a_b: luuid,
    puuid_a: puuid,
    puuid_b: puuid,
    gene_weight_a_b: f64,
    gene_weight_b_a: f64,
    gene_duplication_coefficient_a_b: f64,
    gene_duplication_coefficient_b_a: f64,
) {
    let entity = chunk.entities.get_mut(&euuid).unwrap();
    add_pair(&mut entity.pairs, puuid_a, puuid_b, gene_duplication_coefficient_a_b);
    add_pair(&mut entity.pairs, puuid_b, puuid_a, gene_duplication_coefficient_b_a);
    chunk.particles.get_mut(&puuid_a).unwrap().links.insert(luuid_a_b, ParticleLink{
        luuid: luuid_a_b,
        puuid_linked: puuid_b,
        weight: gene_weight_a_b * 2.0 - 1.0,
    });
    chunk.particles.get_mut(&puuid_b).unwrap().links.insert(luuid_a_b, ParticleLink{
        luuid: luuid_a_b,
        puuid_linked: puuid_a,
        weight: gene_weight_b_a * 2.0 - 1.0,
    });
    chunk.links.insert(luuid_a_b,
        Link{
            puuids: [puuid_a, puuid_b],
            strengh: chunk.constants.link_strengh_default,
            puuids_str: [format!("{}", puuid_a), format!("{}", puuid_b)],
    });
}
fn add_pair(
    pairs: &mut HashMap<puuid, HashMap<puuid, PairInfo>>,
    puuid_a: puuid,
    puuid_b: puuid,
    duplication_coefficient: f64,
) {
    match pairs.get_mut(&puuid_a) {
        Some(hashmap) => {
            hashmap.insert(puuid_b, PairInfo{
                puuid_a: puuid_a,
                puuid_b: puuid_b,
                duplication_coefficient: duplication_coefficient,
            });
        },
        None => {
            let mut hashmap = HashMap::new();
            hashmap.insert(puuid_b, PairInfo{
                puuid_a: puuid_a,
                puuid_b: puuid_b,
                duplication_coefficient: duplication_coefficient,
            });
            pairs.insert(puuid_a, hashmap);
        }
    }
}
fn add_pair_taken(pairs: &mut HashMap<puuid, HashSet<puuid>>, puuid_a: puuid, puuid_b: puuid) {
    match pairs.get_mut(&puuid_a) {
        Some(hashmap) => {
            hashmap.insert(puuid_b);
        },
        None => {
            let mut hashset = HashSet::new();
            hashset.insert(puuid_b);
            pairs.insert(puuid_a, hashset);
        }
    }
}
pub fn get_age(chunk: &Chunk, particle: &Particle) -> u32 {
    return chunk.step - chunk.entities.get(&particle.euuid).unwrap().tick_start;
}
