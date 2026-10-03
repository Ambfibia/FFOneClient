use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LauncherTriggerSpec {
    pub trigger_position: Vec3,
    pub trigger_euler_degrees: Vec3,
    pub min_power: f32,
    pub max_power: f32,
    pub initial_rotation_degrees: Vec3,
    pub maximum_rotation_degrees: Vec3,
}

impl LauncherTriggerSpec {
    pub(super) fn validate(self) -> Result<Self, LauncherUiOpenError> {
        let values = [
            self.trigger_position.x,
            self.trigger_position.y,
            self.trigger_position.z,
            self.trigger_euler_degrees.x,
            self.trigger_euler_degrees.y,
            self.trigger_euler_degrees.z,
            self.min_power,
            self.max_power,
            self.initial_rotation_degrees.x,
            self.initial_rotation_degrees.y,
            self.initial_rotation_degrees.z,
            self.maximum_rotation_degrees.x,
            self.maximum_rotation_degrees.y,
            self.maximum_rotation_degrees.z,
        ];
        if values.into_iter().any(|value| !value.is_finite()) {
            return Err(LauncherUiOpenError::NonFiniteTrigger);
        }
        if self.max_power <= self.min_power {
            return Err(LauncherUiOpenError::InvalidPowerRange);
        }
        if self.maximum_rotation_degrees.x < 0.0 || self.maximum_rotation_degrees.y < 0.0 {
            return Err(LauncherUiOpenError::NegativeAimLimit);
        }
        Ok(self)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LauncherUiPhase {
    Hidden,
    Aiming,
    AwaitingEscapeGate,
}

impl Default for LauncherUiPhase {
    fn default() -> Self {
        Self::Hidden
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LauncherUiDismissalSource {
    Fired,
    Death,
    EscapeGate,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LauncherShot {
    pub position: Vec3,
    pub velocity: Vec3,
    pub forward: Vec3,
    pub facing_yaw_degrees: f32,
    pub power: f32,
    pub request_packet_id: u32,
    pub request_packet_size: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LauncherUiEffect {
    Audio(LauncherUiAudioCue),
    SetCombatIcon(i32),
    SetNameVisible(bool),
    SetTriggerRenderersVisible(bool),
    SetAvatarRenderersVisible(bool),
    SetCameraPosition(Vec3),
    SetCameraCustomControl(bool),
    SetCameraRotationY(f32),
    StartLauncher(LauncherShot),
    RequestEscapeCloseGate {
        event_group: u8,
        event_function: u8,
    },
    ExitMode {
        source: LauncherUiDismissalSource,
        event_group: u8,
        event_function: u8,
    },
    RestoreAvatarPosition(Vec3),
}

#[derive(Clone, Debug, Default, PartialEq, Resource)]
pub struct LauncherUiOutbox(pub(super) VecDeque<LauncherUiEffect>);

impl LauncherUiOutbox {
    pub fn push(&mut self, effect: LauncherUiEffect) {
        self.0.push_back(effect);
    }

    pub fn pop_front(&mut self) -> Option<LauncherUiEffect> {
        self.0.pop_front()
    }

    pub fn drain(&mut self) -> impl Iterator<Item = LauncherUiEffect> + '_ {
        self.0.drain(..)
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Resource)]
pub struct LauncherUiLabels {
    pub power: String,
    pub tip: String,
}

impl Default for LauncherUiLabels {
    fn default() -> Self {
        Self {
            power: LAUNCHER_UI_POWER_LABEL_KEY.to_owned(),
            tip: LAUNCHER_UI_TIP_LABEL_KEY.to_owned(),
        }
    }
}

impl LauncherUiLabels {
    pub(super) fn power_localized(&self) -> LocalizedText {
        if self.power == LAUNCHER_UI_POWER_LABEL_KEY {
            LocalizedText::new(
                LAUNCHER_UI_POWER_LOCALIZATION_KEY,
                LAUNCHER_UI_POWER_LABEL_KEY,
            )
        } else {
            LocalizedText::new("ui.content.passthrough", "{text}")
                .with_arg("text", self.power.clone())
        }
    }

    pub(super) fn tip_localized(&self) -> LocalizedText {
        if self.tip == LAUNCHER_UI_TIP_LABEL_KEY {
            LocalizedText::new(LAUNCHER_UI_TIP_LOCALIZATION_KEY, LAUNCHER_UI_TIP_LABEL_KEY)
        } else {
            LocalizedText::new("ui.content.passthrough", "{text}")
                .with_arg("text", self.tip.clone())
        }
    }
}

#[derive(Clone, Resource)]
pub struct LauncherUiAssets {
    pub backdrop: Handle<Image>,
    pub crosshair: Handle<Image>,
    pub gauge: Handle<Image>,
    pub gauge_bar: Handle<Image>,
    pub font: Handle<Font>,
}

impl LauncherUiAssets {
    pub(super) fn load(asset_server: &AssetServer) -> Self {
        Self {
            backdrop: asset_server.load(LAUNCHER_UI_BACKDROP_PATH),
            crosshair: asset_server.load(LAUNCHER_UI_CROSSHAIR_PATH),
            gauge: asset_server.load(LAUNCHER_UI_GAUGE_PATH),
            gauge_bar: asset_server.load(LAUNCHER_UI_GAUGE_BAR_PATH),
            font: asset_server.load(LAUNCHER_UI_FONT_PATH),
        }
    }
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub enum LauncherUiElement {
    Root,
    Backdrop(usize),
    Crosshair,
    Gauge,
    GaugeBar,
    PowerLabel,
    TipLabel,
}

#[derive(Clone, Copy, Component, Debug, Eq, PartialEq)]
pub enum LauncherUiTextRole {
    Power,
    Tip,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum LauncherUiSet {
    Aim,
    Interaction,
    Bind,
}

#[derive(Default)]
pub struct LauncherUiPlugin;

impl Plugin for LauncherUiPlugin {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<LauncherUiModel>()
            .init_resource::<LauncherUiLabels>()
            .init_resource::<LauncherUiExternalState>()
            .init_resource::<LauncherUiOutbox>()
            .add_systems(
                crate::ui_startup::NativeGameplayUiStartup,
                spawn_launcher_ui,
            )
            .configure_sets(
                Update,
                (LauncherUiSet::Interaction, LauncherUiSet::Bind).chain(),
            )
            .configure_sets(Update, LauncherUiSet::Bind.before(LocalizationSet::Apply))
            .add_systems(
                Update,
                (handle_launcher_keyboard.in_set(LauncherUiSet::Interaction))
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                Update,
                ((sync_launcher_layout, sync_launcher_labels)
                    .chain()
                    .in_set(LauncherUiSet::Bind))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            )
            .add_systems(
                FixedUpdate,
                apply_launcher_fixed_aim.in_set(LauncherUiSet::Aim).in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}
