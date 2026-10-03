use super::*;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoginImageSpec {
    pub role: &'static str,
    pub path: &'static str,
    pub source_asset: &'static str,
    pub source_path_id: i64,
    pub width: u32,
    pub height: u32,
    pub bytes: u64,
    pub sha256: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoginReplacementFontSpec {
    pub role: &'static str,
    pub path: &'static str,
    pub source_font_path_id: i64,
    pub bytes: u64,
    pub sha256: &'static str,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LoginBackgroundMode0104 {
    /// The initial full-screen `BlackTexture` draw, with no later image.
    ///
    /// Clean path 270 decodes to 64 identical `(0, 0, 0, 255)` pixels, so a
    /// solid Bevy background is the exact pixel adapter and avoids publishing
    /// a redundant runtime texture.
    Black,
    /// Serialized path 369 (`load`) using Unity `ScaleMode::ScaleToFit`.
    #[default]
    FallbackScaleToFit,
    /// Dynamically loaded `login_screen_bg_16x10.png` using ScaleAndCrop.
    LoadedScaleAndCrop,
}

impl LoginBackgroundMode0104 {
    pub(super) fn resolved_rect(self, viewport: Vec2) -> Option<UiResolvedRect> {
        let (source, crop) = match self {
            Self::Black => return None,
            Self::FallbackScaleToFit => (LOGIN_FALLBACK_BACKGROUND_SIZE, false),
            Self::LoadedScaleAndCrop => (LOGIN_BACKGROUND_SIZE, true),
        };
        let x_scale = viewport.x / source.x;
        let y_scale = viewport.y / source.y;
        let scale = if crop {
            x_scale.max(y_scale)
        } else {
            x_scale.min(y_scale)
        };
        let size = source * scale;
        Some(UiResolvedRect {
            origin: (viewport - size) * 0.5,
            size,
        })
    }
}

/// Exact clean `FusionFallSkin` text role attached to every login `Text`.
#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub enum LoginTextStyle0104 {
    Label,
    Button,
    TextField,
    /// Native connection-status adapter; hidden during the clean idle form.
    StatusAdapter,
}

impl LoginTextStyle0104 {
    pub(super) fn font(self, assets: &LoginUiAssets) -> (TextFont, LineHeight) {
        let (font, font_size, line_height) = match self {
            Self::TextField => (
                assets.text_field_font.clone(),
                LOGIN_CHALET_FONT_SIZE,
                LOGIN_CHALET_LINE_HEIGHT,
            ),
            Self::Label | Self::Button => (
                assets.font.clone(),
                LOGIN_JEFFE_FONT_SIZE,
                LOGIN_JEFFE_LINE_HEIGHT,
            ),
            Self::StatusAdapter => (assets.font.clone(), 11.0, LOGIN_JEFFE_LINE_HEIGHT),
        };
        (
            TextFont {
                font: (font).into(),
                font_size: (font_size).into(),
                font_smoothing: FontSmoothing::AntiAliased,
                ..default()
            },
            LineHeight::Px(line_height),
        )
    }

    pub(super) fn layout(self) -> TextLayout {
        match self {
            Self::Button => TextLayout::new(Justify::Center, LineBreak::NoWrap),
            Self::TextField => TextLayout::new(Justify::Left, LineBreak::NoWrap),
            Self::Label | Self::StatusAdapter => {
                TextLayout::new(Justify::Left, LineBreak::WordBoundary)
            }
        }
    }

    pub(super) const fn y_offset(self) -> f32 {
        match self {
            Self::Label => LOGIN_LABEL_Y_OFFSET,
            Self::Button => LOGIN_BUTTON_Y_OFFSET,
            Self::TextField => LOGIN_TEXT_FIELD_Y_OFFSET,
            Self::StatusAdapter => LOGIN_STATUS_ADAPTER_Y_OFFSET,
        }
    }

    pub(super) fn apply_to_control(self, node: &mut Node) {
        match self {
            Self::Label => {
                node.justify_content = JustifyContent::FlexStart;
                node.align_items = AlignItems::FlexStart;
                node.padding = LOGIN_LABEL_PADDING;
            }
            Self::Button => {
                node.justify_content = JustifyContent::Center;
                node.align_items = AlignItems::Center;
                node.padding = LOGIN_BUTTON_PADDING;
            }
            Self::TextField => {
                node.justify_content = JustifyContent::FlexStart;
                node.align_items = AlignItems::FlexStart;
                node.padding = LOGIN_TEXT_FIELD_PADDING;
            }
            Self::StatusAdapter => {
                node.justify_content = JustifyContent::FlexStart;
                node.align_items = AlignItems::FlexStart;
            }
        }
        node.overflow = Overflow::clip();
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LoginField {
    #[default]
    Username,
    Password,
}

/// Visible branch selected by `CnLoginMode` before `CnGuiLogin.OnGUI`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum LoginSurface {
    /// `bLogInput == true`: draw the background and the manual credential panel.
    #[default]
    Manual,
    /// The three-second browser-auth callback window: background only.
    WaitingForWebAuthentication,
    /// Cookie authentication is in flight: background only.
    WebAuthentication,
    /// `bAutoLogin == true`: the legacy GUI returns after drawing only black.
    AutoLogin,
    /// `bWarpShard == true`: draw the background and return before the form.
    WarpShard,
}

#[derive(Default, Resource)]
pub struct LoginUiOutbox {
    pub(super) requests: VecDeque<LoginRequest>,
}

impl LoginUiOutbox {
    pub fn drain(&mut self) -> impl Iterator<Item = LoginRequest> + '_ {
        self.requests.drain(..)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LoginUiEffect {
    OpenCommunity { url: &'static str },
    ShowRegistrationInstructions { message: &'static str },
}

#[derive(Default, Resource)]
pub struct LoginUiEffectOutbox {
    pub(super) effects: VecDeque<LoginUiEffect>,
}

impl LoginUiEffectOutbox {
    pub fn drain(&mut self) -> impl Iterator<Item = LoginUiEffect> + '_ {
        self.effects.drain(..)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum LoginUiSet {
    Interaction,
    Assets,
    Bind,
}

#[derive(Default)]
pub struct NativeLoginUiPlugin;

impl Plugin for NativeLoginUiPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<TextEditPlugin>() {
            app.add_plugins(TextEditPlugin);
        }
        app.init_resource::<LoginUiModel>()
            .init_resource::<LoginUiOutbox>()
            .init_resource::<LoginUiEffectOutbox>()
            .init_resource::<LoginLoadedBackgroundState>()
            .init_resource::<LoginUiAssetStatus>()
            .add_systems(Startup, spawn_login_ui)
            .configure_sets(
                Update,
                (
                    LoginUiSet::Interaction,
                    LoginUiSet::Assets,
                    LoginUiSet::Bind,
                )
                    .chain(),
            )
            .add_systems(
                Update,
                (
                    handle_login_keyboard,
                    handle_login_interactions,
                    handle_login_language_interaction,
                    handle_login_edit_pointer,
                    bind_login_edit_visuals,
                )
                    .chain()
                    .in_set(LoginUiSet::Interaction),
            )
            .add_systems(
                Update,
                update_login_ui_asset_status.in_set(LoginUiSet::Assets),
            )
            .add_systems(
                Update,
                (update_login_layout, bind_login_ui, bind_login_language, control_login_music)
                    .chain()
                    .in_set(LoginUiSet::Bind)
                    .before(LocalizationSet::Apply),
            );
    }
}

#[derive(Clone, Resource)]
pub(super) struct LoginUiAssets {
    pub(super) background: Handle<Image>,
    pub(super) fallback_background: Handle<Image>,
    pub(super) panel: Handle<Image>,
    pub(super) button: Handle<Image>,
    pub(super) button_over: Handle<Image>,
    pub(super) button_active: Handle<Image>,
    pub(super) text_field: Handle<Image>,
    pub(super) font: Handle<Font>,
    pub(super) text_field_font: Handle<Font>,
    pub(super) music: Handle<AudioSource>,
}

#[derive(Component)]
pub(super) struct LoginRoot;

#[derive(Component)]
pub(super) struct LoginLoadedBackground;

#[derive(Component)]
pub(super) struct LoginFallbackBackground;

#[derive(Component)]
pub(super) struct LoginPanel;

#[derive(Component)]
pub(super) struct LoginUsernameField;

#[derive(Component)]
pub(super) struct LoginPasswordField;

#[derive(Component)]
pub(super) struct LoginUsernameText;

#[derive(Component)]
pub(super) struct LoginPasswordText;

#[derive(Component)]
pub(super) struct LoginSubmitText;

#[derive(Component)]
pub(super) struct LoginLanguageButton;

#[derive(Component)]
pub(super) struct LoginLanguageText;

#[derive(Component)]
pub(super) struct LoginMusic;
