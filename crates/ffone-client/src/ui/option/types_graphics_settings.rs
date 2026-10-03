use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OptionFontContract {
    pub runtime_path: &'static str,
    pub bytes: u64,
    pub sha256: &'static str,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum OptionTab {
    #[default]
    Graphics,
    GameUi,
    Social,
    Controls,
}

impl OptionTab {
    pub const ALL: [Self; 4] = [Self::Graphics, Self::GameUi, Self::Social, Self::Controls];

    #[must_use]
    pub const fn normal_rect(self) -> OptionUiRect {
        match self {
            Self::Graphics => OptionUiRect::new(8.0, 19.0, 275.0, 44.0),
            Self::GameUi => OptionUiRect::new(330.0, 21.0, 188.0, 32.0),
            Self::Social => OptionUiRect::new(560.0, 21.0, 189.0, 33.0),
            Self::Controls => OptionUiRect::new(798.0, 19.0, 196.0, 45.0),
        }
    }

    #[must_use]
    pub const fn selected_rect(self) -> OptionUiRect {
        match self {
            Self::Graphics => OptionUiRect::new(8.0, 17.0, 271.0, 49.0),
            Self::GameUi => OptionUiRect::new(329.0, 19.0, 192.0, 44.0),
            Self::Social => OptionUiRect::new(557.0, 20.0, 197.0, 44.0),
            Self::Controls => OptionUiRect::new(799.0, 3.0, 194.0, 61.0),
        }
    }

    /// The clean code tests `tab == 4`; no typed, reachable tab can satisfy it.
    #[must_use]
    pub const fn dead_reset_visible(self) -> bool {
        let _ = self;
        false
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum GraphicsDetail {
    BestQuality,
    GoodQuality,
    #[default]
    Balanced,
    GoodPerformance,
    BestPerformance,
    Custom,
}

impl GraphicsDetail {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::BestQuality => "BestQuality",
            Self::GoodQuality => "GoodQuality",
            Self::Balanced => "Balanced",
            Self::GoodPerformance => "GoodPerformance",
            Self::BestPerformance => "BestPerformance",
            Self::Custom => "Custom",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub enum ShadowQuality {
    #[default]
    PlayerOnly,
    AllCharacters,
}

impl ShadowQuality {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::PlayerOnly => "PlayerOnly",
            Self::AllCharacters => "AllCharacters",
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct GraphicsSettings {
    pub width: u32,
    pub height: u32,
    pub windowed: bool,
    pub detail: GraphicsDetail,
    pub visibility: f32,
    pub particle_level: u8,
    pub toon_shading: bool,
    pub glow: bool,
    pub anisotropic_filtering: bool,
    pub fade: bool,
    pub soft_vegetation: bool,
    pub shadow: ShadowQuality,
    pub texture: TextureQuality,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            width: 1_024,
            height: 768,
            windowed: true,
            detail: GraphicsDetail::Balanced,
            visibility: 0.5,
            particle_level: 2,
            toon_shading: true,
            glow: true,
            anisotropic_filtering: true,
            fade: true,
            // The constructor is authoritative here. Applying the Balanced
            // preset later changes this one field to false.
            soft_vegetation: true,
            shadow: ShadowQuality::PlayerOnly,
            texture: TextureQuality::High,
        }
    }
}

impl GraphicsSettings {
    pub fn apply_preset(&mut self, detail: GraphicsDetail) {
        self.detail = detail;
        let (visibility, particles, toon, glow, anisotropic, fade, soft_vegetation, texture) =
            match detail {
                GraphicsDetail::BestQuality => {
                    (1.0, 3, true, true, true, true, true, TextureQuality::High)
                }
                GraphicsDetail::GoodQuality => {
                    (0.75, 3, true, true, true, true, true, TextureQuality::High)
                }
                GraphicsDetail::Balanced => {
                    (0.5, 2, true, true, true, true, false, TextureQuality::High)
                }
                GraphicsDetail::GoodPerformance => (
                    0.25,
                    1,
                    true,
                    true,
                    false,
                    false,
                    false,
                    TextureQuality::Medium,
                ),
                GraphicsDetail::BestPerformance => (
                    0.0,
                    0,
                    false,
                    false,
                    false,
                    false,
                    false,
                    TextureQuality::Low,
                ),
                GraphicsDetail::Custom => return,
            };
        self.visibility = visibility;
        self.particle_level = particles;
        self.toon_shading = toon;
        self.glow = glow;
        self.anisotropic_filtering = anisotropic;
        self.fade = fade;
        self.soft_vegetation = soft_vegetation;
        self.shadow = ShadowQuality::PlayerOnly;
        self.texture = texture;
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DisplaySettings {
    pub my_name: bool,
    pub other_name: bool,
    pub group_name: bool,
    pub npc_name: bool,
    pub monster_name: bool,
    pub floating_numbers: bool,
    pub balloon: bool,
    /// Persisted by clean `cnDisplayOption` even though this build's
    /// `OnGameUI` page does not expose a row for it.
    pub current_objective: bool,
    pub guide_email: bool,
    pub waypoint: bool,
    pub scale_ui: bool,
    /// Clean `bNewChat`; the legacy row is labelled `OLD CHAT` and displays
    /// the inverse value.
    pub new_chat: bool,
    pub game_hint: bool,
    pub npc_messages_in_chat: bool,
    pub combat_in_chat: bool,
    pub animated_nanos: bool,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            my_name: false,
            other_name: false,
            group_name: false,
            npc_name: false,
            monster_name: false,
            floating_numbers: false,
            balloon: true,
            current_objective: true,
            guide_email: true,
            waypoint: true,
            scale_ui: true,
            new_chat: true,
            game_hint: true,
            npc_messages_in_chat: true,
            combat_in_chat: false,
            animated_nanos: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TextColorSettings {
    pub general: u8,
    pub buddy: u8,
    pub group: u8,
}

impl Default for TextColorSettings {
    fn default() -> Self {
        Self {
            general: 1,
            buddy: 2,
            group: 3,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum OptionGraphicsToggle {
    ToonOutline,
    Glow,
    AnisotropicFiltering,
    SoftVegetation,
    ObjectFading,
}

impl OptionGraphicsToggle {
    pub const ALL: [Self; 5] = [
        Self::ToonOutline,
        Self::Glow,
        Self::AnisotropicFiltering,
        Self::SoftVegetation,
        Self::ObjectFading,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::ToonOutline => "TOON OUTLINE",
            Self::Glow => "GLOW",
            Self::AnisotropicFiltering => "ANISOTROPIC FILTERING",
            Self::SoftVegetation => "SOFT VEGETATION",
            Self::ObjectFading => "OBJECT FADING",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum OptionDisplayElement {
    MyName,
    OtherPlayerNames,
    GroupMemberNames,
    NpcNames,
    MonsterNames,
    DamageNumbers,
    BalloonChat,
    ScaleUiElements,
    OldChat,
    ComputressHints,
    NpcMessagesInChat,
    CombatInChat,
    AnimatedNanocom,
}

impl OptionDisplayElement {
    pub const ALL: [Self; 13] = [
        Self::MyName,
        Self::OtherPlayerNames,
        Self::GroupMemberNames,
        Self::NpcNames,
        Self::MonsterNames,
        Self::DamageNumbers,
        Self::BalloonChat,
        Self::ScaleUiElements,
        Self::OldChat,
        Self::ComputressHints,
        Self::NpcMessagesInChat,
        Self::CombatInChat,
        Self::AnimatedNanocom,
    ];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::MyName => "MY NAME",
            Self::OtherPlayerNames => "OTHER PLAYER NAMES",
            Self::GroupMemberNames => "GROUP MEMBER NAMES",
            Self::NpcNames => "NPC NAMES",
            Self::MonsterNames => "MONSTER NAMES",
            Self::DamageNumbers => "DAMAGE NUMBERS",
            Self::BalloonChat => "BALLOON CHAT",
            Self::ScaleUiElements => "SCALE UI ELEMENTS",
            Self::OldChat => "OLD CHAT",
            Self::ComputressHints => "COMPUTRESS HINTS",
            Self::NpcMessagesInChat => "NPC MESSAGES IN CHAT WINDOW",
            Self::CombatInChat => "COMBAT IN CHAT WINDOW",
            Self::AnimatedNanocom => "ANIMATED NANOCOM",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum OptionTextColorChannel {
    General,
    Group,
    Buddy,
}

impl OptionTextColorChannel {
    pub const ALL: [Self; 3] = [Self::General, Self::Group, Self::Buddy];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::General => "GENERAL CHAT",
            Self::Group => "GROUP CHAT",
            Self::Buddy => "BUDDY CHAT",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum OptionControlGroup {
    Movement,
    Interface,
    Camera,
    Combat,
}

impl OptionControlGroup {
    pub const ALL: [Self; 4] = [Self::Movement, Self::Interface, Self::Camera, Self::Combat];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Movement => "MOVEMENT",
            Self::Interface => "INTERFACE",
            Self::Camera => "CAMERA",
            Self::Combat => "COMBAT",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum LegacyPadProfile {
    #[default]
    Xbox360,
    RumblePad2,
    PlayStation2,
}

impl LegacyPadProfile {
    pub const ALL: [Self; 3] = [Self::Xbox360, Self::RumblePad2, Self::PlayStation2];

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Xbox360 => "XBox 360",
            Self::RumblePad2 => "RumblePad 2",
            Self::PlayStation2 => "PS2",
        }
    }
}

impl OptionControlGroup {
    #[must_use]
    pub const fn actions(self) -> &'static [LegacyOptionAction] {
        match self {
            Self::Movement => &OPTION_MOVEMENT_ACTIONS,
            Self::Interface => &OPTION_INTERFACE_ACTIONS,
            Self::Camera => &OPTION_CAMERA_ACTIONS,
            Self::Combat => &OPTION_COMBAT_ACTIONS,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum LegacyPhysicalKey {
    W,
    S,
    A,
    D,
    BackQuote,
    Space,
    Z,
    X,
    Digit1,
    Digit2,
    Digit3,
    Tab,
    R,
    C,
    I,
    N,
    J,
    P,
    H,
    Home,
    Enter,
    M,
    Quote,
    Q,
    E,
    ControlLeft,
    F,
    V,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Numpad1,
    Numpad2,
    Numpad3,
    NumpadEnter,
    ControlRight,
    Mouse0,
    Mouse1,
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub struct OptionSettings {
    pub graphics: GraphicsSettings,
    pub sound: SoundSettings,
    pub display: DisplaySettings,
    pub text_colors: TextColorSettings,
    pub social: SocialRequestSettings,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OptionBuddySlot {
    pub pc_uid: i64,
    pub blocked: bool,
    pub name_check_flag: i8,
    pub first_name: String,
    pub last_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BlockedPlayerRow {
    pub slot: usize,
    pub pc_uid: i64,
    pub display_name: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OptionSystemPopup {
    OldChatRestartRequired,
    DuplicateInputBinding,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum OptionDropdownKind {
    Resolution,
    Detail,
    Shadow,
    Texture,
    Pad,
    Translation,
    Voice,
}

impl OptionSystemPopup {
    #[must_use]
    pub const fn message(self) -> &'static str {
        match self {
            Self::OldChatRestartRequired => {
                "NOTE!\nThis option is experimental.\nYou must restart your game for this option to take effect."
            }
            Self::DuplicateInputBinding => {
                "THAT KEY OR BUTTON IS ALREADY ASSIGNED.\nCHOOSE A DIFFERENT SETTING."
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OptionCloseTrigger {
    CloseButton,
    Shortcut,
}

#[derive(Debug, Default, Resource)]
pub struct OptionUiOutbox {
    pub(super) events: VecDeque<OptionUiEvent>,
}

impl OptionUiOutbox {
    #[must_use]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn pop_front(&mut self) -> Option<OptionUiEvent> {
        self.events.pop_front()
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub(super) fn action(&mut self, action: OptionUiAction) {
        self.events.push_back(OptionUiEvent::Action(action));
    }

    pub(super) fn audio(&mut self, cue: OptionUiAudioCue) {
        self.events.push_back(OptionUiEvent::Audio(cue));
    }
}

#[derive(Clone, Resource)]
pub(super) struct OptionUiAssets {
    pub(super) images: Vec<(OptionTextureRole, Handle<Image>)>,
    pub(super) jeffe: Handle<Font>,
    pub(super) comic: Handle<Font>,
    pub(super) chalet: Handle<Font>,
}

impl OptionUiAssets {
    pub(super) fn load(asset_server: &AssetServer) -> Self {
        Self {
            images: OPTION_TEXTURE_CONTRACTS
                .iter()
                .map(|contract| (contract.role, asset_server.load(contract.runtime_path)))
                .collect(),
            jeffe: asset_server.load(OPTION_JEFFE_FONT_PATH),
            comic: asset_server.load(OPTION_COMIC_FONT_PATH),
            chalet: asset_server.load(OPTION_CHALET_FONT_PATH),
        }
    }

    pub(super) fn image(&self, role: OptionTextureRole) -> Handle<Image> {
        self.images
            .iter()
            .find_map(|(candidate, handle)| (*candidate == role).then(|| handle.clone()))
            .unwrap_or_else(|| panic!("missing Option texture handle for {role:?}"))
    }

    pub(super) fn image_load_states<'a>(
        &'a self,
        asset_server: &'a AssetServer,
    ) -> impl Iterator<Item = LoadState> + 'a {
        self.images
            .iter()
            .map(|(_, handle)| asset_server.load_state(handle.id()))
    }

    pub(super) fn font_load_states<'a>(
        &'a self,
        asset_server: &'a AssetServer,
    ) -> impl Iterator<Item = LoadState> + 'a {
        [&self.jeffe, &self.comic, &self.chalet]
            .into_iter()
            .map(|handle| asset_server.load_state(handle.id()))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionUiRoot;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionUiWindow;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionUiBackdrop;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionPage(pub OptionTab);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct OptionSkyHeader;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct OptionTabLabel(pub OptionTab);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionBlockedRow {
    pub projection_index: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct OptionBlockedRowLabel {
    pub(super) projection_index: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub enum OptionGraphicsSliderThumb {
    Visibility,
    Particles,
    Sound(OptionSoundChannel),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) enum OptionRadioIndicator {
    Graphics {
        toggle: OptionGraphicsToggle,
        value: bool,
    },
    Sound {
        channel: OptionSoundChannel,
        value: bool,
    },
    Display {
        element: OptionDisplayElement,
        value: bool,
    },
    InvertY(bool),
    PadInvertY(bool),
    Social {
        request: SocialRequestKind,
        value: bool,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct OptionRadioLabel(pub(super) OptionRadioIndicator);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionDropdownPanel(pub OptionDropdownKind);

#[derive(Clone, Debug, Eq, PartialEq, Component)]
pub enum OptionDropdownChoice {
    Resolution {
        width: u32,
        height: u32,
        windowed: bool,
    },
    Detail(GraphicsDetail),
    Shadow(ShadowQuality),
    Texture(TextureQuality),
    Pad(LegacyPadProfile),
    Translation(String),
    Voice(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct OptionDropdownValueLabel(pub OptionDropdownKind);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct OptionDropdownChoiceLabel;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct OptionDropdownChoiceBackground;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionSensitivityThumb;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct OptionMappingLabel {
    pub(super) action: LegacyOptionAction,
    pub(super) slot: OptionInputMappingSlot,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct OptionKeyCapturePrompt;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub struct OptionSystemPopupRoot;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub(super) struct OptionSystemPopupLabel;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum OptionUiSet {
    AssetGate,
    Interaction,
    Bind,
    Visuals,
}

/// Passive UI-only plugin. Installing it does not open OptionMode and does
/// not own a camera, persistence, audio playback, input mapping, renderer or
/// protocol adapter.
pub struct OptionUiPlugin;
