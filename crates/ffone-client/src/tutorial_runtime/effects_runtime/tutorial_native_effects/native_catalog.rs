//! Editable native particle definitions. No source containers or serialized
//! object graphs are needed to load these effects.
use super::*;
use crate::assets::AssetLocator;
use serde::Deserialize;

const CATALOG: &str = "effects/shinies/catalog.json";
const PICKUP_MESH: &str = "effects/shinies/pickup/shineni_ai.glb";
const SKILL_MESHES: &[&str] = &[
    "effects/npc-skills/mega/mega.glb",
    "effects/skills/jump_target_e02/jump_target_e02.glb",
    "effects/skills/protection_target_e01/protection_target_e01.glb",
    "effects/skills/protection_target_e02/protection_target_e02.glb",
    "effects/skills/nano_rd_e01/nano_rd_e01.glb",
    "effects/skills/k_rewardcash_self1/k_rewardcash_self1.glb",
    "effects/skills/freedom_self/freedom_self.glb",
    "effects/skills/sleep_target_e01/sleep_target_e01.glb",
];

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Catalog {
    schema: String,
    effects: Vec<Effect>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Effect {
    id: i32,
    name: String,
    maximum_timer: f32,
    longest_lifetime: f32,
    disable_update: bool,
    emitters: Vec<Emitter>,
    mesh: Option<String>,
    #[serde(default)]
    material_animation: Option<NativeMaterialAnimation>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeMaterialAnimation {
    duration: f32,
    curves: Vec<NativeMaterialCurve>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NativeMaterialCurve {
    node_name: String,
    property: MaterialAnimatedProperty,
    keys: Vec<[f32; 4]>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Key {
    time: f32,
    translation: [f32; 3],
    emit: bool,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Emitter {
    name: String,
    generation: i64,
    random_position: f32,
    random_angle: f32,
    random_velocity: f32,
    // The particle generator applies its authored local random rotations
    // before reflecting the completed vector into native coordinates.
    generator_velocity: [f32; 3],
    generator_plane: [f32; 3],
    script_keys: Vec<Key>,
    generations_per_second: f32,
    number_per_generation: f32,
    lifetime: f32,
    initial_size: f32,
    force: [f32; 3],
    colors: [[f32; 4]; 5],
    animate_color: bool,
    uv_tiles: [u32; 2],
    uv_cycles: f32,
    width_curve: Vec<[f32; 4]>,
    height_curve: Vec<[f32; 4]>,
    rotation_curve: Vec<[f32; 4]>,
    render_mode: i64,
    blend_mode: ParticleBlendMode,
    material_tint: [f32; 4],
    texture: Vec<String>,
}

fn curve(keys: Vec<[f32; 4]>) -> Result<AnimationCurve, String> {
    if keys.iter().flatten().any(|v| !v.is_finite()) || keys.windows(2).any(|p| p[0][0] > p[1][0]) {
        return Err("invalid native particle curve".into());
    }
    Ok(AnimationCurve(
        keys.into_iter()
            .map(|[time, value, in_slope, out_slope]| CurveKey {
                time,
                value,
                in_slope,
                out_slope,
            })
            .collect(),
    ))
}

pub(in crate::tutorial_runtime::effects_runtime) fn open(
    locator: &AssetLocator,
) -> Result<BTreeMap<i32, NativeEffectPlan>, String> {
    let mut effects = Vec::new();
    for path in [CATALOG, "effects/skills/catalog.json", "effects/npc-skills/catalog.json", "effects/skill-hits/catalog.json"] {
        let bytes = std::fs::read(locator.path(path)?).map_err(|e| e.to_string())?;
        let catalog: Catalog = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
        if catalog.schema != "ffone.native-particle-effects.v1" {
            return Err("unknown native effects schema".into());
        }
        effects.extend(catalog.effects);
    }
    let mut plans = BTreeMap::new();
    let mut textures = BTreeMap::<Vec<String>, ExactTexture>::new();
    for effect in effects {
        if effect.name.is_empty()
            || !effect.maximum_timer.is_finite()
            || !effect.longest_lifetime.is_finite()
            || effect.longest_lifetime < 0.0
        {
            return Err("invalid native effect lifetime/name".into());
        }
        // The shared renderer currently registers mesh effects by stable route.
        let mesh_scene = match effect.mesh.as_deref() {
            None => None,
            Some(PICKUP_MESH) => Some(PICKUP_MESH),
            Some(path) => Some(
                SKILL_MESHES
                    .iter()
                    .copied()
                    .find(|candidate| *candidate == path)
                    .ok_or_else(|| "unregistered native effect mesh".to_owned())?,
            ),
        };
        if let Some(path) = mesh_scene {
            locator.path(path)?;
        }
        let mut emitters = Vec::new();
        for (index, e) in effect.emitters.into_iter().enumerate() {
            let numeric = [
                e.random_position,
                e.random_angle,
                e.random_velocity,
                e.generations_per_second,
                e.number_per_generation,
                e.lifetime,
                e.initial_size,
                e.uv_cycles,
            ];
            if numeric.iter().any(|v| !v.is_finite() || *v < 0.0)
                || e.lifetime == 0.0
                || e.generations_per_second == 0.0
                || e.initial_size == 0.0
                || e.uv_tiles.contains(&0)
                || e.uv_cycles == 0.0
                || e.script_keys.is_empty()
                || e.script_keys.windows(2).any(|p| p[0].time > p[1].time)
                || e.script_keys.iter().any(|k| {
                    !k.time.is_finite()
                        || k.time < 0.0
                        || k.translation.iter().any(|v| !v.is_finite())
                })
                || e.generator_velocity
                    .iter()
                    .chain(&e.generator_plane)
                    .chain(&e.force)
                    .chain(e.colors.iter().flatten())
                    .chain(&e.material_tint)
                    .any(|v| !v.is_finite())
            {
                return Err(format!("invalid native emitter {}", e.name));
            }
            let texture = if let Some(t) = textures.get(&e.texture) {
                t.clone()
            } else {
                if e.texture.is_empty() || e.texture.len() > 32 {
                    return Err("empty native particle texture chain".into());
                }
                let mut pixels = Vec::new();
                let mut size = (0, 0);
                for (level, path) in e.texture.iter().enumerate() {
                    let bytes = std::fs::read(locator.path(path)?).map_err(|e| e.to_string())?;
                    let image = image::load_from_memory(&bytes)
                        .map_err(|e| e.to_string())?
                        .to_rgba8();
                    if level == 0 {
                        size = (image.width(), image.height());
                    }
                    if image.dimensions() != ((size.0 >> level).max(1), (size.1 >> level).max(1)) {
                        return Err("invalid native particle mip dimensions".into());
                    }
                    pixels.extend_from_slice(image.as_raw());
                }
                let texture = ExactTexture {
                    key: (
                        e.texture[0].clone(),
                        0,
                        blake3::hash(&pixels).to_hex().to_string(),
                    ),
                    width: size.0,
                    height: size.1,
                    mip_count: e.texture.len() as u32,
                    rgba_mips: pixels.into(),
                };
                textures.insert(e.texture.clone(), texture.clone());
                texture
            };
            let keys: Vec<_> = e
                .script_keys
                .into_iter()
                .map(|k| ScriptKey {
                    time: k.time,
                    translate: Vec3::from_array(k.translation),
                    emit: k.emit,
                })
                .collect();
            emitters.push(EmitterPlan {
                source_asset: format!("{}/{}", effect.name, e.name),
                source_path_id: index as i64,
                generation: EmitterGeneration::try_from(e.generation)?,
                random_position: e.random_position,
                random_angle: e.random_angle,
                random_velocity: e.random_velocity,
                initial_velocity: Vec3::from_array(e.generator_velocity),
                plane: Vec3::from_array(e.generator_plane),
                initial_translation: keys[0].translate,
                initial_emit: keys[0].emit,
                script_keys: keys,
                generations_per_second: e.generations_per_second,
                number_per_generation: e.number_per_generation,
                lifetime: e.lifetime,
                initial_size: e.initial_size,
                force: Vec3::from_array(e.force),
                colors: e.colors.map(Vec4::from_array),
                animate_color: e.animate_color,
                uv_tiles: UVec2::from_array(e.uv_tiles),
                uv_cycles: e.uv_cycles,
                width_curve: curve(e.width_curve)?,
                height_curve: curve(e.height_curve)?,
                rotation_curve: curve(e.rotation_curve)?,
                render_mode: LegacyParticleRenderMode::try_from(e.render_mode)?,
                blend_mode: e.blend_mode,
                material_tint: Vec4::from_array(e.material_tint),
                legacy_gamma_accumulation_gain: 1.0,
                texture,
            });
        }
        let material_animation = effect
            .material_animation
            .map(|animation| {
                if !animation.duration.is_finite() || animation.duration <= 0.0 {
                    return Err("invalid native material animation duration".to_owned());
                }
                let curves = animation
                    .curves
                    .into_iter()
                    .map(|c| {
                        if c.node_name.is_empty() || c.keys.is_empty() {
                            return Err("empty native material animation binding".to_owned());
                        }
                        Ok(MaterialCurvePlan {
                            node_name: c.node_name,
                            property: c.property,
                            curve: curve(c.keys)?,
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                Ok(MaterialAnimationPlan {
                    duration: animation.duration,
                    curves,
                })
            })
            .transpose()?;
        let plan = NativeEffectPlan {
            rendered_nodes: emitters.len() + usize::from(mesh_scene.is_some()),
            emitters,
            mesh_scene,
            material_animation,
            maximum_timer: effect.maximum_timer,
            longest_lifetime: effect.longest_lifetime,
            disable_update: effect.disable_update,
        };
        if plans.insert(effect.id, plan).is_some() {
            return Err("duplicate native effect id".into());
        }
    }
    Ok(plans)
}

pub(in crate::tutorial_runtime::effects_runtime) fn load(
    locator: Option<Res<AssetLocator>>,
    mut runtime: ResMut<TutorialEffectRuntime>,
    mut attempted: Local<bool>,
) {
    let Some(locator) = locator else {
        return;
    };
    if *attempted {
        return;
    }
    *attempted = true;
    if let Err(error) = runtime.load_native_particle_catalog(&locator) {
        error!("Native particle effects could not load: {error}");
    }
}

#[cfg(test)]
mod tests;
