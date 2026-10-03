use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Component)]
pub(super) struct PreviewMaterialAnimationBase {
    pub(super) base_color: LinearRgba,
    pub(super) uv_scale_offset: Vec4,
}

pub(super) fn apply_preview_material_animation_curves(
    mut commands: Commands,
    config: Res<PreviewConfig>,
    state: Res<RuntimeState>,
    players: Query<&AnimationPlayer>,
    parents: Query<&ChildOf>,
    names: Query<&Name>,
    surfaces: Query<
        (
            Entity,
            &MeshMaterial3d<LegacyModelMaterial>,
            Option<&PreviewMaterialAnimationBase>,
        ),
        With<LegacyMaterialApplied>,
    >,
    mut materials: ResMut<Assets<LegacyModelMaterial>>,
) {
    let Some(selection) = config.animation.as_deref() else {
        return;
    };
    let Some(prepared) = state.prepared_animation.as_ref() else {
        return;
    };
    let Some(sample_time) = players.iter().find_map(|player| {
        player
            .animation(prepared.node)
            .map(|active| active.seek_time())
    }) else {
        return;
    };
    let Some(clip) = config
        .material_animation_clips
        .iter()
        .find(|clip| clip.name == selection)
    else {
        return;
    };
    for (entity, handle, baseline) in &surfaces {
        let matching_curves = clip
            .float_curves
            .iter()
            .filter(|curve| {
                surface_has_preview_named_ancestor(
                    entity,
                    curve
                        .target_path
                        .rsplit('/')
                        .next()
                        .unwrap_or(&curve.target_path),
                    &parents,
                    &names,
                )
            })
            .collect::<Vec<_>>();
        if matching_curves.is_empty() && baseline.is_none() {
            continue;
        }
        let Some(source_material) = materials.get(&handle.0) else {
            continue;
        };
        let had_baseline = baseline.is_some();
        let baseline = baseline.copied().unwrap_or(PreviewMaterialAnimationBase {
            base_color: source_material.uniform.base_color,
            uv_scale_offset: source_material.uniform.uv_scale_offset,
        });
        let mut material = source_material.clone();
        material.uniform.base_color = baseline.base_color;
        material.uniform.uv_scale_offset = baseline.uv_scale_offset;
        for curve in matching_curves {
            if let Some(value) =
                sample_legacy_float_curve(curve, sample_time, clip.duration, clip.looped)
            {
                apply_legacy_material_float(&mut material, &curve.property, value);
            }
        }
        if !had_baseline {
            commands
                .entity(entity)
                .insert((MeshMaterial3d(materials.add(material)), baseline));
        } else if let Some(mut existing) = materials.get_mut(&handle.0) {
            *existing = material;
        }
    }
}

#[derive(Clone, Resource)]
pub(super) struct CapturedRenderErrors(pub(super) Arc<Mutex<Vec<String>>>);

impl CapturedRenderErrors {
    pub(super) fn messages(&self) -> Vec<String> {
        self.0
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }
}

pub(super) struct RenderErrorCaptureLayer {
    pub(super) errors: Arc<Mutex<Vec<String>>>,
}

impl<S: Subscriber> Layer<S> for RenderErrorCaptureLayer {
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _context: bevy::log::tracing_subscriber::layer::Context<'_, S>,
    ) {
        let metadata = event.metadata();
        let mut visitor = RenderLogVisitor::default();
        event.record(&mut visitor);
        let message = visitor.fields.join(", ");
        if !is_render_error(metadata.level(), metadata.target(), &message) {
            return;
        }
        let mut errors = self
            .errors
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if errors.len() < 16 {
            errors.push(format!("{}: {message}", metadata.target()));
        }
    }
}

#[derive(Default)]
pub(super) struct RenderLogVisitor {
    pub(super) fields: Vec<String>,
}

impl tracing::field::Visit for RenderLogVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        self.fields.push(format!("{}={value:?}", field.name()));
    }
}

pub(super) fn is_render_error(level: &Level, target: &str, message: &str) -> bool {
    let target = target.to_ascii_lowercase();
    let message = message.to_ascii_lowercase();
    let acceptance_target = target.starts_with("bevy_asset")
        || target.starts_with("bevy_gltf")
        || target.starts_with("bevy_render")
        || target.starts_with("wgpu")
        || target.starts_with("naga")
        || target.contains("pipeline_cache");
    let shader_words = message.contains("shader")
        || message.contains("wgsl")
        || message.contains("pipeline")
        || message.contains("validation error");
    let missing_asset = message.contains("missing labeled asset")
        || message.contains("does not contain texture")
        || message.contains("does not contain image")
        || message.contains("failed to load")
        || message.contains("asset load failed");
    (*level == Level::ERROR && (acceptance_target || shader_words || missing_asset))
        || (*level == Level::WARN && acceptance_target && shader_words)
}

pub(super) fn render_error_capture_layer(app: &mut App) -> Option<BoxedLayer> {
    let errors = Arc::new(Mutex::new(Vec::new()));
    app.insert_resource(CapturedRenderErrors(errors.clone()));
    Some(RenderErrorCaptureLayer { errors }.boxed())
}
