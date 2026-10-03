//! Exact Retrobution `WorldNameScript` rectangle lookup used by the Nanocom.
//!
//! The source data is the `worldname` MonoBehaviour recovered from the
//! beta-20100104 Shared assets. Order is significant because the original
//! `WorldDataContainer` returns the first `Rect.Contains` match.

#[derive(Clone, Copy, Debug, PartialEq)]
struct LegacyWorldLocation {
    x: f32,
    z: f32,
    width: f32,
    depth: f32,
    name: &'static str,
}

impl LegacyWorldLocation {
    const fn new(x: f32, z: f32, width: f32, depth: f32, name: &'static str) -> Self {
        Self {
            x,
            z,
            width,
            depth,
            name,
        }
    }

    fn contains(self, x: f32, z: f32) -> bool {
        // Unity Rect.Contains includes the minimum edges and excludes maxima.
        x >= self.x && x < self.x + self.width && z >= self.z && z < self.z + self.depth
    }
}

const LEGACY_WORLD_LOCATIONS: &[LegacyWorldLocation] = &[
    LegacyWorldLocation::new(4096.0, 2560.0, 512.0, 512.0, "Candy Cove"),
    LegacyWorldLocation::new(1024.0, 1536.0, 512.0, 512.0, "Townsville Park"),
    LegacyWorldLocation::new(2560.0, 512.0, 512.0, 512.0, "Bravo Beach"),
    LegacyWorldLocation::new(1536.0, 1024.0, 512.0, 512.0, "Orchid Bay"),
    LegacyWorldLocation::new(1536.0, 512.0, 512.0, 512.0, "Orchid Bay"),
    LegacyWorldLocation::new(2048.0, 512.0, 512.0, 512.0, "Orchid Bay"),
    LegacyWorldLocation::new(1536.0, 1536.0, 512.0, 512.0, "Marquee Row"),
    LegacyWorldLocation::new(512.0, 6144.0, 512.0, 512.0, "Green Maw"),
    LegacyWorldLocation::new(512.0, 5632.0, 512.0, 512.0, "Green Maw"),
    LegacyWorldLocation::new(1024.0, 5632.0, 512.0, 512.0, "Green Maw"),
    LegacyWorldLocation::new(3072.0, 4608.0, 512.0, 512.0, "Forsaken Valley"),
    LegacyWorldLocation::new(3584.0, 4608.0, 512.0, 512.0, "Forsaken Valley"),
    LegacyWorldLocation::new(3584.0, 5120.0, 512.0, 512.0, "Forsaken Valley"),
    LegacyWorldLocation::new(2560.0, 4608.0, 512.0, 512.0, "Dinosaur Pass"),
    LegacyWorldLocation::new(6144.0, 4096.0, 512.0, 512.0, "Nowhere"),
    LegacyWorldLocation::new(6656.0, 4096.0, 512.0, 512.0, "Nowhere"),
    LegacyWorldLocation::new(6656.0, 5120.0, 512.0, 512.0, "Area 51.5"),
    LegacyWorldLocation::new(6656.0, 4608.0, 512.0, 512.0, "Nowhere"),
    LegacyWorldLocation::new(4096.0, 7168.0, 512.0, 512.0, "Monkey Mountain"),
    LegacyWorldLocation::new(4608.0, 7168.0, 512.0, 512.0, "Monkey Foothills"),
    LegacyWorldLocation::new(5120.0, 7168.0, 512.0, 512.0, "Monkey Foothills"),
    LegacyWorldLocation::new(5120.0, 6656.0, 512.0, 512.0, "Monkey Foothills"),
    LegacyWorldLocation::new(2048.0, 2560.0, 512.0, 512.0, "City Point"),
    LegacyWorldLocation::new(2560.0, 2560.0, 512.0, 512.0, "Galaxy Gardens"),
    LegacyWorldLocation::new(2048.0, 1536.0, 512.0, 512.0, "Marquee Row"),
    LegacyWorldLocation::new(2560.0, 1024.0, 512.0, 512.0, "Bravo Beach"),
    LegacyWorldLocation::new(2048.0, 3072.0, 512.0, 512.0, "Endsville"),
    LegacyWorldLocation::new(4608.0, 4608.0, 382.0, 85.0, "Prickly Pines"),
    LegacyWorldLocation::new(4608.0, 4690.0, 338.0, 201.0, "Prickly Pines"),
    LegacyWorldLocation::new(4608.0, 4890.0, 310.0, 741.0, "Prickly Pines"),
    LegacyWorldLocation::new(4991.0, 4608.0, 129.0, 85.0, "Camp Kidney"),
    LegacyWorldLocation::new(4947.0, 4690.0, 173.0, 201.0, "Camp Kidney"),
    LegacyWorldLocation::new(4919.0, 4890.0, 201.0, 741.0, "Camp Kidney"),
    LegacyWorldLocation::new(7680.0, 5632.0, 512.0, 512.0, "Lower Catacombs"),
    LegacyWorldLocation::new(7680.0, 5120.0, 512.0, 512.0, "Upper Catacombs"),
    LegacyWorldLocation::new(2560.0, 5120.0, 512.0, 512.0, "Dinosaur Pass"),
    LegacyWorldLocation::new(1536.0, 3072.0, 512.0, 512.0, "Habitat Homes"),
    LegacyWorldLocation::new(1536.0, 2560.0, 512.0, 512.0, "City Hall"),
    LegacyWorldLocation::new(2048.0, 2048.0, 512.0, 512.0, "City Station"),
    LegacyWorldLocation::new(2560.0, 1536.0, 512.0, 512.0, "Morbucks Towers"),
    LegacyWorldLocation::new(4096.0, 3584.0, 512.0, 512.0, "Peach Creek Commons"),
    LegacyWorldLocation::new(4096.0, 3072.0, 512.0, 512.0, "Candy Cove"),
    LegacyWorldLocation::new(3072.0, 2048.0, 512.0, 512.0, "Steam Alley"),
    LegacyWorldLocation::new(2048.0, 6656.0, 512.0, 512.0, "Dark Glade"),
    LegacyWorldLocation::new(5632.0, 5120.0, 512.0, 512.0, "Devil's Bluff"),
    LegacyWorldLocation::new(5632.0, 4608.0, 512.0, 512.0, "Devil's Canyon"),
    LegacyWorldLocation::new(3072.0, 3072.0, 512.0, 512.0, "Genius Grove"),
    LegacyWorldLocation::new(3072.0, 5120.0, 512.0, 512.0, "Forsaken Valley"),
    LegacyWorldLocation::new(2048.0, 6144.0, 512.0, 512.0, "Huntor's Crest"),
    LegacyWorldLocation::new(1024.0, 2048.0, 512.0, 512.0, "Townsville Park"),
    LegacyWorldLocation::new(4608.0, 4096.0, 512.0, 512.0, "Wilson Way"),
    LegacyWorldLocation::new(1024.0, 6144.0, 512.0, 512.0, "Green Maw"),
    LegacyWorldLocation::new(1536.0, 6144.0, 512.0, 512.0, "Hero's Hollow"),
    LegacyWorldLocation::new(1024.0, 6656.0, 512.0, 512.0, "Fuse's Lair"),
    LegacyWorldLocation::new(1536.0, 6656.0, 512.0, 512.0, "The Precipice"),
    LegacyWorldLocation::new(4608.0, 6144.0, 512.0, 512.0, "Forgotten Falls"),
    LegacyWorldLocation::new(2560.0, 3072.0, 512.0, 512.0, "Eternal Vistas"),
    LegacyWorldLocation::new(2560.0, 3584.0, 512.0, 512.0, "Eternal Meadows"),
    LegacyWorldLocation::new(5632.0, 4096.0, 512.0, 512.0, "Pimpleback Mountains"),
    LegacyWorldLocation::new(5632.0, 3584.0, 512.0, 512.0, "Haunted Ridge"),
    LegacyWorldLocation::new(4608.0, 3584.0, 512.0, 512.0, "Peach Creek Estates"),
    LegacyWorldLocation::new(3584.0, 4096.0, 512.0, 512.0, "Sector V"),
    LegacyWorldLocation::new(5120.0, 4608.0, 512.0, 512.0, "Leakey Lake"),
    LegacyWorldLocation::new(5120.0, 4096.0, 512.0, 512.0, "Acorn Flats"),
    LegacyWorldLocation::new(2560.0, 2048.0, 512.0, 512.0, "Mojo's Volcano"),
    LegacyWorldLocation::new(4096.0, 6656.0, 512.0, 512.0, "Monkey Mountain"),
    LegacyWorldLocation::new(4608.0, 6656.0, 512.0, 512.0, "Monkey Foothills"),
    LegacyWorldLocation::new(6144.0, 4608.0, 512.0, 512.0, "Nowhere"),
    LegacyWorldLocation::new(6144.0, 5120.0, 512.0, 512.0, "Area 51.5"),
    LegacyWorldLocation::new(2048.0, 3584.0, 512.0, 512.0, "Nuclear Plant"),
    LegacyWorldLocation::new(5632.0, 5632.0, 512.0, 512.0, "The Ruins"),
    LegacyWorldLocation::new(4608.0, 3072.0, 512.0, 512.0, "Goat's Junk Yard"),
    LegacyWorldLocation::new(3072.0, 1536.0, 512.0, 512.0, "Offworld Plaza"),
    LegacyWorldLocation::new(1536.0, 2048.0, 512.0, 512.0, "Townsville Center"),
    LegacyWorldLocation::new(2560.0, 6144.0, 512.0, 512.0, "Firepits"),
    LegacyWorldLocation::new(2560.0, 5632.0, 512.0, 512.0, "Fireswamps"),
    LegacyWorldLocation::new(3072.0, 2560.0, 512.0, 512.0, "Tech Square"),
    LegacyWorldLocation::new(3584.0, 3072.0, 512.0, 512.0, "Pokey Oaks South"),
    LegacyWorldLocation::new(3584.0, 3584.0, 512.0, 512.0, "Pokey Oaks North"),
    LegacyWorldLocation::new(5120.0, 5120.0, 512.0, 512.0, "Mount Blackhead"),
    LegacyWorldLocation::new(4608.0, 5632.0, 512.0, 512.0, "Really Twisted Forest"),
    LegacyWorldLocation::new(5120.0, 5632.0, 512.0, 512.0, "Twisted Forest"),
    LegacyWorldLocation::new(2048.0, 1024.0, 512.0, 512.0, "Orchid Bay"),
    LegacyWorldLocation::new(7168.0, 3584.0, 512.0, 512.0, "Crystalline Caverns"),
    LegacyWorldLocation::new(6656.0, 1024.0, 512.0, 512.0, "Peach Creek Commons"),
    LegacyWorldLocation::new(6656.0, 512.0, 512.0, 512.0, "Candy Cove"),
    LegacyWorldLocation::new(5632.0, 512.0, 512.0, 512.0, "Genius Grove"),
    LegacyWorldLocation::new(7168.0, 1024.0, 512.0, 512.0, "Peach Creek Estates"),
    LegacyWorldLocation::new(6144.0, 1536.0, 512.0, 512.0, "Sector V"),
    LegacyWorldLocation::new(6144.0, 1024.0, 512.0, 512.0, "Pokey Oaks North"),
    LegacyWorldLocation::new(6144.0, 512.0, 512.0, 512.0, "Pokey Oaks South"),
    LegacyWorldLocation::new(7168.0, 512.0, 512.0, 512.0, "Goat's Junk Yard"),
    LegacyWorldLocation::new(512.0, 512.0, 512.0, 512.0, "Tech Square"),
    LegacyWorldLocation::new(0.0, 4608.0, 512.0, 512.0, "unknown"),
];

/// Returns the first Retrobution minimap location containing the Unity X/Z point.
#[must_use]
pub fn legacy_world_location_name(unity_x: f32, unity_z: f32) -> Option<&'static str> {
    if !unity_x.is_finite() || !unity_z.is_finite() {
        return None;
    }
    LEGACY_WORLD_LOCATIONS
        .iter()
        .copied()
        .find(|location| location.contains(unity_x, unity_z))
        .map(|location| location.name)
}

/// Returns the caption for the Unity X/Z point, keeping `last` when the point
/// lies outside every rectangle (instances/dungeons). Clean
/// `WorldDataContainer.Update` replaces its current area only on a
/// `Rect.Contains` hit, so the minimap keeps the last named location.
#[must_use]
pub fn legacy_world_location_name_or_last(last: &str, unity_x: f32, unity_z: f32) -> Option<&str> {
    match legacy_world_location_name(unity_x, unity_z) {
        Some(name) => Some(name),
        None => is_legacy_world_location_name(last).then_some(last),
    }
}

/// Whether `name` is a WorldNameScript caption rather than a technical
/// scene/map fallback such as `Map 12`.
#[must_use]
pub fn is_legacy_world_location_name(name: &str) -> bool {
    LEGACY_WORLD_LOCATIONS
        .iter()
        .any(|location| location.name == name)
}

#[cfg(test)]
mod tests;
