use serde::{Deserialize, Serialize};
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct AverageStat {
    pub age_in_ticks: u32,
    pub distance_traveled: f64,
}
