use serde::{Deserialize, Serialize};
use crate::Vector;
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Constants {
    pub collision_push_rate: f64,
    pub drag_coefficient: f64,
    pub link_length_coefficient: f64,
    pub diameter_muscle_change_rate: f64,
    pub delta_time: f64,
    pub default_diameter: f64,
    pub default_mass: f64,
    pub gravity: Vector,
    pub min_body_parts_count: u32,
    pub max_body_parts_count: u32,
    pub link_strengh_default: f64,
    pub energy_max: f64,
    pub energy_min: f64,
    pub eye_sight_length: f64,
    pub mouth_energy_eating_rate_per_second: f64,
    pub bloop: EntityConstants,
    pub plant: PlantConstants,
    pub enable_auto_link_6: bool,
    pub max_stats_count: i32,
    pub destroy_unstable_entities: bool,
    pub muscles_use_output: bool,
    pub display_simulation_logs: bool,
    pub use_distance_traveled_as_fitness_function: bool,
    pub thread_count: u32,
}
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct EntityConstants {
    pub gene_random_mutation_rate: f64,
    pub gene_progressive_mutation_rate: f64,
    pub gene_progressive_mutation_strength: f64,
    pub new_dna_rate: f64,
    pub min_count: u32,
    pub energy_drop_rate_per_tick: f64,
    pub starting_energy: f64,
}
#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct PlantConstants {
    pub mutation_rate: f64,
    pub max_mutation_strength: f64,
    pub new_dna_rate: f64,
    pub min_count: u32,
    pub energy_drop_rate_per_tick: f64,
    pub energy_drop_rate_per_tick_circle: f64,
}
