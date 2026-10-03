use super::*;

impl RigQueryIndex {
    pub(super) fn rebuild(&mut self, entities: impl Iterator<Item = Entity>) {
        self.epoch = self.epoch.wrapping_add(1);
        if self.epoch == 0 {
            self.ranks.clear();
            self.epoch = 1;
        }
        for (rank, entity) in entities.enumerate() {
            let index = entity.index().index() as usize;
            if self.ranks.len() <= index {
                self.ranks.resize(index + 1, (Entity::PLACEHOLDER, 0, 0));
            }
            self.ranks[index] = (entity, rank, self.epoch);
        }
    }

    pub(super) fn rank(&self, entity: Entity) -> Option<usize> {
        let (owner, rank, epoch) = *self.ranks.get(entity.index().index() as usize)?;
        (owner == entity && epoch == self.epoch).then_some(rank)
    }
}

/// Native semantic animation additions have no Unity object identities.
pub(super) fn append_vehicle_animation_catalog(
    contract: &mut PlayerSharedRigContract,
    locator: &AssetLocator,
) -> Result<(), String> {
    let names: Vec<_> = crate::tutorial_player_presentation::TutorialPlayerClip::ALL
        .into_iter()
        .filter(|clip| clip.is_vehicle())
        .map(|clip| clip.name())
        .collect();
    append_native_animation_catalog(
        contract,
        locator,
        "characters/player/shared/vehicle_animations.json",
        &names,
    )
}

pub(super) fn append_native_animation_catalog(
    contract: &mut PlayerSharedRigContract,
    locator: &AssetLocator,
    path: &str,
    expected_names: &[&str],
) -> Result<(), String> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase", deny_unknown_fields)]
    struct Clip {
        name: String,
        gltf_animation_index: u32,
        channel_count: u32,
        duration_seconds: f64,
        playback: String,
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Gender {
        gender: PlayerRigGender,
        clips: Vec<Clip>,
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Catalog {
        schema: String,
        genders: Vec<Gender>,
    }
    let bytes = locator.read(path)?;
    let catalog: Catalog = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    if catalog.schema != "ffone.player-animation-catalog.v1" || catalog.genders.len() != 2 {
        return Err(format!("invalid animation catalog {path}"));
    }
    let mut seen = BTreeSet::new();
    for entry in catalog.genders {
        if !seen.insert(player_rig_gender_key(entry.gender)) {
            return Err(format!("duplicate animation gender in {path}"));
        }
        let gender = contract
            .genders
            .iter_mut()
            .find(|g| g.gender == entry.gender)
            .ok_or("missing native actor rig")?;
        let mut names = BTreeSet::new();
        let mut indices: BTreeSet<_> = gender
            .clips
            .iter()
            .map(|c| c.gltf_animation_index)
            .collect();
        for clip in entry.clips {
            let semantic =
                crate::tutorial_player_presentation::TutorialPlayerClip::from_exact_name(
                    &clip.name,
                )
                .filter(|c| expected_names.contains(&c.name()))
                .ok_or_else(|| format!("unknown clip in {path}"))?;
            if !names.insert(clip.name.clone())
                || !indices.insert(clip.gltf_animation_index)
                || clip.channel_count == 0
                || !clip.duration_seconds.is_finite()
                || clip.duration_seconds <= 0.0
                || semantic.playback().contract_value() != clip.playback
            {
                return Err(format!("invalid or duplicate clip in {path}"));
            }
            gender
                .clips
                .push(ffone_runtime_contracts::PlayerRigClipContract {
                    name: clip.name,
                    source_path_id: 0,
                    gltf_animation_index: clip.gltf_animation_index,
                    channel_count: clip.channel_count,
                    source_key_count: 0,
                    duration_seconds_bits: clip.duration_seconds.to_bits(),
                    playback: clip.playback,
                    runtime_status: "native-bevy-ready".into(),
                });
        }
        if names.len() != expected_names.len() {
            return Err(format!("incomplete animation catalog {path}"));
        }
    }
    Ok(())
}
