use super::*;

pub(super) const PRIMARY_EFFECTS_ASSET: &str = "CustomAssetBundle-fa9dbcf4604f64024b06ff1d5e375918";

impl TutorialEffectRuntime {
    pub fn load_native_particle_catalog(&mut self, locator: &crate::assets::AssetLocator) -> Result<(), String> {
        let catalog = tutorial_native_effects::native_catalog::open(locator)?;
        let projectiles = native_skill_projectiles::open(locator, &catalog)?;
        self.native_catalog = catalog;
        self.native_skill_projectiles = projectiles;
        Ok(())
    }

    pub fn with_library(library: TutorialEffectLibrary) -> Self {
        Self {
            library: Some(Arc::new(library)),
            ..Self::default()
        }
    }

    pub fn install_library(&mut self, library: TutorialEffectLibrary) {
        self.install_shared_library(Arc::new(library));
    }

    pub fn install_shared_library(&mut self, library: Arc<TutorialEffectLibrary>) {
        self.library = Some(library);
        self.preloaded.clear();
        self.native_preload_complete.clear();
        self.native_preload_failures_reported.clear();
        self.active.clear();
        self.tracked.clear();
        self.named.clear();
        self.native_preloads.clear();
        self.native_spawns.clear();
        self.native_despawns.clear();
    }

    pub fn library(&self) -> Option<&TutorialEffectLibrary> {
        self.library.as_deref()
    }

    pub fn enqueue(&mut self, command: TutorialEffectRuntimeCommand) {
        self.pending.push_back(command);
    }

    /// Queue an ambient effect owned by a streamed world root.
    ///
    /// These commands intentionally use a separate admission queue. Tutorial,
    /// combat and UI effects retain their immediate source ordering while a
    /// newly loaded tile cannot compile hundreds of ambient ES closures in one
    /// frame.
    pub fn enqueue_streamed_world_effect(&mut self, command: TutorialEffectRuntimeCommand) {
        debug_assert!(matches!(
            &command,
            TutorialEffectRuntimeCommand::Add { placement, .. }
                if placement.stream_owner().is_some()
        ));
        self.streamed_world_pending.push_back(command);
    }

    /// Queue an exact serialized `EffectEmitterController` recovered from a
    /// streamed world tile. These closures are not numeric EP effects and do
    /// not belong in the tutorial catalog; they still use the same audited
    /// Unity particle compiler and renderer.
    pub fn enqueue_world_serialized_effect(
        &mut self,
        closure: TutorialEffectClosureFile,
        placement: TutorialEffectPlacement,
        scale: f32,
    ) -> Result<u64, String> {
        if closure.schema != TUTORIAL_EFFECT_CLOSURE_SCHEMA {
            return Err(format!(
                "world effect closure uses schema {:?}, expected {TUTORIAL_EFFECT_CLOSURE_SCHEMA}",
                closure.schema
            ));
        }
        if !placement.is_valid() || !scale.is_finite() || scale <= 0.0 {
            return Err("world effect closure has an invalid placement or scale".to_owned());
        }
        let compiled = tutorial_native_effects::compile_world_emitter_plan(closure);
        if !compiled.blockers.is_empty() {
            let details = compiled
                .blockers
                .iter()
                .map(|blocker| {
                    format!(
                        "{}:{} {}: {:?}",
                        blocker.asset, blocker.path_id, blocker.object_type, blocker.reason
                    )
                })
                .collect::<Vec<_>>()
                .join("; ");
            return Err(format!(
                "world effect closure has unsupported serialized nodes: {details}"
            ));
        }
        let plan = compiled
            .plan
            .ok_or_else(|| "world effect closure produced no native plan".to_owned())?;
        let instance_id = self.allocate_instance(false, None);
        self.active
            .get_mut(&instance_id)
            .expect("new native world effect instance is active")
            .stream_owner = placement.stream_owner();
        // Streamed ambient effects are untracked and do not participate in the
        // tutorial preload gate. Their particle textures are materialized by
        // the emitter on first visible use; eagerly decoding every effect in
        // all five resident tiles made ordinary-world entry pay for distant
        // particles that the gameplay camera cannot display.
        self.native_spawns
            .push_back(tutorial_native_effects::NativeSpawnRequest::Effect {
                instance_id,
                effect_id: i32::MIN,
                streamed_world: true,
                placement,
                scale,
                name: None,
                destroy_after_seconds: None,
                source_line: 0,
                plan,
            });
        Ok(instance_id)
    }

    pub(super) fn retain_live_streamed_world_commands(&mut self, owner_is_live: impl Fn(Entity) -> bool) {
        self.streamed_world_pending.retain(|command| {
            let TutorialEffectRuntimeCommand::Add { placement, .. } = command else {
                return false;
            };
            placement
                .stream_owner()
                .is_some_and(|owner| owner_is_live(owner))
        });
    }

    pub fn process_pending(&mut self) {
        self.process_pending_with_stream_liveness(|_| true);
    }

    pub(super) fn process_pending_with_stream_liveness(&mut self, owner_is_live: impl Fn(Entity) -> bool) {
        // Legacy gameplay calls instantiate synchronously. Preserve that
        // ordering before spending this frame's bounded ambient allowance.
        while let Some(command) = self.pending.pop_front() {
            self.process_one(command, false);
        }
        // A tile can leave the 340-unit unload band while hundreds of its EP
        // commands are still queued. Purge those commands before they consume
        // the eight-command ambient allowance and delay effects from live
        // tiles for dozens of frames.
        self.retain_live_streamed_world_commands(owner_is_live);
        for _ in 0..STREAMED_WORLD_EFFECT_COMMANDS_PER_FRAME {
            let Some(command) = self.streamed_world_pending.pop_front() else {
                break;
            };
            self.process_one(command, true);
        }
    }

    pub fn drain_issues(&mut self) -> impl Iterator<Item = TutorialEffectRuntimeIssue> + '_ {
        self.issues.drain(..)
    }

    pub fn drain_records(&mut self) -> impl Iterator<Item = TutorialEffectRuntimeRecord> + '_ {
        self.records.drain(..)
    }

    pub fn is_serialized_closure_preloaded(&self, effect_id: i32) -> bool {
        self.preloaded.contains(&effect_id)
    }

    /// Mirrors the terminal state of Retrobution's `AssetBundleRequest`.
    ///
    /// Compiling and retaining the closure starts the native preload, but a
    /// mesh-backed effect is not complete until its GLTF scene and recursive
    /// dependencies have either loaded or reached a terminal failure.
    pub fn is_native_preload_complete(&self, effect_id: i32) -> bool {
        self.native_preload_complete.contains(&effect_id)
    }

    pub(in super::super) fn mark_native_preload_complete(&mut self, effect_id: i32) {
        self.native_preload_complete.insert(effect_id);
    }

    pub(in super::super) fn mark_native_preload_failed(
        &mut self,
        effect_id: i32,
        asset_path: impl Into<String>,
        detail: impl Into<String>,
    ) {
        if self.native_preload_failures_reported.insert(effect_id) {
            self.issues
                .push_back(TutorialEffectRuntimeIssue::NativePreloadFailed {
                    effect_id,
                    asset_path: asset_path.into(),
                    detail: detail.into(),
                });
        }
        // Retrobution's request coroutine is terminal on failure. Choreography
        // must resume, but the renderer failure remains observable above.
        self.mark_native_preload_complete(effect_id);
    }

    /// Returns whether the exact serialized closure can synchronously produce
    /// a native effect instance.
    ///
    /// Retrobution's `InstantiateEffect` returns its root object immediately;
    /// child meshes and textures may continue loading afterwards. The finale
    /// retry loops therefore gate on plan construction, not on Bevy's later
    /// scene-asset readiness.
    pub fn can_instantiate_native_effect(&self, effect_id: i32) -> bool {
        self.library
            .as_ref()
            .and_then(|library| library.effects.get(&effect_id))
            .is_some_and(|effect| {
                tutorial_native_effects::compile_effect_plan(effect_id, effect)
                    .plan
                    .is_some()
            })
    }

    pub fn active_native_instance_count(&self) -> usize {
        self.active.len()
    }

    pub fn has_named_native_instance(&self, name: &str) -> bool {
        self.named
            .get(name)
            .is_some_and(|instance_id| self.active.contains_key(instance_id))
    }

    /// A queued/replacement instance is not a playing effect. Gameplay clocks
    /// must wait for the renderer's mesh-surface barrier before counting time.
    pub fn named_native_presentation_ready(&self, name: &str) -> bool {
        if self.pending.iter().any(|command| {
            matches!(command,
            TutorialEffectRuntimeCommand::Add { name: Some(pending_name), .. }
                if pending_name == name)
        }) {
            return false;
        }
        self.named
            .get(name)
            .and_then(|id| self.active.get(id))
            .is_some_and(|instance| instance.presentation_ready)
    }

    /// Renderer acknowledgement for an instantiated effect whose playback
    /// barrier has opened. The acknowledgement lives only with that instance.
    pub fn mark_native_presentation_ready(&mut self, instance_id: u64) {
        if let Some(instance) = self.active.get_mut(&instance_id) {
            instance.presentation_ready = true;
        }
    }

    /// Clears scene-owned instances while retaining app-lifetime preload
    /// caches, matching Unity scene teardown after tutorial exit.
    pub fn clear_scene_instances(&mut self) {
        self.pending.clear();
        self.streamed_world_pending.clear();
        self.native_spawns.clear();
        for instance_id in std::mem::take(&mut self.active).into_keys() {
            self.native_despawns.push_back(instance_id);
        }
        self.tracked.clear();
        self.named.clear();
        self.issues.clear();
        self.records.clear();
    }

    pub(super) fn allocate_instance(&mut self, tracked: bool, name: Option<String>) -> u64 {
        self.next_instance_id = self.next_instance_id.saturating_add(1).max(1);
        let id = self.next_instance_id;
        self.active.insert(
            id,
            ActiveNativeInstance {
                name: name.clone(),
                tracked,
                stream_owner: None,
                root_entity: None,
                presentation_ready: false,
            },
        );
        if tracked {
            self.tracked.insert(id);
        }
        if let Some(name) = name {
            if let Some(replaced) = self.named.insert(name, id) {
                self.forget_instance(replaced);
                self.native_despawns.push_back(replaced);
            }
        }
        id
    }

    pub(in super::super) fn bind_native_root(&mut self, id: u64, root_entity: Entity) {
        if let Some(instance) = self.active.get_mut(&id) {
            instance.root_entity = Some(root_entity);
        }
    }

    pub(super) fn reconcile_named_native_roots(&mut self, live_roots: &BTreeMap<u64, Entity>) -> Vec<u64> {
        let stale = self
            .active
            .iter()
            .filter_map(|(id, instance)| {
                let root_entity = instance.root_entity?;
                (instance.name.is_some() && live_roots.get(id) != Some(&root_entity)).then_some(*id)
            })
            .collect::<Vec<_>>();
        for id in &stale {
            self.forget_instance(*id);
            self.native_despawns.push_back(*id);
        }
        stale
    }

    pub(super) fn forget_instance(&mut self, id: u64) -> bool {
        let Some(meta) = self.active.remove(&id) else {
            return false;
        };
        if meta.tracked {
            self.tracked.remove(&id);
        }
        if let Some(name) = meta.name {
            if self.named.get(&name) == Some(&id) {
                self.named.remove(&name);
            }
        }
        true
    }

    pub(super) fn push_blockers(
        &mut self,
        effect_id: i32,
        source_line: u32,
        blockers: &[tutorial_native_effects::NativeClosureBlocker],
    ) {
        for blocker in blockers {
            self.issues.push_back(
                TutorialEffectRuntimeIssue::UnsupportedSerializedClosureNode {
                    effect_id,
                    asset: blocker.asset.clone(),
                    path_id: blocker.path_id,
                    object_type: blocker.object_type.clone(),
                    reason: blocker.reason.clone(),
                    source_line,
                },
            );
        }
    }

    pub(super) fn compile_available_effect(&self, effect_id: i32) -> Option<tutorial_native_effects::NativeCompileResult<tutorial_native_effects::NativeEffectPlan>> {
        if let Some(plan) = self.native_catalog.get(&effect_id) {
            return Some(tutorial_native_effects::NativeCompileResult { plan: Some(plan.clone()), blockers: Vec::new() });
        }
        self.library.as_ref()?.effects.get(&effect_id)
            .map(|effect| tutorial_native_effects::compile_effect_plan(effect_id, effect))
    }
}

pub(super) fn safe_asset_path(root: &Path, relative: &str) -> Result<PathBuf, TutorialEffectLibraryError> {
    let normalized = relative.replace('\\', "/");
    if normalized != relative
        || relative.starts_with('/')
        || relative
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == ".." || part.contains(':'))
    {
        return fail(format!("unsafe tutorial asset path {relative:?}"));
    }
    let mut path = root.to_path_buf();
    for part in relative.split('/') {
        path.push(part);
    }
    Ok(path)
}
