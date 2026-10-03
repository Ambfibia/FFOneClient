//! User-owned map filters and custom waypoints, independent of mission state.
use serde::{Deserialize, Serialize};

pub const WAYPOINT_HUES: [u16; 6] = [235, 290, 60, 100, 30, 180];

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CustomWaypoint {
    pub x: f32,
    pub z: f32,
    pub color: u8,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct MapPreferences {
    pub enabled_icons: Vec<u8>,
    pub waypoints: Vec<CustomWaypoint>,
}

impl Default for MapPreferences {
    fn default() -> Self {
        Self {
            enabled_icons: vec![
                4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 15, 16, 21, 22, 24, 25, 30, 31, 32, 33, 34,
            ],
            waypoints: Vec::new(),
        }
    }
}

impl MapPreferences {
    pub fn validate(&self) -> bool {
        self.enabled_icons
            .iter()
            .all(|icon| (1..=34).contains(icon))
            && self.waypoints.len() <= WAYPOINT_HUES.len()
            && self.waypoints.iter().enumerate().all(|(index, point)| {
                point.x.is_finite()
                    && point.z.is_finite()
                    && (0.0..=8192.0).contains(&point.x)
                    && (0.0..=8192.0).contains(&point.z)
                    && usize::from(point.color) < WAYPOINT_HUES.len()
                    && !self.waypoints[..index]
                        .iter()
                        .any(|other| other.color == point.color)
            })
    }

    pub fn add_waypoint(&mut self, x: f32, z: f32) -> Option<u8> {
        if !x.is_finite()
            || !z.is_finite()
            || !(0.0..=8192.0).contains(&x)
            || !(0.0..=8192.0).contains(&z)
        {
            return None;
        }
        let color =
            (0..6).find(|color| !self.waypoints.iter().any(|point| point.color == *color))?;
        self.waypoints.push(CustomWaypoint { x, z, color });
        Some(color)
    }
}

#[cfg(test)]
mod tests;
