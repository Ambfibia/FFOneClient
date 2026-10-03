use super::*;

#[derive(Debug)]
pub(super) struct EditorArguments {
    pub(super) asset_root: PathBuf,
    pub(super) language: String,
    pub(super) npc: Option<i64>,
    pub(super) nano: Option<i64>,
    pub(super) capture: Option<PathBuf>,
    pub(super) equipment: Option<String>,
    pub(super) female: bool,
    pub(super) search: String,
    pub(super) compact: bool,
    pub(super) xdt_table: Option<String>,
    pub(super) xdt_row: Option<usize>,
}

impl EditorArguments {
    pub(super) fn parse() -> Self {
        let mut asset_root = PathBuf::from("assets/game");
        let mut language = "ru".to_owned();
        let mut npc = None;
        let mut nano = None;
        let mut capture = None;
        let mut equipment = None;
        let mut female = false;
        let mut search = String::new();
        let mut compact = false;
        let mut xdt_table = None;
        let mut xdt_row = None;
        let mut args = env::args().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--nano" => nano = Some(args.next().expect("--nano ID").parse().expect("Nano ID")),
                "--equipment" => equipment = Some(args.next().expect("--equipment category/id")),
                "--female" => female = true,
                "--search" => search = args.next().expect("--search text"),
                "--compact" => compact = true,
                "--xdt-table" => xdt_table = Some(args.next().expect("--xdt-table table path")),
                "--xdt-row" => {
                    xdt_row = Some(
                        args.next()
                            .expect("--xdt-row zero-based index")
                            .parse()
                            .expect("row index"),
                    )
                }
                "--npc" => {
                    npc = Some(
                        args.next()
                            .expect("--npc value")
                            .parse()
                            .expect("NPC integer"),
                    );
                }
                "--capture" => {
                    capture = Some(PathBuf::from(args.next().expect("--capture path")));
                }
                "--asset-root" => {
                    if let Some(value) = args.next() {
                        asset_root = PathBuf::from(value);
                    }
                }
                "--language" => {
                    if let Some(value) = args.next() {
                        language = value;
                    }
                }
                _ => {}
            }
        }
        Self {
            asset_root,
            language,
            npc,
            nano,
            capture,
            equipment,
            female,
            search,
            compact,
            xdt_table,
            xdt_row,
        }
    }
}

#[derive(Debug, Default, Resource)]
pub(super) struct PreparedAnimations(pub(super) Option<PreparedAnimationSet>);

#[derive(Component)]
pub(super) struct PreviewRoot;

#[derive(Component)]
pub(super) struct PreviewCamera;

#[derive(Debug, Resource)]
pub(super) struct OrbitCamera {
    pub(super) yaw: f32,
    pub(super) pitch: f32,
    pub(super) distance: f32,
    pub(super) target_y: f32,
    pub(super) target_xz: Vec2,
    pub(super) fit_requested: bool,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.12,
            distance: 4.2,
            target_y: 1.0,
            target_xz: Vec2::ZERO,
            fit_requested: true,
        }
    }
}

impl OrbitCamera {
    pub(super) fn reset_for(&mut self, entry: &EditorCatalogEntry) {
        let height = match entry.kind {
            CatalogKind::Npc => entry
                .height_server_units
                .map(|height| height as f32 * 0.01)
                .unwrap_or(1.8),
            CatalogKind::Nano => 1.05,
            CatalogKind::Equipment => 1.8,
        }
        .clamp(0.55, 30.0);
        self.yaw = 0.0;
        self.pitch = 0.1;
        self.target_y = height * 0.46;
        self.target_xz = Vec2::ZERO;
        self.fit_requested = true;
        self.distance = (height * 2.7 + 0.8).clamp(2.5, 90.0);
    }
}

#[derive(Resource)]
pub(super) struct EditorCapture {
    pub(super) output: Option<PathBuf>,
    pub(super) frames: u32,
    pub(super) ready_frames: u32,
    pub(super) issued: bool,
}

#[derive(Resource, Clone)]
pub(super) struct EditorFonts {
    pub(super) body: Handle<Font>,
    pub(super) display: Handle<Font>,
    pub(super) button: Handle<Image>,
    pub(super) button_hover: Handle<Image>,
    pub(super) panel: Handle<Image>,
    pub(super) textfield: Handle<Image>,
}

#[derive(Component, Debug, Clone, Copy)]
pub(super) enum DynamicTextRole {
    Search,
    CatalogCount,
    CatalogSlot(usize),
    CurrentTitle,
    CurrentSubtitle,
    RuntimeStatus,
    InspectorIdentity,
    InspectorSource,
    InspectorGeometry,
    InspectorTextures,
    AnimationPage,
    AnimationSlot(usize),
    Playback,
    Loop,
    Speed,
    Turntable,
    Timeline,
    Language,
}

#[derive(Component)]
pub(super) struct TimelineFill;

#[derive(Component)]
pub(super) struct EditorButtonSkin;

#[derive(Component)]
pub(super) struct EditorButtonLabel;
