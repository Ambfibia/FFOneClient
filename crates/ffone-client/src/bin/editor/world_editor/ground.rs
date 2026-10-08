//! Heightfield sampling for both placement views uses the published native basis.
use super::*;
use std::io::BufReader;

pub(super) struct Tile {
    pub authored: Option<ffone_client::native_terrain::NativeTerrain>,
    pub descriptor: Value,
    pub samples: Vec<u16>,
    pub path: PathBuf,
    pub weights: Vec<Vec<u8>>,
    pub revision: u64,
    pub transform: Mat4,
    pub width: usize,
    pub height: usize,
    pub spacing: Vec2,
    pub height_scale: f32,
    pub collider: ffone_client::native_terrain::NativeHeightmapCollider,
}
#[derive(serde::Deserialize)]
struct Frame {
    root: ffone_client::world::AuthoredWorldTransform,
    #[serde(rename = "nativeTerrain")]
    terrain: ffone_client::world::NativeWorldTerrainInstance,
}
impl Tile {
    pub fn load(root: &Path, id: &str) -> Result<Self, String> {
        let frame: Frame = serde_json::from_reader(BufReader::new(
            fs::File::open(root.join(format!("map/tiles/{id}/scene.json")))
                .map_err(|e| e.to_string())?,
        ))
        .map_err(|e| e.to_string())?;
        let mut transform = frame
            .root
            .try_to_bevy("editor ground root")
            .map_err(|e| e.to_string())?;
        if let Some(chain) = &frame.terrain.root_chain {
            for node in chain.nodes.iter().rev() {
                transform = transform.mul_transform(
                    node.native_local_transform
                        .try_to_bevy("editor ground hierarchy")
                        .map_err(|e| e.to_string())?,
                );
            }
        }
        transform = transform.mul_transform(
            frame
                .terrain
                .transform
                .try_to_bevy("editor ground")
                .map_err(|e| e.to_string())?,
        );
        let path = root.join(&frame.terrain.path);
        let descriptor: Value = serde_json::from_reader(BufReader::new(
            fs::File::open(&path).map_err(|e| e.to_string())?,
        ))
        .map_err(|e| e.to_string())?;
        let image = ::image::open(
            path.parent().unwrap().join(
                descriptor["heightmap"]["path"]
                    .as_str()
                    .ok_or("Missing heightmap")?,
            ),
        )
        .map_err(|e| e.to_string())?
        .into_luma16();
        let width = image.width() as usize;
        let height = image.height() as usize;
        if width < 2 || height < 2 {
            return Err("Invalid terrain dimensions".into());
        }
        let samples = image.into_raw();
        let weights = descriptor["splat"]["weightMaps"]
            .as_array()
            .ok_or("Missing terrain weights")?
            .iter()
            .map(|v| {
                let name = v["path"].as_str().ok_or("Missing weight image")?;
                Ok(::image::open(path.parent().unwrap().join(name))
                    .map_err(|e| e.to_string())?
                    .into_rgba8()
                    .into_raw())
            })
            .collect::<Result<Vec<_>, String>>()?;
        let native = serde_json::from_value(descriptor.clone()).map_err(|e| e.to_string())?;
        let collider =
            ffone_client::native_terrain::NativeHeightmapCollider::from_samples(&native, &samples)
                .map_err(|e| e.to_string())?;
        Ok(Self {
            authored: None,
            width,
            height,
            spacing: Vec2::new(
                descriptor["scale"]["sampleSpacingX"]
                    .as_f64()
                    .ok_or("Missing sample spacing")? as f32,
                descriptor["scale"]["sampleSpacingZ"]
                    .as_f64()
                    .ok_or("Missing sample spacing")? as f32,
            ),
            height_scale: descriptor["scale"]["heightScale"]
                .as_f64()
                .ok_or("Missing height scale")? as f32
                / descriptor["scale"]["heightNormalizationDenominator"]
                    .as_f64()
                    .ok_or("Missing height denominator")? as f32,
            transform: transform.to_matrix(),
            samples,
            descriptor,
            collider,
            path,
            weights,
            revision: 0,
        })
    }
    pub fn height_at(&self, point: Vec3) -> Option<f32> {
        self.collider.ground_height(
            &GlobalTransform::from(Transform::from_matrix(self.transform)),
            point.x,
            point.z,
            -1_000_000.,
            1_000_000.,
        )
    }
}
impl WorldEditor {
    pub(super) fn load_ground(&mut self, point: Vec3) -> Result<(), String> {
        let Some(tile) = atlas::tile_at(point) else {
            return Err("Invalid map coordinate".into());
        };
        if !self.available_tiles.contains(&tile) {
            return Ok(());
        }
        let id = atlas::tile_id(tile);
        if !self.ground.contains_key(&id) {
            if self.sources.iter().find(|s|s.path==self.root.join(format!("map/tiles/{id}/scene.json"))).is_some_and(|s|s.draft["nativeTerrain"].is_null()) { return Ok(()); }
            self.ground.insert(id.clone(), Tile::load(&self.root, &id)?);
            self.sync_terrain()?;
        }
        Ok(())
    }
    pub(super) fn ground_height(&self, point: Vec3) -> Option<f32> {
        let id = atlas::tile_id(atlas::tile_at(point)?);
        self.ground.get(&id)?.height_at(point)
    }
    pub(super) fn snap_to_ground(&mut self, mut point: Vec3) -> Result<Vec3, String> {
        self.load_ground(point)?;
        if let Some(height) = self.ground_height(point) {
            point.y = height;
        }
        Ok(point)
    }
    pub(super) fn ground_ray(&self, ray: Ray3d) -> Option<Vec3> {
        let mut previous: Option<(f32, f32)> = None;
        for step in 0..2500 {
            let t = step as f32 * 8.;
            let point = ray.origin + *ray.direction * t;
            let Some(height) = self.ground_height(point) else {
                previous = None;
                continue;
            };
            let gap = point.y - height;
            if let Some((old_t, old_gap)) = previous {
                if old_gap >= 0. && gap <= 0. {
                    let (mut low, mut high) = (old_t, t);
                    for _ in 0..16 {
                        let mid = (low + high) * 0.5;
                        let at = ray.origin + *ray.direction * mid;
                        if at.y > self.ground_height(at)? {
                            low = mid;
                        } else {
                            high = mid;
                        }
                    }
                    return Some(ray.origin + *ray.direction * ((low + high) * 0.5));
                }
            }
            previous = Some((t, gap));
        }
        None
    }
}
