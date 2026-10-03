//! Native skill projectile contracts; no Unity containers are runtime inputs.
use super::*;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SkillProjectile {
    pub id: i32,
    pub particle: i32,
    pub impact: i32,
    pub scale: f32,
    pub impact_scale: f32,
    pub hide_seconds: f32,
    pub duration_seconds: f32,
    pub source_link: String,
    pub target_link: String,
    pub impact_sound: Option<String>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Catalog {
    schema: String,
    projectiles: Vec<SkillProjectile>,
}

pub(super) fn open(
    locator: &crate::assets::AssetLocator,
    effects: &BTreeMap<i32, tutorial_native_effects::NativeEffectPlan>,
) -> Result<BTreeMap<i32, SkillProjectile>, String> {
    let bytes = locator.read("effects/skill-hits/projectiles.json")?;
    let catalog: Catalog = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if catalog.schema != "ffone.native-skill-projectiles.v1" {
        return Err("unknown skill projectile schema".into());
    }
    let mut rows = BTreeMap::new();
    for row in catalog.projectiles {
        if row.id <= 0
            || row.duration_seconds <= 0.0
            || row.hide_seconds > row.duration_seconds
            || [
                row.scale,
                row.impact_scale,
                row.hide_seconds,
                row.duration_seconds,
            ]
            .iter()
            .any(|v| !v.is_finite() || *v < 0.0)
            || (row.particle != 0 && !effects.contains_key(&row.particle))
            || (row.impact != 0 && !effects.contains_key(&row.impact))
        {
            return Err(format!("invalid native skill projectile {}", row.id));
        }
        if let Some(path) = &row.impact_sound {
            locator.path(path)?;
        }
        if rows.insert(row.id, row).is_some() {
            return Err("duplicate native skill projectile".into());
        }
    }
    Ok(rows)
}
impl TutorialEffectRuntime {
    pub fn native_skill_projectile_links(&self, id: i32) -> Option<(&str, &str)> {
        self.native_skill_projectiles
            .get(&id)
            .map(|r| (r.source_link.as_str(), r.target_link.as_str()))
    }
    pub(super) fn queue_native_skill_projectile(
        &mut self,
        command: &TutorialEffectRuntimeCommand,
    ) -> bool {
        let TutorialEffectRuntimeCommand::Projectile {
            bullet_type,
            source,
            target,
            target_exists,
            source_style,
            target_style,
            motion,
            ..
        } = command
        else {
            return false;
        };
        let Some(row) = self.native_skill_projectiles.get(bullet_type).cloned() else {
            return false;
        };
        if !source.is_finite()
            || !target.is_finite()
            || !matches!(motion, TutorialProjectileMotion::BulletMove)
        {
            self.record(
                command.clone(),
                TutorialEffectRuntimeDisposition::RejectedFailClosed,
            );
            return true;
        }
        let carried_effect = self.native_catalog.get(&row.particle).cloned();
        let impact_plan = self.native_catalog.get(&row.impact).cloned();
        let rendered_nodes =
            carried_effect.as_ref().map_or(0, |p| p.rendered_nodes)
                + impact_plan.as_ref().map_or(0, |p| p.rendered_nodes);
        let impact = impact_plan.filter(|_| *target_exists).map(|plan| tutorial_native_effects::NativeLinearImpactPlan {
            instance_id: self.allocate_instance(false, None),
            effect_id: row.impact,
            scale: exact_projectile_success_scale(row.impact_scale, *source_style, *target_style),
            sound_path: row.impact_sound,
            plan,
        });
        let instance_id = self.allocate_instance(false, None);
        self.native_spawns.push_back(
            tutorial_native_effects::NativeSpawnRequest::LinearProjectile {
                instance_id,
                bullet_type: *bullet_type,
                effect_id: row.particle,
                source: *source,
                target: *target,
                scale: row.scale,
                motion: tutorial_native_effects::NativeLinearProjectileMotion::BulletMove {
                    hide_seconds: row.hide_seconds,
                    duration_seconds: row.duration_seconds,
                },
                impact,
                plan: None,
                carried_effect,
            },
        );
        self.record(
            command.clone(),
            TutorialEffectRuntimeDisposition::NativeProjectileQueued {
                instance_id,
                rendered_nodes,
                blocked_nodes: 0,
            },
        );
        true
    }
}

#[cfg(test)]
mod tests;
