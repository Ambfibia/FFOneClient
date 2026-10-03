use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CharacterCreationFontRole {
    Jeffe,
    Chalet,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CharacterCreationTextAnchor {
    MiddleLeft,
    MiddleCenter,
}

/// Text-bearing `GUIStyle`s reached by clean `CnGuiNameCreation` and
/// `CnGuiCharCreation`. Runtime-created clones remain separate variants so
/// their changed font/color is source-auditable.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Component)]
pub enum CharacterCreationTextStyle {
    Label,
    Transparent,
    SectionLabel,
    Toggle,
    BodyText,
    Button,
    ButtonTabFont,
    ExitButton,
    TabButton,
    TabText,
    OrText,
    NameDisplay,
    Transparent4,
    Transparent5,
    CustomQuestion,
    TextField,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CharacterCreationTextStyleSpec {
    pub source_style: &'static str,
    pub source_font_path_id: i64,
    pub font_role: CharacterCreationFontRole,
    pub font_size: f32,
    pub line_height: f32,
    /// Left, right, top, bottom in clean IMGUI pixels.
    pub padding: [f32; 4],
    pub anchor: CharacterCreationTextAnchor,
    pub normal_color: [f32; 4],
    pub word_wrap: bool,
    pub clip: bool,
    pub content_offset: [f32; 2],
    pub y_offset: f32,
}

impl CharacterCreationTextStyle {
    #[must_use]
    pub const fn spec(self) -> CharacterCreationTextStyleSpec {
        use CharacterCreationFontRole::{Chalet, Jeffe};
        use CharacterCreationTextAnchor::{MiddleCenter, MiddleLeft};

        let (source_style, source_font_path_id, font_role, font_size, line_height) = match self {
            Self::Label => (
                "label",
                CHARACTER_CREATION_JEFFE_16_PATH_ID,
                Jeffe,
                CHARACTER_CREATION_JEFFE_16_FONT_SIZE,
                CHARACTER_CREATION_JEFFE_16_LINE_HEIGHT,
            ),
            Self::Transparent
            | Self::Toggle
            | Self::Button
            | Self::ExitButton
            | Self::TextField => (
                match self {
                    Self::Transparent => "Transparent",
                    Self::Toggle => "toggle",
                    Self::Button => "button",
                    Self::ExitButton => "ExitButton",
                    Self::TextField => "textField",
                    _ => unreachable!(),
                },
                CHARACTER_CREATION_JEFFE_16_PATH_ID,
                Jeffe,
                CHARACTER_CREATION_JEFFE_16_FONT_SIZE,
                CHARACTER_CREATION_JEFFE_16_LINE_HEIGHT,
            ),
            Self::SectionLabel
            | Self::BodyText
            | Self::ButtonTabFont
            | Self::TabButton
            | Self::TabText
            | Self::OrText
            | Self::NameDisplay
            | Self::CustomQuestion => (
                match self {
                    Self::SectionLabel => "Transparent + TabButton.font",
                    Self::BodyText => "BodyText",
                    Self::ButtonTabFont => "button + TabButton.font",
                    Self::TabButton => "TabButton",
                    Self::TabText => "TabText",
                    Self::OrText => "OrText",
                    Self::NameDisplay => "NameDisplay",
                    Self::CustomQuestion => "runtime question + TabButton.font",
                    _ => unreachable!(),
                },
                CHARACTER_CREATION_JEFFE_14_PATH_ID,
                Jeffe,
                CHARACTER_CREATION_JEFFE_14_FONT_SIZE,
                CHARACTER_CREATION_JEFFE_14_LINE_HEIGHT,
            ),
            Self::Transparent4 => (
                "Transparent4",
                CHARACTER_CREATION_CHALET_SMALL_PATH_ID,
                Chalet,
                CHARACTER_CREATION_CHALET_SMALL_FONT_SIZE,
                CHARACTER_CREATION_CHALET_SMALL_LINE_HEIGHT,
            ),
            Self::Transparent5 => (
                "Transparent5",
                CHARACTER_CREATION_CHALET_REGULAR_PATH_ID,
                Chalet,
                CHARACTER_CREATION_CHALET_REGULAR_FONT_SIZE,
                CHARACTER_CREATION_CHALET_REGULAR_LINE_HEIGHT,
            ),
        };
        let padding = match self {
            Self::Label => [0.0, 0.0, 3.0, 3.0],
            // Section captions already have tight, authored text rectangles.
            // Transparent's padding reduces SKIN's 41x12 rect to 25x2 and
            // makes the localization fitter shrink even this short caption.
            Self::SectionLabel => [0.0; 4],
            Self::Transparent | Self::BodyText | Self::NameDisplay => {
                [10.0, 6.0, 4.0, 6.0]
            }
            // The source toggle's -30 right padding is represented explicitly
            // by its 102px child content rect; Bevy layout never receives a
            // negative CSS-like padding value.
            Self::Toggle => [0.0, -30.0, 0.0, 0.0],
            Self::Button
            | Self::ButtonTabFont
            | Self::Transparent4
            | Self::Transparent5
            | Self::CustomQuestion => [0.0; 4],
            Self::ExitButton => [6.0, 6.0, 4.0, 6.0],
            Self::TabButton => [0.0, 0.0, 0.0, 5.0],
            Self::TabText | Self::OrText => [0.0, 0.0, 4.0, 7.0],
            Self::TextField => [3.0; 4],
        };
        let anchor = if matches!(self, Self::TextField) {
            MiddleLeft
        } else {
            MiddleCenter
        };
        let normal_color = match self {
            Self::Label => [0.799_270_1, 1.0, 1.0, 1.0],
            Self::SectionLabel | Self::CustomQuestion => {
                if matches!(self, Self::CustomQuestion) {
                    [1.0; 4]
                } else {
                    [0.0, 1.0, 1.0, 1.0]
                }
            }
            Self::TabButton => [0.898_039_2, 0.898_039_2, 0.898_039_2, 1.0],
            Self::Button | Self::ButtonTabFont => [0.9, 0.9, 0.9, 1.0],
            Self::OrText => [0.0, 1.0, 1.0, 1.0],
            Self::NameDisplay | Self::TextField => [1.0, 1.0, 0.0, 1.0],
            Self::Transparent
            | Self::Toggle
            | Self::BodyText
            | Self::ExitButton
            | Self::TabText
            | Self::Transparent4
            | Self::Transparent5 => [0.8, 1.0, 1.0, 1.0],
        };
        let word_wrap = matches!(self, Self::Label | Self::TabText | Self::OrText);
        let clip = !matches!(
            self,
            Self::Transparent
                | Self::SectionLabel
                | Self::BodyText
                | Self::ExitButton
                | Self::TabButton
                | Self::NameDisplay
        );
        CharacterCreationTextStyleSpec {
            source_style,
            source_font_path_id,
            font_role,
            font_size,
            line_height,
            padding,
            anchor,
            normal_color,
            word_wrap,
            clip,
            content_offset: [0.0, 0.0],
            y_offset: CHARACTER_CREATION_TEXT_Y_OFFSET,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CharacterCreationScreen {
    #[default]
    Name,
    Appearance,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(u8)]
pub enum CharacterGender {
    #[default]
    Boy = 1,
    Girl = 2,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AppearanceField {
    Height,
    Body,
    Hair,
    Face,
    Shirt,
    Pants,
    Shoes,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CharacterCreationOptionCounts {
    pub male_hair: u8,
    pub female_hair: u8,
    pub male_face: u8,
    pub female_face: u8,
    pub male_shirt: u8,
    pub female_shirt: u8,
    pub male_pants: u8,
    pub female_pants: u8,
    pub male_shoes: u8,
    pub female_shoes: u8,
}

impl Default for CharacterCreationOptionCounts {
    fn default() -> Self {
        // `m_pCreationItemData[1]` in the Retrobution XDT. These are counts,
        // while selectors in CnGuiCharCreation cycle through 2..=count+1.
        Self {
            male_hair: 23,
            female_hair: 21,
            male_face: 5,
            female_face: 5,
            male_shirt: 30,
            female_shirt: 30,
            male_pants: 30,
            female_pants: 30,
            male_shoes: 30,
            female_shoes: 27,
        }
    }
}

impl CharacterCreationOptionCounts {
    pub const fn count(self, gender: CharacterGender, field: AppearanceField) -> u8 {
        match (gender, field) {
            (_, AppearanceField::Height) => 5,
            (_, AppearanceField::Body) => 3,
            (CharacterGender::Boy, AppearanceField::Hair) => self.male_hair,
            (CharacterGender::Girl, AppearanceField::Hair) => self.female_hair,
            (CharacterGender::Boy, AppearanceField::Face) => self.male_face,
            (CharacterGender::Girl, AppearanceField::Face) => self.female_face,
            (CharacterGender::Boy, AppearanceField::Shirt) => self.male_shirt,
            (CharacterGender::Girl, AppearanceField::Shirt) => self.female_shirt,
            (CharacterGender::Boy, AppearanceField::Pants) => self.male_pants,
            (CharacterGender::Girl, AppearanceField::Pants) => self.female_pants,
            (CharacterGender::Boy, AppearanceField::Shoes) => self.male_shoes,
            (CharacterGender::Girl, AppearanceField::Shoes) => self.female_shoes,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CharacterAppearance {
    pub gender: CharacterGender,
    /// Protocol value, 1..=12.
    pub skin_color: u8,
    /// Protocol value, 1..=18.
    pub hair_color: u8,
    /// Protocol value, 1..=5.
    pub eye_color: u8,
    /// Exact CnGui selector, 0=HEAVY, 1=MEDIUM, 2=LIGHT.
    pub body: u8,
    /// Exact CnGui selector, 0=SHORTEST .. 4=TALLEST.
    pub height: u8,
    /// Creation-item table selector. Valid range starts at 2.
    pub hair: u8,
    /// Creation-item table selector. Valid range starts at 2.
    pub face: u8,
    /// Creation-item table selector. Valid range starts at 2.
    pub shirt: u8,
    /// Creation-item table selector. Valid range starts at 2.
    pub pants: u8,
    /// Creation-item table selector. Valid range starts at 2.
    pub shoes: u8,
}

impl Default for CharacterAppearance {
    fn default() -> Self {
        Self {
            gender: CharacterGender::Boy,
            skin_color: 1,
            hair_color: 1,
            eye_color: 1,
            body: 1,
            height: 2,
            hair: 2,
            face: 2,
            shirt: 2,
            pants: 2,
            shoes: 2,
        }
    }
}

impl CharacterAppearance {
    pub fn set_gender(&mut self, gender: CharacterGender, counts: CharacterCreationOptionCounts) {
        self.gender = gender;
        for field in [
            AppearanceField::Hair,
            AppearanceField::Face,
            AppearanceField::Shirt,
            AppearanceField::Pants,
            AppearanceField::Shoes,
        ] {
            let maximum = counts.count(gender, field) + 1;
            let selected = self.selector_mut(field);
            *selected = (*selected).clamp(2, maximum);
        }
    }

    pub fn step(
        &mut self,
        field: AppearanceField,
        delta: i8,
        counts: CharacterCreationOptionCounts,
    ) {
        if delta == 0 {
            return;
        }
        let (minimum, maximum) = match field {
            AppearanceField::Height => (0, 4),
            AppearanceField::Body => (0, 2),
            AppearanceField::Hair
            | AppearanceField::Face
            | AppearanceField::Shirt
            | AppearanceField::Pants
            | AppearanceField::Shoes => (2, counts.count(self.gender, field) + 1),
        };
        let selected = self.selector_mut(field);
        let span = i16::from(maximum - minimum + 1);
        let offset = i16::from(*selected) - i16::from(minimum) + i16::from(delta);
        *selected = minimum + offset.rem_euclid(span) as u8;
    }

    pub const fn selector(&self, field: AppearanceField) -> u8 {
        match field {
            AppearanceField::Height => self.height,
            AppearanceField::Body => self.body,
            AppearanceField::Hair => self.hair,
            AppearanceField::Face => self.face,
            AppearanceField::Shirt => self.shirt,
            AppearanceField::Pants => self.pants,
            AppearanceField::Shoes => self.shoes,
        }
    }

    pub(super) fn selector_mut(&mut self, field: AppearanceField) -> &mut u8 {
        match field {
            AppearanceField::Height => &mut self.height,
            AppearanceField::Body => &mut self.body,
            AppearanceField::Hair => &mut self.hair,
            AppearanceField::Face => &mut self.face,
            AppearanceField::Shirt => &mut self.shirt,
            AppearanceField::Pants => &mut self.pants,
            AppearanceField::Shoes => &mut self.shoes,
        }
    }

    pub const fn height_label(&self) -> &'static str {
        ["SHORTEST", "SHORT", "MEDIUM", "TALL", "TALLEST"][self.height as usize]
    }

    pub const fn body_label(&self) -> &'static str {
        ["HEAVY", "MEDIUM", "LIGHT"][self.body as usize]
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub enum CharacterNamePart {
    First = 0,
    Middle = 1,
    Last = 2,
}

#[derive(Clone, Debug, Default, Eq, PartialEq, Resource)]
pub struct CharacterNameLists {
    /// Exact XDT arrays, including the empty sentinel at element zero.
    pub first: Vec<String>,
    pub middle: Vec<String>,
    pub last: Vec<String>,
}

impl CharacterNameLists {
    pub fn valid(&self) -> bool {
        self.first.len() > 2 && self.middle.len() > 2 && self.last.len() > 2
    }

    pub(super) fn list(&self, part: CharacterNamePart) -> &[String] {
        match part {
            CharacterNamePart::First => &self.first,
            CharacterNamePart::Middle => &self.middle,
            CharacterNamePart::Last => &self.last,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratedCharacterName {
    pub first: String,
    pub last: String,
    pub first_index: usize,
    pub middle_index: usize,
    pub last_index: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CustomCharacterName {
    pub first: String,
    pub last: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CharacterCreationPending {
    NameTableNotPublished,
    CustomNameFilterNotPublished,
    CreationItemCatalogNotPublished,
    StarterClothingIconsNotPublished,
    PlayerAssemblyNotReady,
    ReserveNameNetworkCommand,
    SaveAppearanceNetworkCommand,
}

impl CharacterCreationPending {
    pub const fn message(self) -> &'static str {
        match self {
            Self::NameTableNotPublished => {
                "The native first/middle/last name table is not available"
            }
            Self::CustomNameFilterNotPublished => {
                "The native custom-name slang filter is not available"
            }
            Self::CreationItemCatalogNotPublished => {
                "The native creation-item selector catalog is not available"
            }
            Self::StarterClothingIconsNotPublished => {
                "The source starter-clothing icon textures are not published"
            }
            Self::PlayerAssemblyNotReady => "The native player preview assembly is not ready",
            Self::ReserveNameNetworkCommand => {
                "The native OpenFusion name reservation command is not wired"
            }
            Self::SaveAppearanceNetworkCommand => {
                "The native OpenFusion appearance save command is not wired"
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CharacterCreationCapability {
    Enabled,
    Pending(CharacterCreationPending),
}

impl CharacterCreationCapability {
    pub const fn enabled(self) -> bool {
        matches!(self, Self::Enabled)
    }
}

#[derive(Default, Resource)]
pub struct CharacterCreationUiOutbox {
    pub(super) actions: VecDeque<CharacterCreationUiAction>,
}

impl CharacterCreationUiOutbox {
    pub fn push(&mut self, action: CharacterCreationUiAction) {
        self.actions.push_back(action);
    }

    pub fn drain(&mut self) -> impl Iterator<Item = CharacterCreationUiAction> + '_ {
        self.actions.drain(..)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CharacterCreationControl {
    Exit,
    ToggleFullscreen,
    ContinueAppearance,
    RandomAppearance,
    Camera(CharacterCreationCameraAction),
    Gender(CharacterGender),
    Step(AppearanceField, i8),
    ClothingChoice(AppearanceField, i8),
    Skin(u8),
    ColorPage(CharacterCreationColorKind, i8),
    HairColor(u8),
    EyeColor(u8),
    NameMode(CharacterNameMode),
    NameScroll(CharacterNamePart, i8),
    RandomName,
    FocusCustomName,
    ContinueName,
}

#[derive(Component)]
pub(super) struct CreationClothingIcon {
    pub(super) row: usize,
    pub(super) column: usize,
}

#[derive(Component)]
pub struct NativeCharacterCreationRoot;

#[derive(Component)]
pub(super) struct CreationBackground;

#[derive(Component)]
pub(super) struct AppearanceRoot;

#[derive(Component)]
pub(super) struct NameRoot;

#[derive(Component)]
pub(super) struct CreationFullscreen;

#[derive(Component)]
pub(super) struct CharacterCreationBaseCamera;

#[derive(Component)]
pub(super) struct CharacterCreationForegroundCamera;

#[derive(Component)]
pub(super) struct CreationPreviewControlsRoot;

#[derive(Component)]
pub(super) struct CreationMusic;

#[derive(Component)]
pub(super) struct AppearanceBodyText;

#[derive(Component)]
pub(super) struct AppearanceHairText;

#[derive(Component)]
pub(super) struct AppearanceFaceText;

#[derive(Component)]
pub(super) struct CustomNameText;

#[derive(Component)]
pub(super) struct GeneratedNameText;

#[derive(Component)]
pub(super) struct NameColumnText(pub(super) CharacterNamePart, pub(super) usize);

#[derive(Component)]
pub(super) struct ColorSwatch(pub(super) CharacterCreationColorKind, pub(super) u8);

#[derive(Resource)]
pub(super) struct CharacterCreationRandom(pub(super) u64);

impl Default for CharacterCreationRandom {
    fn default() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0x4343_4e41_4d45, |duration| duration.as_nanos() as u64);
        Self(seed | 1)
    }
}

impl CharacterCreationRandom {
    pub(super) fn below(&mut self, exclusive_maximum: usize) -> usize {
        if exclusive_maximum <= 1 {
            return 0;
        }
        // Native equivalent of the legacy DateTime-seeded Unity random calls.
        // Exact sequences were never stable between legacy process launches.
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0 as usize % exclusive_maximum
    }

    pub(super) fn inclusive(&mut self, minimum: u8, maximum: u8) -> u8 {
        minimum + self.below(usize::from(maximum - minimum + 1)) as u8
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CharacterCreationColorKind {
    Skin,
    Hair,
    Eye,
}

impl CharacterCreationColorKind {
    pub(super) fn index(self) -> usize {
        match self {
            Self::Skin => 0,
            Self::Hair => 1,
            Self::Eye => 2,
        }
    }
    pub(super) fn page_size(self) -> u8 {
        match self {
            Self::Skin => 12,
            Self::Hair => 18,
            Self::Eye => 5,
        }
    }
    pub(super) fn offset(self, model: &CharacterCreationUiModel) -> u8 {
        model.color_pages[self.index()] * self.page_size()
    }
}

#[derive(Clone, Copy)]
pub struct CharacterCreationImageSpec {
    pub true_name: &'static str,
    pub path: &'static str,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Resource)]
pub(super) struct CharacterCreationAssets {
    pub(super) images: BTreeMap<&'static str, Handle<Image>>,
    pub(super) sounds: BTreeMap<CharacterCreationSound, Handle<AudioSource>>,
    pub(super) button_sounds: [Handle<AudioSource>; 5],
    pub(super) music: Handle<AudioSource>,
    pub(super) font: Handle<Font>,
    pub(super) display_font: Handle<Font>,
}

impl CharacterCreationAssets {
    pub(super) fn load(asset_server: &AssetServer) -> Self {
        let images = CHARACTER_CREATION_IMAGE_SPECS
            .iter()
            .chain(CHARACTER_CREATION_SHARED_IMAGE_SPECS.iter())
            .chain(CHARACTER_CREATION_ENGINE_IMAGE_SPECS.iter())
            .map(|spec| (spec.path, asset_server.load(spec.path)))
            .collect();
        let sounds = [
            (
                CharacterCreationSound::Continue,
                CHARACTER_CREATION_CONTINUE_SOUND_PATH,
            ),
            (
                CharacterCreationSound::Random,
                CHARACTER_CREATION_RANDOM_SOUND_PATH,
            ),
            (
                CharacterCreationSound::Tab,
                CHARACTER_CREATION_TAB_SOUND_PATH,
            ),
            (
                CharacterCreationSound::Color,
                CHARACTER_CREATION_COLOR_SOUND_PATH,
            ),
            (
                CharacterCreationSound::HeightDown,
                CHARACTER_CREATION_HEIGHT_DOWN_SOUND_PATH,
            ),
            (
                CharacterCreationSound::HeightUp,
                CHARACTER_CREATION_HEIGHT_UP_SOUND_PATH,
            ),
            (
                CharacterCreationSound::GirthNarrow,
                CHARACTER_CREATION_GIRTH_NARROW_SOUND_PATH,
            ),
            (
                CharacterCreationSound::GirthWide,
                CHARACTER_CREATION_GIRTH_WIDE_SOUND_PATH,
            ),
        ]
        .map(|(cue, path)| (cue, asset_server.load(path)))
        .into_iter()
        .collect();
        Self {
            images,
            sounds,
            button_sounds: CHARACTER_CREATION_BUTTON_SOUND_PATHS
                .map(|path| asset_server.load(path)),
            music: asset_server.load(CHARACTER_CREATION_MUSIC_PATH),
            font: asset_server.load(CHARACTER_CREATION_FONT_PATH),
            display_font: asset_server.load(CHARACTER_CREATION_DISPLAY_FONT_PATH),
        }
    }

    pub(super) fn image(&self, path: &'static str) -> Handle<Image> {
        self.images
            .get(path)
            .unwrap_or_else(|| panic!("character-creation image was not registered: {path}"))
            .clone()
    }
}

#[derive(Default)]
pub struct NativeCharacterCreationUiPlugin;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum CharacterCreationUiSet {
    Assets,
    Layout,
    Interaction,
    Bind,
    Audio,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub(super) struct CharacterCreationStartupSet;
