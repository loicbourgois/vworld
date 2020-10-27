//use crate::Particle;
use crate::bob;
use crate::Chunk;
use crate::euuid;
use crate::puuid;
use crate::Point;
use crate::compute;
use std::collections::HashSet;
use std::collections::HashMap;
use rand::Rng;
use serde::{Deserialize, Serialize};
use crate::add_first_particle;
use crate::add_second_particle;
use crate::add_particle;
use crate::particle::add_plant_particle;
use crate::particle::add_egg_particle;
use crate::particle::ParticleType;
#[derive(Serialize, Deserialize, Hash, Eq)]
pub enum EntityType {
    Plant = 1,
    Bloop = 2,
    Egg = 3
}
impl PartialEq for EntityType {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}
#[derive(Serialize, Deserialize)]
pub struct Entity {
    pub euuid: euuid,
    pub puuids: HashSet<puuid>,
    pub x_start: f64,
    pub y_start: f64,
    pub x: f64,
    pub y: f64,
    pub dna: Vec<f64>,
    pub next_gene_id: usize,
    // Pairs have a direction
    // A->B != B->A
    pub pairs: HashMap<puuid, HashMap<puuid, PairInfo>>,
    // TODO: tranform pairs_taken into trinome:
    //  this means we need to add a third puuid
    //  probably HashMap<puuid, HashMap<puuid, puuid>>,
    pub pairs_taken: HashMap<puuid, HashSet<puuid>>,
    pub tick_start: u32,
    pub type_: EntityType,
}
#[derive(Serialize, Deserialize, Copy, Clone)]
pub struct PairInfo {
    pub puuid_a: puuid,
    pub puuid_b: puuid,
    pub duplication_coefficient: f64,
}
pub fn get_next_gene(entity: &mut Entity, rng: &mut rand::prelude::ThreadRng) -> f64 {
    let next_gene_id = entity.next_gene_id;
    if entity.dna.len() == entity.next_gene_id {
        entity.dna.push(rng.gen::<f64>());
    }
    entity.next_gene_id += 1;
    return entity.dna[next_gene_id];
}
//pub fn add_new_plant_at(chunk: &mut Chunk, p: Point, dna_option: Option<Vec<f64>>) {
//    add_new_plant(chunk, Some(p), dna_option);
//}
pub fn add_new_plant(mut chunk: &mut Chunk, coord_option: Option<Point>, dna_option: Option<Vec<f64>>) {
    // Todo, replace by const
    let border = 0.0;
    let mut rng = rand::thread_rng();
    let mut euuids: Vec<euuid> = Vec::new();
    for (euuid, entity) in chunk.entities.iter() {
        match entity.type_ {
            EntityType::Plant => euuids.push(*euuid),
            _ => {}
        }
    }
    let parent_euuid_option = if euuids.len() > 0 {
        let i = (rng.gen::<f64>() * euuids.len() as f64 ) as usize;
        Some(euuids[i])
    } else {
        None
    };
    let mut dna = match (dna_option, parent_euuid_option) {
        (Some(dna), _) => {
            dna.to_vec()
        },
        (None, Some(parent_euuid)) => {
            chunk.entities.get(&parent_euuid).unwrap().dna.to_vec()
        },
        (None, None) => {
            Vec::new()
        }
    };
    if rng.gen::<f64>() < chunk.constants.plant.new_dna_rate {
        for gen in dna.iter_mut() {
            *gen = rng.gen::<f64>();
        }
    } else {
        for gen in dna.iter_mut() {
            if rng.gen::<f64>() < chunk.constants.plant.mutation_rate {
                *gen = *gen + (rng.gen::<f64>() * chunk.constants.plant.max_mutation_strength * 2.0) - chunk.constants.plant.max_mutation_strength;
                *gen = gen.max(0.0).min(1.0);
            }
        }
    }
    let x = match coord_option {
        Some(coord) => coord.x,
        None => {
            (match parent_euuid_option {
                Some(parent_euuid) => {
                    chunk.entities.get(&parent_euuid).unwrap().x + rng.gen::<f64>() * 0.00002 - 0.00001
                },
                None => {
                    rng.gen::<f64>() * (1.0 - 2.0 * border) + border
                }
            }).max(0.0).min(1.0)
        }
    };
    let y = match coord_option {
        Some(coord) => coord.y,
        None => {
            (match parent_euuid_option {
                Some(parent_euuid) => {
                    chunk.entities.get(&parent_euuid).unwrap().y + rng.gen::<f64>() * 0.00002 - 0.00001
                },
                None => {
                    println!("none y");
                    rng.gen::<f64>() * (1.0 - 2.0 * border) + border
                }
            }).max(0.0).min(1.0)
        }
    };
    let euuid: euuid = bob::new_v4().as_u128();
    chunk.entities.insert(euuid, Entity {
        euuid: euuid,
        puuids: HashSet::new(),
        x_start: x,
        y_start: y,
        x: x,
        y: y,
        dna: dna,
        next_gene_id: 0,
        pairs: HashMap::new(),
        pairs_taken: HashMap::new(),
        tick_start: chunk.step,
        type_: EntityType::Plant
    });
    add_plant_particle(&mut chunk, &euuid, &mut rng);
}
pub fn add_new_bloop_from_dna_at(mut chunk: &mut Chunk, dna: Vec<f64>, x:f64, y:f64) -> euuid {
    let mut rng = rand::thread_rng();
    let euuid: euuid = bob::new_v4().as_u128();
    let constants = chunk.constants;
    chunk.entities.insert(euuid, Entity {
        euuid: euuid,
        puuids: HashSet::new(),
        x_start: x,
        y_start: y,
        x: x,
        y: y,
        dna: dna,
        next_gene_id: 0,
        pairs: HashMap::new(),
        pairs_taken: HashMap::new(),
        tick_start: chunk.step,
        type_: EntityType::Bloop
    });
    let puuid_a = add_first_particle(&mut chunk, &euuid, &mut rng);
    add_second_particle(&mut chunk, &euuid, &mut rng, puuid_a);
    for _ in 2..constants.min_body_parts_count {
        let entity = chunk.entities.get(&euuid).unwrap();
        let free_pairs = get_free_pairs(entity);
        let mut best_duplication_coefficient = 0.0;
        let mut best_pair_id = free_pairs.len();
        for (pair_id, puuid_pair) in free_pairs.iter().enumerate() {
            if puuid_pair.duplication_coefficient >= best_duplication_coefficient {
                best_pair_id = pair_id;
                best_duplication_coefficient = puuid_pair.duplication_coefficient;
            }
        }
        let best_pair = free_pairs[best_pair_id];
        add_particle(&mut chunk, &euuid, [best_pair.puuid_a, best_pair.puuid_b], &mut rng);
    }
    for _ in constants.min_body_parts_count..constants.max_body_parts_count {
        let entity = chunk.entities.get(&euuid).unwrap();
        let free_pairs = get_free_pairs(entity);
        let mut best_duplication_coefficient = 0.0;
        let mut best_pair_id = free_pairs.len();
        for (pair_id, puuid_pair) in free_pairs.iter().enumerate() {
            if puuid_pair.duplication_coefficient >= best_duplication_coefficient {
                best_pair_id = pair_id;
                best_duplication_coefficient = puuid_pair.duplication_coefficient;
            }
        }
        if best_duplication_coefficient >= constants.min_duplication_coefficient {
            let best_pair = free_pairs[best_pair_id];
            add_particle(&mut chunk, &euuid, [best_pair.puuid_a, best_pair.puuid_b], &mut rng);
        } else {
            break;
        }
    }
    return euuid;
}
struct EuuidDnaPair {
    dna: Vec<f64>,
    euuid: euuid,
}
pub fn add_new_bloop(chunk: &mut Chunk) {
    let mut rng = rand::thread_rng();
    let mut dnas: Vec<EuuidDnaPair> = Vec::new();
    for (euuid, entity) in &chunk.entities {
        match entity.type_ {
            EntityType::Bloop => dnas.push(EuuidDnaPair{
                dna: entity.dna.to_vec(),
                euuid: *euuid,
            }),
            _ => {}
        }
    }
    // Assign dna
    let mut x = rng.gen::<f64>();
    let mut y = rng.gen::<f64>();
    let mut lay_egg = false;
    let mut dna = if chunk.constants.use_distance_traveled_as_fitness_function && chunk.best_dna_alive_by_distance_traveled.dna.len() > 0 {
        chunk.best_dna_alive_by_distance_traveled.dna.to_vec()
    } else if dnas.len() > 0 {
        let i = (rng.gen::<f64>() * dnas.len() as f64 ) as usize;
        let euuid = dnas[i].euuid;
        for p in chunk.particles.values() {
            if p.euuid == euuid {
                match p.type_ {
                    ParticleType::Heart => {
                        let direction = compute::get_direction(chunk, p);
                        if chunk.constants.same_position_as_parent {
                            x = (p.x + direction.x * p.diameter).max(0.0).min(1.0);
                            y = (p.y + direction.y * p.diameter).max(0.0).min(1.0);
                        }
                        lay_egg = p.phase < chunk.constants.lay_egg_rate;
                        break;
                    },
                    _ => {}
                }
            }
        }
        dnas[i].dna.to_vec()
    } else {
        Vec::new()
    };
    // Mutate dna
    let mut rng = rand::thread_rng();
    if rng.gen::<f64>() < chunk.constants.bloop.new_dna_rate {
        for gen in dna.iter_mut() {
            *gen = rng.gen::<f64>();
        }
    } else {
        for gen in dna.iter_mut() {
            if rng.gen::<f64>() < chunk.constants.bloop.gene_progressive_mutation_rate {
                *gen = *gen + (rng.gen::<f64>() * chunk.constants.bloop.gene_progressive_mutation_strength * 2.0) - chunk.constants.bloop.gene_progressive_mutation_strength;
                *gen = gen.max(0.0).min(1.0);
            }
            if rng.gen::<f64>() < chunk.constants.bloop.gene_random_mutation_rate {
                *gen = rng.gen::<f64>();
            }
        }
    }
    if lay_egg {
        add_new_egg_from_dna_at(chunk, dna, x, y);
    } else {
        add_new_bloop_from_dna_at(chunk, dna, x, y);
    }
}
pub fn add_new_egg_from_dna_at(mut chunk: &mut Chunk, dna: Vec<f64>, x:f64, y:f64) {
    let euuid: euuid = bob::new_v4().as_u128();
    chunk.entities.insert(euuid, Entity {
        euuid: euuid,
        puuids: HashSet::new(),
        x_start: x,
        y_start: y,
        x: x,
        y: y,
        dna: dna,
        next_gene_id: 0,
        pairs: HashMap::new(),
        pairs_taken: HashMap::new(),
        tick_start: chunk.step,
        type_: EntityType::Egg
    });
    let _puuid = add_egg_particle(&mut chunk, &euuid);
}
fn get_free_pairs(entity: &Entity) -> Vec<PairInfo> {
    let mut v: Vec<PairInfo> = Vec::new();
    for (puuid_a, hashmap) in entity.pairs.iter() {
        for (puuid_b, pair_info) in hashmap {
            match entity.pairs_taken.get(puuid_a) {
                Some(hashset_taken) => {
                    match hashset_taken.get(puuid_b) {
                        Some(_) => (),
                        None => v.push(*pair_info)
                    }
                }, None => v.push(*pair_info)
            }
        }
    }
    return v;
}
