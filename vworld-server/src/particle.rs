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
    Plant
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
pub fn add_first_particle(chunk: &mut Chunk, euuid: &euuid, mut rng: &mut rand::prelude::ThreadRng) -> puuid {
    let entity = chunk.entities.get_mut(euuid).unwrap();
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
        duplication_coefficient: get_next_gene(entity, &mut rng),
        frequency: get_next_gene(entity, &mut rng),
        phase: get_next_gene(entity, &mut rng),
        data: ParticleData::None,
        output: get_next_gene(entity, &mut rng),
        bias_weight: get_next_gene(entity, &mut rng),
        is_colliding_other_entity: false,
        max_contraction: get_next_gene(entity, &mut rng) * chunk.constants.bloop.max_contraction,
    });
    entity.puuids.insert(puuid);
    return puuid;
}
fn get_cell_type(genes: [f64; 4]) -> ParticleType {
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
        _ => ParticleType::Default
    }
}
pub fn add_second_particle(chunk: &mut Chunk, euuid: &euuid, mut rng: &mut rand::prelude::ThreadRng, puuid_a: puuid) {
    let entity = chunk.entities.get_mut(euuid).unwrap();
    let energy = chunk.constants.bloop.starting_energy;
    let puuid_b = bob::new_v4().as_u128();
    let luuid = bob::new_v4().as_u128();
    let type_ = get_cell_type([
        get_next_gene(entity, &mut rng),
        get_next_gene(entity, &mut rng),
        get_next_gene(entity, &mut rng),
        get_next_gene(entity, &mut rng)
    ]);
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
        duplication_coefficient: get_next_gene(entity, &mut rng),
        frequency: get_next_gene(entity, &mut rng),
        phase: get_next_gene(entity, &mut rng),
        data: ParticleData::None,
        output: get_next_gene(entity, &mut rng),
        bias_weight: get_next_gene(entity, &mut rng),
        is_colliding_other_entity: false,
        max_contraction: get_next_gene(entity, &mut rng) * chunk.constants.bloop.max_contraction,
    });
    entity.puuids.insert(puuid_b);
    add_link(chunk, *euuid, luuid, puuid_a, puuid_b, &mut rng);
}
pub fn add_particle(chunk: &mut Chunk, euuid: &euuid, puuid_pair: [puuid; 2], mut rng: &mut rand::prelude::ThreadRng) {
    //let entity = chunk.entities.get_mut(euuid).unwrap();
    let energy = chunk.constants.bloop.starting_energy;
    let type_ = get_cell_type([
        get_next_gene(chunk.entities.get_mut(euuid).unwrap(), &mut rng),
        get_next_gene(chunk.entities.get_mut(euuid).unwrap(), &mut rng),
        get_next_gene(chunk.entities.get_mut(euuid).unwrap(), &mut rng),
        get_next_gene(chunk.entities.get_mut(euuid).unwrap(), &mut rng)
    ]);
    let puuid_a = puuid_pair[0];
    let puuid_b = puuid_pair[1];
    let puuid_c = bob::new_v4().as_u128();
    let luuid_a_c = bob::new_v4().as_u128();
    let luuid_b_c = bob::new_v4().as_u128();
    let p_a = &chunk.particles[&puuid_a];
    let p_b = &chunk.particles[&puuid_b];
    let normalized_vector = Vector::get_normalized_vector(p_a.x, p_a.y, p_b.x, p_b.y);
    let x = (p_a.x + p_b.x) * 0.5 - normalized_vector.unwrap().y * chunk.constants.default_diameter;
    let y = (p_a.y + p_b.y) * 0.5 + normalized_vector.unwrap().x * chunk.constants.default_diameter;
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
        duplication_coefficient: get_next_gene(chunk.entities.get_mut(euuid).unwrap(), &mut rng),
        frequency: get_next_gene(chunk.entities.get_mut(euuid).unwrap(), &mut rng),
        phase: get_next_gene(chunk.entities.get_mut(euuid).unwrap(), &mut rng),
        data: ParticleData::None,
        output: get_next_gene(chunk.entities.get_mut(euuid).unwrap(), &mut rng),
        bias_weight: get_next_gene(chunk.entities.get_mut(euuid).unwrap(), &mut rng),
        is_colliding_other_entity: false,
        max_contraction: get_next_gene(chunk.entities.get_mut(euuid).unwrap(), &mut rng) * chunk.constants.bloop.max_contraction,
    });
    add_pair(&mut chunk.entities.get_mut(euuid).unwrap().pairs_taken, puuid_a, puuid_b);
    add_pair(&mut chunk.entities.get_mut(euuid).unwrap().pairs_taken, puuid_b, puuid_c);
    add_pair(&mut chunk.entities.get_mut(euuid).unwrap().pairs_taken, puuid_c, puuid_a);
    add_link(chunk, *euuid, luuid_a_c, puuid_a, puuid_c, &mut rng);
    add_link(chunk, *euuid, luuid_b_c, puuid_b, puuid_c, &mut rng);
    if chunk.constants.enable_auto_link_6
    {
        struct DataMember {
            euuid: euuid,
            luuid_a_b: luuid,
            puuid_a: puuid,
            puuid_b: puuid,
            puuid_c: puuid
        };
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
            add_link(chunk, d.euuid, d.luuid_a_b, d.puuid_a, d.puuid_b, &mut rng);
            add_pair(&mut chunk.entities.get_mut(&d.euuid).unwrap().pairs_taken, d.puuid_a, d.puuid_c);
            add_pair(&mut chunk.entities.get_mut(&d.euuid).unwrap().pairs_taken, d.puuid_c, d.puuid_a);
            add_pair(&mut chunk.entities.get_mut(&d.euuid).unwrap().pairs_taken, d.puuid_b, d.puuid_c);
            add_pair(&mut chunk.entities.get_mut(&d.euuid).unwrap().pairs_taken, d.puuid_c, d.puuid_b);
            // Todo: only add pair taken in the correct direction
            add_pair(&mut chunk.entities.get_mut(&d.euuid).unwrap().pairs_taken, d.puuid_a, d.puuid_b);
            add_pair(&mut chunk.entities.get_mut(&d.euuid).unwrap().pairs_taken, d.puuid_b, d.puuid_a);
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
    mut rng: &mut rand::prelude::ThreadRng
) {
    let entity = chunk.entities.get_mut(&euuid).unwrap();
    add_pair(&mut entity.pairs, puuid_a, puuid_b);
    add_pair(&mut entity.pairs, puuid_b, puuid_a);
    chunk.particles.get_mut(&puuid_a).unwrap().links.insert(luuid_a_b, ParticleLink{
        luuid: luuid_a_b,
        puuid_linked: puuid_b,
        weight: get_next_gene(entity, &mut rng) * 2.0 - 1.0,
    });
    chunk.particles.get_mut(&puuid_b).unwrap().links.insert(luuid_a_b, ParticleLink{
        luuid: luuid_a_b,
        puuid_linked: puuid_a,
        weight: get_next_gene(entity, &mut rng) * 2.0 - 1.0,
    });
    chunk.links.insert(luuid_a_b,
        Link{
            puuids: [puuid_a, puuid_b],
            strengh: chunk.constants.link_strengh_default,
            puuids_str: [format!("{}", puuid_a), format!("{}", puuid_b)],
    });
}
fn add_pair(pairs: &mut HashMap<puuid, HashSet<puuid>>, puuid_a: puuid, puuid_b: puuid) {
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
