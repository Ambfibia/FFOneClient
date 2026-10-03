use super::*;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum SkillBuffTextAnchor {
    MiddleLeft,
}

/// The only text-bearing GUIStyle reached by clean `CnGuiSkillBuffIcon`.
///
/// Icons themselves are image-only `GUI.Label` calls. Only the cash-duration
/// copy reaches inherited `GUI.skin.label`, so unrelated HUD skin styles must
/// not enter this passive tree.
#[derive(Clone, Copy, Component, Debug, Eq, Hash, PartialEq)]
pub enum SkillBuffTextStyle {
    HudLabel,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkillBuffTextStyleSpec {
    pub source_style: &'static str,
    pub source_skin_path_id: i64,
    pub source_font_path_id: i64,
    pub font_path: &'static str,
    pub font_size: f32,
    pub line_height: f32,
    /// Unity `RectOffset` order: left, right, top, bottom.
    pub padding: [f32; 4],
    pub anchor: SkillBuffTextAnchor,
    pub justify: Justify,
    pub linebreak: LineBreak,
    pub normal_color: [f32; 4],
    pub word_wrap: bool,
    pub text_clipping: bool,
    pub y_offset: f32,
}

impl SkillBuffTextStyle {
    #[must_use]
    pub const fn spec(self) -> SkillBuffTextStyleSpec {
        match self {
            Self::HudLabel => SkillBuffTextStyleSpec {
                source_style: "label",
                source_skin_path_id: SKILL_BUFF_SKIN_PATH_ID,
                source_font_path_id: SKILL_BUFF_FONT_PATH_ID,
                font_path: SKILL_BUFF_FONT_PATH,
                font_size: SKILL_BUFF_FONT_SIZE,
                line_height: SKILL_BUFF_FONT_LINE_HEIGHT,
                padding: [0.0; 4],
                anchor: SkillBuffTextAnchor::MiddleLeft,
                justify: Justify::Left,
                linebreak: LineBreak::WordBoundary,
                normal_color: SKILL_BUFF_LABEL_COLOR,
                word_wrap: true,
                text_clipping: true,
                y_offset: SKILL_BUFF_FONT_Y_OFFSET,
            },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SkillBuffUiDefinition {
    pub buff_id: i32,
    pub icon_number: u16,
    pub icon_path: String,
    pub cash_icon_number: u16,
    pub cash_icon_path: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SkillBuffTargetUi {
    pub character_type: i32,
    pub character_id: i32,
    pub condition_bit_flag: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SkillBuffIconView {
    pub buff_id: i32,
    pub icon_number: u16,
    pub icon_path: String,
    pub rect: SkillBuffUiRect,
    pub cash_time: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SkillBuffUiView {
    pub visible: bool,
    pub local: Vec<SkillBuffIconView>,
    pub cash: Vec<SkillBuffIconView>,
    pub target: Vec<SkillBuffIconView>,
}

#[derive(Clone, Copy)]
pub(super) enum IconProjection {
    Local,
    Cash,
    Target,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum SkillBuffRow {
    Local,
    Cash,
    Target,
}

impl SkillBuffUiView {
    pub(super) fn row(&self, row: SkillBuffRow) -> &[SkillBuffIconView] {
        match row {
            SkillBuffRow::Local => &self.local,
            SkillBuffRow::Cash => &self.cash,
            SkillBuffRow::Target => &self.target,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct SkillBuffUiRoot;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) enum SkillBuffUiElement {
    Slot(SkillBuffRow, usize),
    Icon(SkillBuffRow, usize),
    CashTime(usize),
    CashText(usize),
}

#[derive(Clone, Debug, Resource)]
pub(super) struct SkillBuffUiAssets {
    pub(super) background: Handle<Image>,
    pub(super) font: Handle<Font>,
    pub(super) icons: BTreeMap<String, Handle<Image>>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum SkillBuffUiSet {
    AdvanceTimers,
    ApplyNanoGumballs,
    Bind,
}

pub struct SkillBuffUiPlugin;

impl Plugin for SkillBuffUiPlugin {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<SkillBuffUiCatalog>()
            .init_resource::<SkillBuffUiModel>()
            .add_systems(
                crate::ui_startup::NativeGameplayUiStartup,
                spawn_skill_buff_ui,
            )
            .add_systems(
                Update,
                ((
                    advance_skill_buff_cash_timer.in_set(SkillBuffUiSet::AdvanceTimers),
                    sync_skill_buff_nano_gumballs
                        .in_set(SkillBuffUiSet::ApplyNanoGumballs)
                        .before(GameplayUiSet::BindNanoWheel),
                    bind_skill_buff_ui
                        .in_set(SkillBuffUiSet::Bind)
                        .before(LocalizationSet::Apply),
                )
                    .chain())
                .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}
