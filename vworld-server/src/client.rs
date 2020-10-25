use serde::{Deserialize, Serialize};
use crate::Point;
use crate::puuid;
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct VisionData {
    pub puuid_eye: puuid,
    pub puuid_target: puuid,
    pub origin: Point,
    pub target: Point,
}
