use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum GenderFilter {
    All,
    Male,
    Female,
}

#[derive(Clone, Debug)]
pub(super) struct Cli {
    pub(super) asset_root: PathBuf,
    pub(super) output_root: PathBuf,
    pub(super) report: PathBuf,
    pub(super) category: Option<AvatarItemCategory>,
    pub(super) gender: GenderFilter,
    pub(super) start: usize,
    pub(super) limit: Option<usize>,
    pub(super) screenshots: ScreenshotMode,
}

impl Cli {
    pub(super) fn parse() -> Result<Self, String> {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut cli = Self {
            asset_root: repo.join("assets/game"),
            output_root: repo.join("target/avatar-equipment-gpu-audit/frames"),
            report: repo.join("target/avatar-equipment-gpu-audit/report.json"),
            category: None,
            gender: GenderFilter::All,
            start: 0,
            limit: None,
            screenshots: ScreenshotMode::All,
        };
        let mut args = std::env::args().skip(1);
        while let Some(flag) = args.next() {
            let mut value = || {
                args.next()
                    .ok_or_else(|| format!("{flag} requires a value"))
            };
            match flag.as_str() {
                "--asset-root" => cli.asset_root = PathBuf::from(value()?),
                "--output-root" => cli.output_root = PathBuf::from(value()?),
                "--report" => cli.report = PathBuf::from(value()?),
                "--category" => {
                    let raw = value()?;
                    cli.category = if raw == "all" {
                        None
                    } else {
                        Some(parse_category(&raw)?)
                    };
                }
                "--gender" => {
                    cli.gender = match value()?.as_str() {
                        "all" => GenderFilter::All,
                        "male" => GenderFilter::Male,
                        "female" => GenderFilter::Female,
                        other => {
                            return Err(format!(
                                "invalid --gender {other:?}; expected all, male, or female"
                            ));
                        }
                    }
                }
                "--start" => {
                    cli.start = value()?
                        .parse()
                        .map_err(|error| format!("invalid --start: {error}"))?;
                }
                "--limit" => {
                    cli.limit = Some(
                        value()?
                            .parse()
                            .map_err(|error| format!("invalid --limit: {error}"))?,
                    );
                }
                "--screenshots" => {
                    cli.screenshots = match value()?.as_str() {
                        "all" => ScreenshotMode::All,
                        "failures" => ScreenshotMode::Failures,
                        other => {
                            return Err(format!(
                                "invalid --screenshots {other:?}; expected all or failures"
                            ));
                        }
                    }
                }
                "-h" | "--help" => {
                    println!(
                        "avatar_equipment_gpu_audit [--category all|shirt|pants|shoes|hat|glasses|back] \
                         [--gender all|male|female] [--start N] [--limit N] \
                         [--screenshots all|failures] [--output-root DIR] [--report JSON]"
                    );
                    std::process::exit(0);
                }
                other => return Err(format!("unknown option {other:?}")),
            }
        }
        cli.asset_root = fs::canonicalize(&cli.asset_root).map_err(|error| {
            format!(
                "cannot resolve asset root {}: {error}",
                cli.asset_root.display()
            )
        })?;
        fs::create_dir_all(&cli.output_root).map_err(|error| {
            format!(
                "cannot create output root {}: {error}",
                cli.output_root.display()
            )
        })?;
        if let Some(parent) = cli.report.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "cannot create report directory {}: {error}",
                    parent.display()
                )
            })?;
        }
        Ok(cli)
    }
}

#[derive(Clone, Debug)]
pub(super) struct Job {
    pub(super) ordinal: usize,
    pub(super) category: AvatarItemCategory,
    pub(super) item_number: u32,
    pub(super) name: String,
    pub(super) gender: AuditGender,
    pub(super) declared_model_status: NativeLookupStatus,
    pub(super) declared_primary_status: Option<NativeLookupStatus>,
    pub(super) declared_secondary_status: Option<NativeLookupStatus>,
}

impl Job {
    pub(super) fn identity(&self) -> String {
        format!(
            "equipment-audit-{}-{}-{}",
            category_label(self.category),
            self.item_number,
            self.gender.label()
        )
    }

    pub(super) fn screenshot_path(&self, root: &Path) -> PathBuf {
        root.join(category_label(self.category))
            .join(self.gender.label())
            .join(format!("{:04}.png", self.item_number))
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SurfaceEvidence {
    pub(super) kind: String,
    pub(super) exact_route: String,
    pub(super) material_true_name: String,
    pub(super) role: String,
    pub(super) bound_texture_path: Option<String>,
    pub(super) base_texture_assigned: bool,
    pub(super) source_main_texture: Option<String>,
    pub(super) hidden: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ItemResult {
    pub(super) ordinal: usize,
    pub(super) category: String,
    pub(super) item_number: u32,
    pub(super) name: String,
    pub(super) gender: AuditGender,
    pub(super) identity: String,
    pub(super) declared_model_status: String,
    pub(super) declared_primary_status: Option<String>,
    pub(super) declared_secondary_status: Option<String>,
    pub(super) target_route: Option<String>,
    pub(super) expected_primary_texture: Option<String>,
    pub(super) expected_secondary_texture: Option<String>,
    pub(super) rendered: bool,
    pub(super) passed: bool,
    pub(super) failure_reasons: Vec<String>,
    pub(super) surfaces: Vec<SurfaceEvidence>,
    pub(super) screenshot: Option<String>,
    pub(super) capture_visible_pixels: Option<u64>,
    pub(super) capture_total_pixels: Option<u64>,
    pub(super) frames: u32,
    pub(super) elapsed_seconds: f64,
}

#[derive(Clone, Debug)]
pub(super) enum Phase {
    Start,
    Loading {
        started: Instant,
        frames: u32,
        identity: String,
        look: NativePlayerLook,
        target_kind: NativePlayerPartKind,
    },
    ReadyWarmup {
        started: Instant,
        frames: u32,
        ready_frames: u32,
        identity: String,
        look: NativePlayerLook,
        target_kind: NativePlayerPartKind,
    },
    Capturing {
        result: ItemResult,
        path: PathBuf,
    },
    Finished,
}

#[derive(Clone, Debug)]
pub(super) struct CaptureOutcome {
    pub(super) path: PathBuf,
    pub(super) visible_pixels: u64,
    pub(super) total_pixels: u64,
    pub(super) save_error: Option<String>,
}

#[derive(Component)]
pub(super) struct PendingItemCapture {
    pub(super) path: PathBuf,
}
