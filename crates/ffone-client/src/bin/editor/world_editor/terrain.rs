//! Brush strokes, shared undo history and validated native terrain publication.
use super::*;
use ffone_client::native_terrain::{NativeHeightmapCollider, NativeTerrain, NativeTerrainMaterial};
use model::{Patch, Source};
use std::io::Cursor;

pub(super) struct Stroke {
    sources: BTreeSet<usize>,
    target: f32,
    texture: Option<String>,
}
pub(super) fn path_texture(name: &str) -> bool {
    let name=name.to_ascii_lowercase();
    ["road", "path", "trail", "walk", "pavement"].iter().any(|part|name.contains(part))
}
pub(super) fn pixels(width: u32, height: u32, color: &str, values: impl serde::Serialize) -> Value {
    serde_json::json!({"schema":"ffone.editor-pixels.v1","width":width,"height":height,"color":color,"pixels":values})
}
pub(super) fn read_source(path: &Path) -> Result<Value, String> {
    if path.extension().is_some_and(|p| p == "png") {
        let image = ::image::open(path).map_err(|e| format!("{}: {e}", path.display()))?;
        let (width, height) = (image.width(), image.height());
        return Ok(if image.color() == ::image::ColorType::L16 {
            pixels(width, height, "gray16", image.into_luma16().into_raw())
        } else {
            pixels(width, height, "rgba8", image.into_rgba8().into_raw())
        });
    }
    serde_json::from_slice(&fs::read(path).map_err(|e| e.to_string())?).map_err(|e| e.to_string())
}
pub(super) fn pixel_bytes(v: &Value) -> Result<Vec<u8>, String> {
    let width = v["width"].as_u64().ok_or("Missing image width")? as u32;
    let height = v["height"].as_u64().ok_or("Missing image height")? as u32;
    let values = v["pixels"].as_array().ok_or("Missing pixels")?;
    let image = if v["color"] == "gray16" {
        let raw = values
            .iter()
            .map(|v| {
                v.as_u64()
                    .filter(|n| *n <= 65535)
                    .map(|n| n as u16)
                    .ok_or("Invalid height sample")
            })
            .collect::<Result<Vec<_>, _>>()?;
        ::image::DynamicImage::ImageLuma16(
            ::image::ImageBuffer::from_raw(width, height, raw)
                .ok_or("Invalid height dimensions")?,
        )
    } else {
        let raw = values
            .iter()
            .map(|v| {
                v.as_u64()
                    .filter(|n| *n <= 255)
                    .map(|n| n as u8)
                    .ok_or("Invalid texture weight")
            })
            .collect::<Result<Vec<_>, _>>()?;
        ::image::DynamicImage::ImageRgba8(
            ::image::ImageBuffer::from_raw(width, height, raw)
                .ok_or("Invalid weight dimensions")?,
        )
    };
    let mut encoded = Cursor::new(Vec::new());
    image
        .write_to(&mut encoded, ::image::ImageFormat::Png)
        .map_err(|e| e.to_string())?;
    Ok(encoded.into_inner())
}
pub(super) fn source_bytes(v: &Value) -> Result<Vec<u8>, String> {
    if v["schema"] == "ffone.editor-pixels.v1" {
        pixel_bytes(v)
    } else {
        let mut bytes = serde_json::to_vec_pretty(v).map_err(|e| e.to_string())?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}
fn hash(bytes: &[u8]) -> Value {
    Value::from(format!("blake3:{}", blake3::hash(bytes).to_hex()))
}
fn raw16(v: &[u16]) -> Vec<u8> {
    v.iter().flat_map(|v| v.to_le_bytes()).collect()
}

pub(super) fn normalize_weights(values: &mut [u8], preferred: usize) {
    let mut correction = 255 - values.iter().map(|v| i32::from(*v)).sum::<i32>();
    for i in std::iter::once(preferred).chain((0..values.len()).filter(|i| *i != preferred)) {
        let next = (i32::from(values[i]) + correction).clamp(0, 255);
        correction -= next - i32::from(values[i]);
        values[i] = next as u8;
        if correction == 0 {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn painted_weights_always_sum_to_255() {
        for initial in [
            vec![0, 0, 0, 0],
            vec![0, 85, 85, 86],
            vec![255, 255, 255, 255],
            vec![1; 16],
        ] {
            for preferred in 0..initial.len() {
                let mut weights = initial.clone();
                normalize_weights(&mut weights, preferred);
                assert_eq!(weights.iter().map(|v| u32::from(*v)).sum::<u32>(), 255);
            }
        }
    }
    #[test]
    fn pixel_sources_round_trip_without_losing_height_precision() {
        let temp = tempfile::tempdir().unwrap();
        for value in [
            pixels(2, 2, "gray16", vec![0u16, 1, 32768, 65535]),
            pixels(1, 2, "rgba8", vec![0u8, 3, 252, 0, 255, 0, 0, 0]),
        ] {
            let path = temp.path().join("pixels.png");
            fs::write(&path, pixel_bytes(&value).unwrap()).unwrap();
            assert_eq!(read_source(&path).unwrap(), value);
        }
    }
}

impl WorldEditor {
    fn terrain_source(&mut self, path: PathBuf) -> Result<usize, String> {
        if let Some(i) = self.sources.iter().position(|s| s.path == path) {
            return Ok(i);
        }
        let base = if path.exists(){read_source(&path)?}else{Value::Null};
        let i = self.sources.len();
        self.sources.push(Source {
            path,
            draft: base.clone(),
            base,
        });
        Ok(i)
    }
    pub(super) fn sync_terrain(&mut self) -> Result<(), String> {
        self.restore_authored_terrain()?;
        for tile in self.ground.values_mut() {
            let parent = tile.path.parent().ok_or("Missing terrain directory")?;
            let mut changed = false;
            let height = parent.join(
                tile.descriptor["heightmap"]["path"]
                    .as_str()
                    .ok_or("Missing heightmap")?,
            );
            if let Some(source) = self.sources.iter().find(|s| s.path == height) {
                let samples = source.draft["pixels"]
                    .as_array()
                    .ok_or("Invalid height draft")?
                    .iter()
                    .map(|v| v.as_u64().map(|n| n as u16).ok_or("Invalid height sample"))
                    .collect::<Result<Vec<_>, _>>()?;
                if samples != tile.samples {
                    tile.samples = samples;
                    changed = true;
                }
            }
            for (i, map) in tile.descriptor["splat"]["weightMaps"]
                .as_array()
                .into_iter()
                .flatten()
                .enumerate()
            {
                if let Some(source) = self
                    .sources
                    .iter()
                    .find(|s| s.path == parent.join(map["path"].as_str().unwrap_or_default()))
                {
                    let weights = source.draft["pixels"]
                        .as_array()
                        .ok_or("Invalid weights draft")?
                        .iter()
                        .map(|v| v.as_u64().map(|n| n as u8).ok_or("Invalid weight"))
                        .collect::<Result<Vec<_>, _>>()?;
                    if weights != tile.weights[i] {
                        tile.weights[i] = weights;
                        changed = true;
                    }
                }
            }
            if changed {
                tile.collider = NativeHeightmapCollider::from_samples(
                    &serde_json::from_value(tile.descriptor.clone()).map_err(|e| e.to_string())?,
                    &tile.samples,
                )
                .map_err(|e| e.to_string())?;
                tile.revision += 1;
            }
        }
        Ok(())
    }
    pub(super) fn terrain_brush(
        &mut self,
        point: Vec3,
        elapsed: f32,
        lower: bool,
    ) -> Result<(), String> {
        let Some(tool) = self.terrain_tool else {
            return Ok(());
        };
        self.load_ground(point)?;
        if self.brush.is_none() {
            let texture = self
                .ground
                .get(&atlas::tile_id(
                    atlas::tile_at(point).ok_or("Invalid terrain position")?,
                ))
                .and_then(|t| t.descriptor["splat"]["layers"].get(self.brush_layer))
                .and_then(|l| l["trueTextureName"].as_str())
                .map(str::to_owned);
            self.brush = Some(Stroke {
                sources: BTreeSet::new(),
                target: point.y,
                texture,
            });
        }
        let target = self.brush.as_ref().unwrap().target;
        let texture = self.brush.as_ref().unwrap().texture.clone();
        let radius = self.brush_radius;
        let strength = self.brush_strength * elapsed.min(0.1).max(0.001);
        let ids: Vec<_> = self.ground.keys().cloned().collect();
        for id in ids {
            let tile = &self.ground[&id];
            let local = tile.transform.inverse().transform_point3(point);
            let extent = Vec2::new(
                (tile.width - 1) as f32 * tile.spacing.x,
                (tile.height - 1) as f32 * tile.spacing.y,
            );
            if local.x > radius
                || -local.x > extent.x + radius
                || local.z < -radius
                || local.z > extent.y + radius
            {
                continue;
            }
            let parent = tile.path.parent().unwrap().to_path_buf();
            let paths: Vec<_> = if tool >= 2 {
                tile.descriptor["splat"]["weightMaps"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|m| parent.join(m["path"].as_str().unwrap()))
                    .collect()
            } else {
                vec![parent.join(tile.descriptor["heightmap"]["path"].as_str().unwrap())]
            };
            for path in paths {
                let i = self.terrain_source(path)?;
                self.brush.as_mut().unwrap().sources.insert(i);
            }
            let tile = self.ground.get_mut(&id).unwrap();
            let mut changed = false;
            if tool < 2 {
                let y_step = tile
                    .transform
                    .inverse()
                    .transform_vector3(Vec3::Y * strength)
                    .y
                    / tile.height_scale;
                let local_target = tile
                    .transform
                    .inverse()
                    .transform_point3(Vec3::new(point.x, target, point.z))
                    .y
                    / tile.height_scale;
                let shifts: BTreeMap<_, _> = tile.descriptor["heightmap"]["vertexShifts"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .filter_map(|v| {
                        Some((
                            v["row"].as_u64()? as usize * tile.width
                                + v["column"].as_u64()? as usize,
                            v["flags"].as_u64()?,
                        ))
                    })
                    .collect();
                for row in 0..tile.height {
                    for col in 0..tile.width {
                        let index = row * tile.width + col;
                        let flags = shifts.get(&index).copied().unwrap_or(0);
                        let x_shift = i32::from(flags & 2 != 0) - i32::from(flags & 1 != 0);
                        let z_shift = i32::from(flags & 8 != 0) - i32::from(flags & 4 != 0);
                        let world = tile.transform.transform_point3(Vec3::new(
                            -(col as f32 + x_shift as f32) * tile.spacing.x,
                            tile.samples[index] as f32 * tile.height_scale,
                            (row as f32 + z_shift as f32) * tile.spacing.y,
                        ));
                        let distance = Vec2::new(world.x - point.x, world.z - point.z).length();
                        if distance >= radius {
                            continue;
                        }
                        let falloff = (1. - distance / radius).powi(2);
                        let old = tile.samples[index] as f32;
                        let next = if tool == 0 {
                            old + y_step * falloff * if lower { -1. } else { 1. }
                        } else {
                            old + (local_target - old) * (strength * falloff).min(1.)
                        };
                        let next = next.round().clamp(0., 65535.) as u16;
                        changed |= next != tile.samples[index];
                        tile.samples[index] = next;
                    }
                }
            } else if let Some(layer) = tile.descriptor["splat"]["layers"]
                .as_array()
                .unwrap()
                .iter()
                .position(|l| l["trueTextureName"].as_str() == texture.as_deref())
            {
                let res = tile.descriptor["splat"]["resolution"].as_u64().unwrap() as usize;
                let layers = tile.descriptor["splat"]["layers"].as_array().unwrap().len();
                for row in 0..res {
                    for col in 0..res {
                        let world = tile.transform.transform_point3(Vec3::new(
                            -(col as f32) / (res - 1).max(1) as f32 * extent.x,
                            0.,
                            (1. - row as f32 / (res - 1).max(1) as f32) * extent.y,
                        ));
                        let distance = Vec2::new(world.x - point.x, world.z - point.z).length();
                        if distance >= radius {
                            continue;
                        }
                        let i = (row * res + col) * 4;
                        let mut old: Vec<_> = (0..layers)
                            .map(|c| f32::from(tile.weights[c / 4][i + c % 4]))
                            .collect();
                        let sum = old.iter().sum::<f32>();
                        if sum > 0. {
                            for value in &mut old {
                                *value *= 255. / sum;
                            }
                        } else {
                            old[layer] = 255.;
                        }
                        let amount = (strength * (1. - distance / radius).powi(2)).clamp(0., 1.);
                        let mut values: Vec<u8> = old
                            .iter()
                            .enumerate()
                            .map(|(c, v)| {
                                (v * (1. - amount) + if c == layer { 255. * amount } else { 0. })
                                    .round()
                                    .clamp(0., 255.) as u8
                            })
                            .collect();
                        normalize_weights(&mut values, layer);
                        for c in 0..layers {
                            changed |= tile.weights[c / 4][i + c % 4] != values[c];
                            tile.weights[c / 4][i + c % 4] = values[c];
                        }
                    }
                }
            }
            if changed {
                tile.collider = NativeHeightmapCollider::from_samples(
                    &serde_json::from_value(tile.descriptor.clone()).map_err(|e| e.to_string())?,
                    &tile.samples,
                )
                .map_err(|e| e.to_string())?;
                tile.revision += 1;
            }
        }
        self.brush_cursor = Some(point);
        Ok(())
    }
    pub(super) fn finish_brush(&mut self) -> Result<(), String> {
        self.finish_grass();
        let Some(stroke) = self.brush.take() else {
            return Ok(());
        };
        let mut patches = vec![];
        for i in stroke.sources {
            let path = &self.sources[i].path;
            let mut after = None;
            for tile in self.ground.values() {
                let parent = tile.path.parent().unwrap();
                if path == &parent.join(tile.descriptor["heightmap"]["path"].as_str().unwrap()) {
                    after = Some(pixels(
                        tile.width as u32,
                        tile.height as u32,
                        "gray16",
                        &tile.samples,
                    ));
                    break;
                }
                for (map, weights) in tile.descriptor["splat"]["weightMaps"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .zip(&tile.weights)
                {
                    if path == &parent.join(map["path"].as_str().unwrap()) {
                        let res = tile.descriptor["splat"]["resolution"].as_u64().unwrap() as u32;
                        after = Some(pixels(res, res, "rgba8", weights));
                    }
                }
            }
            if let Some(after) = after {
                patches.push(Patch {
                    source: i,
                    pointer: "/pixels".into(),
                    before: Some(self.sources[i].draft["pixels"].clone()),
                    after: Some(after["pixels"].clone()),
                });
            }
        }
        self.commit(patches)
    }
    pub(super) fn prepare_terrain_publish(&mut self) -> Result<(), String> {
        self.finish_brush()?;
        let ids: Vec<_> = self.ground.keys().cloned().collect();
        for id in ids {
            let tile = &self.ground[&id];
            let parent = tile.path.parent().unwrap();
            if !self.sources.iter().any(|s| {
                s.path.starts_with(parent)
                    && s.draft["schema"] == "ffone.editor-pixels.v1"
                    && s.base != s.draft
            }) {
                continue;
            }
            let mut descriptor = tile.descriptor.clone();
            terrain_create::update_height_metadata(&mut descriptor,&tile.samples)?;
            let path = tile.path.clone();
            let height = parent.join(descriptor["heightmap"]["path"].as_str().unwrap());
            if let Some(source) = self
                .sources
                .iter()
                .find(|s| s.path == height && s.base != s.draft)
            {
                descriptor["heightmap"]["pngBlake3"] = hash(&pixel_bytes(&source.draft)?);
                descriptor["heightmap"]["canonicalOrderRawBlake3"] = hash(&raw16(&tile.samples));
                let mut transposed = Vec::with_capacity(tile.samples.len());
                for col in 0..tile.width {
                    for row in 0..tile.height {
                        transposed.push(tile.samples[row * tile.width + col]);
                    }
                }
                descriptor["heightmap"]["sourceOrderRawBlake3"] = hash(&raw16(&transposed));
                descriptor["heightmap"]["rawMin"] =
                    Value::from(*tile.samples.iter().min().unwrap());
                descriptor["heightmap"]["rawMax"] =
                    Value::from(*tile.samples.iter().max().unwrap());
            }
            // Every declared control-map mip is regenerated from the painted base.
            let environment = self.maps.get(&id).and_then(|map| {
                self.sources[map.scene].draft["nativeTerrain"]["environment"]["blake3"].as_str()
            });
            let native = if let Some(native)=&tile.authored { native.clone() } else { NativeTerrain::open_with_authoritative_environment(
                &self.root,
                path.strip_prefix(&self.root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/")
                    .as_str(),
                &blake3::hash(&fs::read(&path).map_err(|e| e.to_string())?)
                    .to_hex()
                    .to_string(),
                environment,
            )
            .map_err(|e| e.to_string())? };
            let edited = native
                .with_editor_data(&tile.samples, &tile.weights)
                .map_err(|e| e.to_string())?;
            let mips = edited.editor_weight_mips();
            let parent = parent.to_path_buf();
            for (map, chain) in descriptor["splat"]["weightMaps"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .zip(mips)
            {
                let base_path = parent.join(map["path"].as_str().ok_or("Missing weight map")?);
                if !self
                    .sources
                    .iter()
                    .any(|s| s.path == base_path && s.base != s.draft)
                {
                    continue;
                }
                for mip in map["mips"].as_array_mut().unwrap() {
                    let level = mip["level"].as_u64().unwrap() as usize;
                    let (width, height, raw) = &chain[level];
                    let value = pixels(*width, *height, "rgba8", raw);
                    let bytes = pixel_bytes(&value)?;
                    mip["pngBlake3"] = hash(&bytes);
                    mip["canonicalRgbaBlake3"] = hash(raw);
                    let i = self.terrain_source(parent.join(mip["path"].as_str().unwrap()))?;
                    self.sources[i].draft = value;
                }
                let first = map["mips"][0].clone();
                map["pngBlake3"] = first["pngBlake3"].clone();
                map["canonicalRgbaBlake3"] = first["canonicalRgbaBlake3"].clone();
            }
            let bytes = source_bytes(&descriptor)?;
            let digest = blake3::hash(&bytes).to_hex().to_string();
            let i = self.terrain_source(path)?;
            self.sources[i].draft = descriptor;
            if let Some(map) = self.maps.get(&id) {
                self.sources[map.scene].draft["nativeTerrain"]["blake3"] = Value::from(digest);
            }
        }
        Ok(())
    }
}

pub(super) fn preview(
    mut commands: Commands,
    mut e: ResMut<WorldEditor>,
    mut world: ResMut<preview::WorldPreview>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    materials: Res<Assets<NativeTerrainMaterial>>,
    mut query: Query<(
        &Mesh3d,
        &MeshMaterial3d<NativeTerrainMaterial>,
        &mut NativeHeightmapCollider,
    )>,
) {
    let ids: Vec<_> = world.terrains.keys().cloned().collect();
    for id in ids {
        let Some(tile) = e.ground.get(&id) else {
            continue;
        };
        if world.terrain_applied.get(&id) == Some(&tile.revision) {
            continue;
        }
        let Some(native) = world.terrain_data.get(&id) else {
            continue;
        };
        let entity = world.terrains[&id];
        let Ok((mesh, material, mut collider)) = query.get_mut(entity) else {
            continue;
        };
        match native.with_editor_data(&tile.samples, &tile.weights) {
            Ok(edited) => {
                if let Some(mut asset) = meshes.get_mut(&mesh.0) {
                    *asset = edited.editor_mesh();
                }
                if let Some(material) = materials.get(&material.0) {
                    if let Some(mut image) = images.get_mut(&material.weight_maps) {
                        *image = edited.editor_weight_image();
                    }
                }
                commands
                    .entity(entity)
                    .remove::<bevy::camera::primitives::Aabb>();
                *collider = NativeHeightmapCollider::from_terrain(&edited, mesh.0.clone());
                world.terrain_applied.insert(id, tile.revision);
            }
            Err(error) => e.error(error.to_string()),
        }
    }
}

pub(super) fn cursor(e: Res<WorldEditor>, state: Res<EditorState>, mut gizmos: Gizmos) {
    if state.world_open != Some(true) || e.map_picker || e.terrain_tool.is_none() {
        return;
    }
    if let Some(point) = e.brush_cursor {
        let point = Vec3::new(
            point.x,
            e.ground_height(point).unwrap_or(point.y) + 0.15,
            point.z,
        );
        gizmos
            .circle(
                Isometry3d::new(point, Quat::from_rotation_x(std::f32::consts::FRAC_PI_2)),
                e.brush_radius,
                Color::srgb(1., 0.8, 0.1),
            )
            .resolution(64);
    }
}
