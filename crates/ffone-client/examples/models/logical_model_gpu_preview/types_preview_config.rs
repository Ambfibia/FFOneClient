use super::*;

#[derive(Debug, Clone, Resource, PartialEq)]
pub(super) struct PreviewConfig {
    pub(super) asset_root: PathBuf,
    pub(super) model: String,
    pub(super) screenshot: PathBuf,
    pub(super) animation: Option<String>,
    pub(super) animation_exact_name: bool,
    pub(super) sample_midpoint: bool,
    pub(super) outline: PreviewOutlineMode,
    pub(super) max_frames: u64,
    pub(super) timeout: Duration,
    pub(super) evidence: Option<EvidenceConfig>,
    pub(super) character: Option<CharacterRuntimeConfig>,
    pub(super) main_texture: Option<String>,
    pub(super) sub_texture: Option<String>,
    pub(super) main_material: Option<String>,
    pub(super) sub_material: Option<String>,
    pub(super) main_sampler: Option<NativeSampler>,
    pub(super) sub_sampler: Option<NativeSampler>,
    pub(super) only_material: Option<String>,
    pub(super) camera_view: PreviewCameraView,
    pub(super) blank_camera_retry: BlankCameraRetry,
    pub(super) report: Option<PathBuf>,
    pub(super) runtime_smoke: Option<PathBuf>,
    pub(super) material_animation_clips: Arc<[LegacyMaterialAnimationClip]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CharacterKind {
    Npc,
    Nano,
    Player,
}

impl CharacterKind {
    pub(super) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "npc" => Ok(Self::Npc),
            "nano" => Ok(Self::Nano),
            "player" => Ok(Self::Player),
            _ => Err(format!(
                "invalid --character-kind value {value:?}; expected npc, nano, or player"
            )),
        }
    }

    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Npc => "npc",
            Self::Nano => "nano",
            Self::Player => "player",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct EvidenceConfig {
    pub(super) root: PathBuf,
    pub(super) json_path: PathBuf,
    pub(super) relative_json: String,
    pub(super) relative_png: String,
    pub(super) model_path: PathBuf,
    pub(super) glb_byte_length: u64,
    pub(super) glb_sha256: String,
    pub(super) facts: LogicalModelGpuFacts,
}

impl PreviewConfig {
    pub(super) fn parse(args: impl IntoIterator<Item = String>) -> Result<ParseOutcome, String> {
        let mut asset_root = None;
        let mut model = None;
        let mut screenshot = None;
        let mut animation = None;
        let mut animation_exact_name = false;
        let mut sample_midpoint = false;
        let mut evidence_root = None;
        let mut outline = PreviewOutlineMode::Source;
        let mut max_frames = DEFAULT_MAX_FRAMES;
        let mut timeout = Duration::from_secs_f64(DEFAULT_TIMEOUT_SECS);
        let mut character_kind = None;
        let mut true_root = None;
        let mut npc_scale = None;
        let mut main_texture = None;
        let mut sub_texture = None;
        let mut main_material = None;
        let mut sub_material = None;
        let mut main_sampler = None;
        let mut sub_sampler = None;
        let mut only_material = None;
        let mut camera_view = PreviewCameraView::Primary;
        let mut blank_camera_retry = BlankCameraRetry::AllAxes;
        let mut report = None;
        let mut runtime_smoke = None;
        let mut args = args.into_iter();

        while let Some(flag) = args.next() {
            if matches!(flag.as_str(), "-h" | "--help") {
                return Ok(ParseOutcome::Help);
            }
            let value = args
                .next()
                .ok_or_else(|| format!("{flag} requires a value"))?;
            match flag.as_str() {
                "--asset-root" => set_once(&mut asset_root, PathBuf::from(value), &flag)?,
                "--model" => set_once(&mut model, value, &flag)?,
                "--screenshot" => set_once(&mut screenshot, PathBuf::from(value), &flag)?,
                "--animation" => {
                    if value.is_empty() {
                        return Err("--animation may not be empty".to_owned());
                    }
                    set_once(&mut animation, value, &flag)?;
                }
                "--animation-name" => {
                    if value.is_empty() {
                        return Err("--animation-name may not be empty".to_owned());
                    }
                    set_once(&mut animation, value, &flag)?;
                    animation_exact_name = true;
                }
                "--character-kind" => {
                    set_once(&mut character_kind, CharacterKind::parse(&value)?, &flag)?;
                }
                "--true-root" => {
                    if value.is_empty() {
                        return Err("--true-root may not be empty".to_owned());
                    }
                    set_once(&mut true_root, value, &flag)?;
                }
                "--npc-scale" => {
                    let scale = value
                        .parse::<f32>()
                        .map_err(|_| format!("invalid --npc-scale value {value:?}"))?;
                    if !scale.is_finite() || scale <= 0.0 {
                        return Err("--npc-scale must be a finite positive number".to_owned());
                    }
                    set_once(&mut npc_scale, scale, &flag)?;
                }
                "--main-texture" => {
                    validate_relative_png_asset(&value, &flag)?;
                    set_once(&mut main_texture, value.replace('\\', "/"), &flag)?;
                }
                "--sub-texture" => {
                    validate_relative_png_asset(&value, &flag)?;
                    set_once(&mut sub_texture, value.replace('\\', "/"), &flag)?;
                }
                "--main-material" => {
                    if value.is_empty() {
                        return Err("--main-material may not be empty".to_owned());
                    }
                    set_once(&mut main_material, value, &flag)?;
                }
                "--sub-material" => {
                    if value.is_empty() {
                        return Err("--sub-material may not be empty".to_owned());
                    }
                    set_once(&mut sub_material, value, &flag)?;
                }
                "--main-sampler" => {
                    let sampler = serde_json::from_str(&value)
                        .map_err(|error| format!("invalid --main-sampler JSON: {error}"))?;
                    set_once(&mut main_sampler, sampler, &flag)?;
                }
                "--sub-sampler" => {
                    let sampler = serde_json::from_str(&value)
                        .map_err(|error| format!("invalid --sub-sampler JSON: {error}"))?;
                    set_once(&mut sub_sampler, sampler, &flag)?;
                }
                "--only-material" => {
                    if value.is_empty() {
                        return Err("--only-material may not be empty".to_owned());
                    }
                    set_once(&mut only_material, value, &flag)?;
                }
                "--camera-view" => camera_view = PreviewCameraView::parse(&value)?,
                "--blank-camera-retry" => {
                    blank_camera_retry = BlankCameraRetry::parse(&value)?;
                }
                "--report" => set_once(&mut report, PathBuf::from(value), &flag)?,
                "--runtime-smoke" => {
                    set_once(&mut runtime_smoke, PathBuf::from(value), &flag)?;
                }
                "--evidence-root" => {
                    set_once(&mut evidence_root, PathBuf::from(value), &flag)?;
                }
                "--outline" => {
                    outline = match value.as_str() {
                        "source" => PreviewOutlineMode::Source,
                        "hidden" => PreviewOutlineMode::Hidden,
                        _ => {
                            return Err(format!(
                                "invalid --outline value {value:?}; expected source or hidden"
                            ));
                        }
                    };
                }
                "--frames" => {
                    max_frames = value
                        .parse::<u64>()
                        .map_err(|_| format!("invalid --frames value {value:?}"))?;
                    if max_frames == 0 {
                        return Err("--frames must be greater than zero".to_owned());
                    }
                }
                "--sample-midpoint" => {
                    sample_midpoint = value
                        .parse::<bool>()
                        .map_err(|_| "--sample-midpoint expects true or false".to_owned())?;
                }
                "--timeout" => {
                    let seconds = value
                        .parse::<f64>()
                        .map_err(|_| format!("invalid --timeout value {value:?}"))?;
                    if !seconds.is_finite() || seconds <= 0.0 {
                        return Err("--timeout must be a finite positive number".to_owned());
                    }
                    timeout = Duration::from_secs_f64(seconds);
                }
                _ => return Err(format!("unknown option: {flag}")),
            }
        }

        let asset_root = asset_root.ok_or("--asset-root is required")?;
        if sample_midpoint && animation.is_none() {
            return Err("--sample-midpoint requires an animation".to_owned());
        }
        let model = model.ok_or("--model is required")?;
        validate_relative_glb(&model)?;
        if screenshot.is_none() && evidence_root.is_none() {
            return Err("--screenshot or --evidence-root is required".to_owned());
        }
        if screenshot.is_some() && evidence_root.is_some() {
            return Err("--screenshot and --evidence-root are mutually exclusive".to_owned());
        }
        if let Some(screenshot) = screenshot.as_deref() {
            validate_png_path(screenshot)?;
        }
        let character = match (character_kind, true_root, npc_scale) {
            (None, None, None) => None,
            (None, _, _) => {
                return Err(
                    "--true-root/--npc-scale require --character-kind npc|nano|player".to_owned(),
                );
            }
            (Some(_), None, _) => {
                return Err("--character-kind requires --true-root".to_owned());
            }
            (Some(CharacterKind::Npc), Some(true_root), Some(npc_scale)) => {
                Some(CharacterRuntimeConfig {
                    kind: CharacterKind::Npc,
                    true_root,
                    npc_scale: Some(npc_scale),
                })
            }
            (Some(CharacterKind::Npc), Some(_), None) => {
                return Err("--character-kind npc requires --npc-scale".to_owned());
            }
            (Some(kind), Some(true_root), None) => Some(CharacterRuntimeConfig {
                kind,
                true_root,
                npc_scale: None,
            }),
            (Some(kind), Some(_), Some(_)) => {
                return Err(format!(
                    "--npc-scale is only valid with --character-kind npc, not {}",
                    kind.as_str()
                ));
            }
        };
        if let Some(report) = report.as_deref() {
            validate_json_path(report)?;
        }
        if let Some(runtime_smoke) = runtime_smoke.as_deref() {
            validate_json_path(runtime_smoke)?;
            if evidence_root.is_none() {
                return Err("--runtime-smoke requires --evidence-root".to_owned());
            }
        }
        if main_sampler.is_some() && main_texture.is_none() {
            return Err("--main-sampler requires --main-texture".to_owned());
        }
        if sub_sampler.is_some() && sub_texture.is_none() {
            return Err("--sub-sampler requires --sub-texture".to_owned());
        }
        if main_material.is_some() && main_texture.is_none() {
            return Err("--main-material requires --main-texture".to_owned());
        }
        if sub_material.is_some() && sub_texture.is_none() {
            return Err("--sub-material requires --sub-texture".to_owned());
        }
        if main_material.is_some() && main_material == sub_material {
            return Err(
                "--main-material and --sub-material must name different materials".to_owned(),
            );
        }

        Ok(ParseOutcome::Run(Self {
            asset_root,
            model: model.replace('\\', "/"),
            screenshot: screenshot.unwrap_or_default(),
            animation,
            animation_exact_name,
            sample_midpoint,
            outline,
            max_frames,
            timeout,
            character,
            main_texture,
            sub_texture,
            main_material,
            sub_material,
            main_sampler,
            sub_sampler,
            only_material,
            camera_view,
            blank_camera_retry,
            report,
            runtime_smoke,
            material_animation_clips: Arc::from([]),
            evidence: evidence_root.map(|root| EvidenceConfig {
                root,
                json_path: PathBuf::new(),
                relative_json: String::new(),
                relative_png: String::new(),
                model_path: PathBuf::new(),
                glb_byte_length: 0,
                glb_sha256: String::new(),
                facts: LogicalModelGpuFacts {
                    true_name: String::new(),
                    standard_animation_names: Vec::new(),
                    mesh_parts: 0,
                    skinned_mesh_parts: 0,
                    skin_joint_references: 0,
                    inverse_bind_matrices: 0,
                    materials_applied: 0,
                    legacy_pass_companions: 0,
                    outline_pass_companions: 0,
                    assigned_texture_bindings: 0,
                    exact_mip_markers: 0,
                    exact_mip_chains: 0,
                    exact_mip_levels: 0,
                },
            }),
        }))
    }

    pub(super) fn validate_and_resolve(mut self) -> Result<Self, String> {
        self.asset_root = fs::canonicalize(&self.asset_root).map_err(|error| {
            format!(
                "cannot open native --asset-root {}: {error}",
                self.asset_root.display()
            )
        })?;
        if !self.asset_root.is_dir() {
            return Err(format!(
                "--asset-root is not a directory: {}",
                self.asset_root.display()
            ));
        }

        let model_path = self.asset_root.join(Path::new(&self.model));
        if !model_path.is_file() {
            return Err(format!(
                "native model does not exist below --asset-root: {}",
                model_path.display()
            ));
        }
        let glb = fs::read(&model_path)
            .map_err(|error| format!("cannot read native GLB {}: {error}", model_path.display()))?;
        self.material_animation_clips = parse_legacy_material_animation_clips(&glb)
            .map_err(|error| format!("cannot read native material animation metadata: {error}"))?
            .into();
        for (flag, texture) in [
            ("--main-texture", self.main_texture.as_deref()),
            ("--sub-texture", self.sub_texture.as_deref()),
        ] {
            let Some(texture) = texture else {
                continue;
            };
            let texture_path = self.asset_root.join(Path::new(texture));
            if !texture_path.is_file() {
                return Err(format!(
                    "{flag} does not exist below --asset-root: {}",
                    texture_path.display()
                ));
            }
        }
        if self.evidence.is_some() && (self.main_texture.is_some() || self.sub_texture.is_some()) {
            return Err(
                "--main-texture/--sub-texture are diagnostic runtime overrides and cannot be used with --evidence-root"
                    .to_owned(),
            );
        }

        if let Some(mut evidence) = self.evidence.take() {
            if self.outline != PreviewOutlineMode::Source {
                return Err("--evidence-root requires --outline source".to_owned());
            }
            if self.animation.is_some() && !self.animation_exact_name {
                return Err(
                    "--evidence-root requires --animation-name; index selectors are diagnostic only"
                        .to_owned(),
                );
            }
            let facts = gpu_model_facts_from_glb(&glb)
                .map_err(|error| format!("cannot derive strict GPU facts: {error}"))?;
            match (
                facts.standard_animation_names.is_empty(),
                self.animation.as_deref(),
            ) {
                (false, None) => {
                    return Err(format!(
                        "--animation-name is required for evidence: GLB has {} standard clips",
                        facts.standard_animation_names.len()
                    ));
                }
                (false, Some(name))
                    if !facts
                        .standard_animation_names
                        .iter()
                        .any(|candidate| candidate == name) =>
                {
                    return Err(format!(
                        "exact evidence animation {name:?} does not exist in the GLB"
                    ));
                }
                (true, Some(_)) => {
                    return Err(
                        "--animation-name must be omitted for a GLB without standard clips"
                            .to_owned(),
                    );
                }
                _ => {}
            }
            fs::create_dir_all(&evidence.root).map_err(|error| {
                format!(
                    "cannot create --evidence-root {}: {error}",
                    evidence.root.display()
                )
            })?;
            evidence.root = fs::canonicalize(&evidence.root).map_err(|error| {
                format!(
                    "cannot resolve --evidence-root {}: {error}",
                    evidence.root.display()
                )
            })?;
            if evidence.root.starts_with(&self.asset_root)
                || self.asset_root.starts_with(&evidence.root)
            {
                return Err(format!(
                    "--evidence-root and --asset-root must be disjoint: {}",
                    evidence.root.display()
                ));
            }
            let (relative_json, relative_png) =
                gpu_evidence_relative_paths(&self.model).map_err(|error| error.to_string())?;
            evidence.json_path = evidence.root.join(&relative_json);
            self.screenshot = evidence.root.join(&relative_png);
            evidence.relative_json = slash_path(&relative_json);
            evidence.relative_png = slash_path(&relative_png);
            evidence.model_path = model_path;
            evidence.glb_byte_length = u64::try_from(glb.len())
                .map_err(|_| "evidence GLB byte length exceeds u64".to_owned())?;
            evidence.glb_sha256 = sha256_hex(&glb);
            evidence.facts = facts;
            for output in [&evidence.json_path, &self.screenshot] {
                if output.exists() {
                    return Err(format!(
                        "immutable GPU evidence output already exists: {}",
                        output.display()
                    ));
                }
                if let Some(parent) = output.parent() {
                    fs::create_dir_all(parent).map_err(|error| {
                        format!(
                            "cannot create GPU evidence directory {}: {error}",
                            parent.display()
                        )
                    })?;
                }
            }
            self.evidence = Some(evidence);
            self.resolve_runtime_smoke_path()?;
            self.resolve_report_path()?;
            return Ok(self);
        }

        let screenshot_parent = self
            .screenshot
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(screenshot_parent).map_err(|error| {
            format!(
                "cannot create screenshot directory {}: {error}",
                screenshot_parent.display()
            )
        })?;
        let screenshot_parent = fs::canonicalize(screenshot_parent).map_err(|error| {
            format!(
                "cannot resolve screenshot directory {}: {error}",
                screenshot_parent.display()
            )
        })?;
        let file_name = self
            .screenshot
            .file_name()
            .ok_or("--screenshot must name a PNG file")?;
        self.screenshot = screenshot_parent.join(file_name);
        if self.screenshot.starts_with(&self.asset_root) {
            return Err(format!(
                "--screenshot must be outside --asset-root so the candidate stays immutable: {}",
                self.screenshot.display()
            ));
        }
        self.resolve_report_path()?;
        Ok(self)
    }

    pub(super) fn resolve_report_path(&mut self) -> Result<(), String> {
        let Some(report) = self.report.as_ref() else {
            return Ok(());
        };
        let parent = report
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "cannot create report directory {}: {error}",
                parent.display()
            )
        })?;
        let parent = fs::canonicalize(parent).map_err(|error| {
            format!(
                "cannot resolve report directory {}: {error}",
                parent.display()
            )
        })?;
        let file_name = report.file_name().ok_or("--report must name a JSON file")?;
        let resolved = parent.join(file_name);
        if resolved.starts_with(&self.asset_root) {
            return Err(format!(
                "--report must be outside --asset-root so the candidate stays immutable: {}",
                resolved.display()
            ));
        }
        self.report = Some(resolved);
        Ok(())
    }

    pub(super) fn resolve_runtime_smoke_path(&mut self) -> Result<(), String> {
        let Some(runtime_smoke) = self.runtime_smoke.as_ref() else {
            return Ok(());
        };
        let parent = runtime_smoke
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent).map_err(|error| {
            format!(
                "cannot create runtime-smoke directory {}: {error}",
                parent.display()
            )
        })?;
        let parent = fs::canonicalize(parent).map_err(|error| {
            format!(
                "cannot resolve runtime-smoke directory {}: {error}",
                parent.display()
            )
        })?;
        let file_name = runtime_smoke
            .file_name()
            .ok_or("--runtime-smoke must name a JSON file")?;
        let resolved = parent.join(file_name);
        let evidence_root = &self
            .evidence
            .as_ref()
            .ok_or("--runtime-smoke requires --evidence-root")?
            .root;
        if resolved.starts_with(&self.asset_root) || resolved.starts_with(evidence_root) {
            return Err(format!(
                "--runtime-smoke must be outside --asset-root and --evidence-root: {}",
                resolved.display()
            ));
        }
        if resolved.exists() {
            return Err(format!(
                "immutable runtime-smoke output already exists: {}",
                resolved.display()
            ));
        }
        self.runtime_smoke = Some(resolved);
        Ok(())
    }
}

#[derive(Resource)]
pub(super) struct CharacterSceneHandles {
    pub(super) gameplay_root: Entity,
    pub(super) visual_container: Entity,
    pub(super) scene: Entity,
}

#[derive(Component)]
pub(super) struct PreviewCamera;

/// The automated gate proves that a real surface reached the GPU; it does not
/// assert an authored gameplay camera. Start from the historic primary view,
/// then permit a deterministic diagnostic cycle for one-sided or nearly
/// planar equipment whose useful face is aligned to a cardinal model axis.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum PreviewCameraView {
    #[default]
    Primary,
    Reverse,
    PositiveX,
    NegativeX,
    PositiveY,
    NegativeY,
    PositiveZ,
    NegativeZ,
}

impl PreviewCameraView {
    pub(super) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "primary" => Ok(Self::Primary),
            "reverse" => Ok(Self::Reverse),
            "+x" | "positive-x" => Ok(Self::PositiveX),
            "-x" | "negative-x" => Ok(Self::NegativeX),
            "+y" | "positive-y" => Ok(Self::PositiveY),
            "-y" | "negative-y" => Ok(Self::NegativeY),
            "+z" | "positive-z" => Ok(Self::PositiveZ),
            "-z" | "negative-z" => Ok(Self::NegativeZ),
            _ => Err(format!(
                "invalid --camera-view value {value:?}; expected primary, reverse, +/-x, +/-y, or +/-z"
            )),
        }
    }

    pub(super) const fn opposite(self) -> Self {
        match self {
            Self::Primary => Self::Reverse,
            Self::Reverse => Self::Primary,
            Self::PositiveX => Self::NegativeX,
            Self::NegativeX => Self::PositiveX,
            Self::PositiveY => Self::NegativeY,
            Self::NegativeY => Self::PositiveY,
            Self::PositiveZ => Self::NegativeZ,
            Self::NegativeZ => Self::PositiveZ,
        }
    }

    pub(super) const fn next_diagnostic(self) -> Self {
        match self {
            Self::Primary => Self::Reverse,
            Self::Reverse => Self::PositiveX,
            Self::PositiveX => Self::NegativeX,
            Self::NegativeX => Self::PositiveY,
            Self::PositiveY => Self::NegativeY,
            Self::NegativeY => Self::PositiveZ,
            Self::PositiveZ => Self::NegativeZ,
            Self::NegativeZ => Self::Primary,
        }
    }

    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Reverse => "reverse",
            Self::PositiveX => "positive-x",
            Self::NegativeX => "negative-x",
            Self::PositiveY => "positive-y",
            Self::NegativeY => "negative-y",
            Self::PositiveZ => "positive-z",
            Self::NegativeZ => "negative-z",
        }
    }

    pub(super) fn direction(self) -> Vec3 {
        let primary = Vec3::new(1.35, 0.72, 2.15).normalize();
        match self {
            Self::Primary => primary,
            Self::Reverse => -primary,
            Self::PositiveX => Vec3::X,
            Self::NegativeX => Vec3::NEG_X,
            Self::PositiveY => Vec3::Y,
            Self::NegativeY => Vec3::NEG_Y,
            Self::PositiveZ => Vec3::Z,
            Self::NegativeZ => Vec3::NEG_Z,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) enum BlankCameraRetry {
    Same,
    Opposite,
    #[default]
    AllAxes,
}

impl BlankCameraRetry {
    pub(super) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "same" => Ok(Self::Same),
            "opposite" => Ok(Self::Opposite),
            "all-axes" => Ok(Self::AllAxes),
            _ => Err(format!(
                "invalid --blank-camera-retry value {value:?}; expected same, opposite, or all-axes"
            )),
        }
    }

    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::Same => "same",
            Self::Opposite => "opposite",
            Self::AllAxes => "all-axes",
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct SceneStats {
    pub(super) meshes: u64,
    pub(super) skinned_meshes: u64,
    pub(super) skin_joint_references: u64,
    pub(super) resolved_skin_joint_references: u64,
    pub(super) inverse_bind_matrices: u64,
    pub(super) animation_players: u64,
    pub(super) animation_graph_handles: u64,
    pub(super) materials_applied: u64,
    pub(super) legacy_pass_companions: u64,
    pub(super) outline_pass_companions: u64,
    pub(super) exact_mip_markers: u64,
    pub(super) assigned_texture_bindings: u64,
    pub(super) exact_mip_chains: u64,
    pub(super) exact_mip_levels: u64,
    pub(super) material_errors: u64,
    pub(super) shader_errors: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct Bounds3 {
    pub(super) min: Vec3,
    pub(super) max: Vec3,
}

impl Bounds3 {
    pub(super) fn from_point(point: Vec3) -> Option<Self> {
        point.is_finite().then_some(Self {
            min: point,
            max: point,
        })
    }

    pub(super) fn include_point(&mut self, point: Vec3) {
        if point.is_finite() {
            self.min = self.min.min(point);
            self.max = self.max.max(point);
        }
    }

    pub(super) fn center(self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    pub(super) fn half_extents(self) -> Vec3 {
        (self.max - self.min) * 0.5
    }

    pub(super) fn as_json(self) -> serde_json::Value {
        json!({
            "min": [self.min.x, self.min.y, self.min.z],
            "max": [self.max.x, self.max.y, self.max.z],
        })
    }
}
