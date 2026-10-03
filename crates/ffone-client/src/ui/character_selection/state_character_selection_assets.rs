use super::*;

impl CharacterSelectionTextStyle {
    #[must_use]
    pub const fn spec(self) -> CharacterSelectionTextStyleSpec {
        use CharacterSelectionFontRole::{Chalet, Jeffe};
        use CharacterSelectionTextAnchor::{MiddleCenter, MiddleLeft};

        let (source_font_path_id, font_role, font_size, line_height) = match self {
            Self::Transparent2 | Self::Transparent3 | Self::QuitButton | Self::CreateButton => (
                CHARACTER_SELECTION_JEFFE_14_PATH_ID,
                Jeffe,
                CHARACTER_SELECTION_JEFFE_14_FONT_SIZE,
                CHARACTER_SELECTION_JEFFE_14_LINE_HEIGHT,
            ),
            Self::Cancel => (
                CHARACTER_SELECTION_JEFFE_12_PATH_ID,
                Jeffe,
                CHARACTER_SELECTION_JEFFE_12_FONT_SIZE,
                CHARACTER_SELECTION_JEFFE_12_LINE_HEIGHT,
            ),
            Self::EnterGame => (
                CHARACTER_SELECTION_JEFFE_16_PATH_ID,
                Jeffe,
                CHARACTER_SELECTION_JEFFE_16_FONT_SIZE,
                CHARACTER_SELECTION_JEFFE_16_LINE_HEIGHT,
            ),
            Self::CharNameUp | Self::CharNameDown | Self::CharLevelUp | Self::DeleteText => (
                CHARACTER_SELECTION_CHALET_SMALL_PATH_ID,
                Chalet,
                CHARACTER_SELECTION_CHALET_SMALL_FONT_SIZE,
                CHARACTER_SELECTION_CHALET_SMALL_LINE_HEIGHT,
            ),
            Self::AvatarName => (
                CHARACTER_SELECTION_CHALET_REGULAR_PATH_ID,
                Chalet,
                CHARACTER_SELECTION_CHALET_REGULAR_FONT_SIZE,
                CHARACTER_SELECTION_CHALET_REGULAR_LINE_HEIGHT,
            ),
        };
        let padding = match self {
            Self::EnterGame => [0.0; 4],
            Self::Cancel => CHARACTER_SELECTION_CANCEL_PADDING,
            Self::Transparent2
            | Self::Transparent3
            | Self::CharNameUp
            | Self::CharNameDown
            | Self::CharLevelUp
            | Self::DeleteText
            | Self::AvatarName
            | Self::QuitButton
            | Self::CreateButton => CHARACTER_SELECTION_STYLE_PADDING,
        };
        let anchor = match self {
            Self::CharNameUp | Self::CharNameDown | Self::CharLevelUp => MiddleLeft,
            Self::Transparent2
            | Self::Transparent3
            | Self::DeleteText
            | Self::AvatarName
            | Self::QuitButton
            | Self::CreateButton
            | Self::EnterGame
            | Self::Cancel => MiddleCenter,
        };
        let normal_color = match self {
            Self::Transparent2 => [0.0, 0.0, 0.405_109_5, 1.0],
            Self::Transparent3
            | Self::DeleteText
            | Self::AvatarName
            | Self::EnterGame
            | Self::Cancel => [0.8, 1.0, 1.0, 1.0],
            Self::CharNameUp | Self::CreateButton => [0.898_039_2, 0.898_039_2, 0.898_039_2, 1.0],
            Self::CharNameDown => [0.0, 0.178_832_11, 0.558_394_13, 1.0],
            Self::CharLevelUp => [0.0, 0.8, 1.0, 1.0],
            Self::QuitButton => [1.0; 4],
        };
        let y_offset = match self {
            Self::Transparent2 => CHARACTER_SELECTION_TRANSPARENT2_Y_OFFSET,
            Self::Transparent3 => CHARACTER_SELECTION_TRANSPARENT3_Y_OFFSET,
            Self::CharNameUp => CHARACTER_SELECTION_CHAR_NAME_UP_Y_OFFSET,
            Self::CharNameDown => CHARACTER_SELECTION_CHAR_NAME_DOWN_Y_OFFSET,
            Self::CharLevelUp => CHARACTER_SELECTION_CHAR_LEVEL_UP_Y_OFFSET,
            Self::DeleteText => CHARACTER_SELECTION_DELETE_TEXT_Y_OFFSET,
            Self::AvatarName => CHARACTER_SELECTION_AVATAR_NAME_Y_OFFSET,
            Self::QuitButton => CHARACTER_SELECTION_QUIT_BUTTON_Y_OFFSET,
            Self::CreateButton => CHARACTER_SELECTION_CREATE_BUTTON_Y_OFFSET,
            Self::EnterGame => CHARACTER_SELECTION_ENTER_GAME_Y_OFFSET,
            Self::Cancel => CHARACTER_SELECTION_CANCEL_Y_OFFSET,
        };
        CharacterSelectionTextStyleSpec {
            source_font_path_id,
            font_role,
            font_size,
            line_height,
            padding,
            anchor,
            normal_color,
            word_wrap: false,
            y_offset,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResolvedCharacterSelectionLocation {
    pub district: &'static str,
    pub zone: &'static str,
    pub background: CharacterLocationBackground,
}

/// Without a catalog the source text is kept, as the diagnostics path shows it.
pub(super) fn character_selection_location_part(
    source: &str,
    localization: Option<&Localization>,
    language: Option<&Language>,
) -> String {
    let (Some(localization), Some(language)) = (localization, language) else {
        return source.to_owned();
    };
    let text = match source {
        "CHARACTER CREATION" => {
            LocalizedText::new("ui.character_select.location.character_creation", source)
        }
        "UNKNOWN" => LocalizedText::new("ui.character_select.location.unknown", source),
        _ => localized_world_location_text(source),
    };
    localization.text(language, &text).to_uppercase()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CharacterSelectionPending {
    CreateCharacterNetworkCommand,
    DeleteCharacterNetworkCommand,
}

impl CharacterSelectionPending {
    pub const fn message(self) -> &'static str {
        match self {
            Self::CreateCharacterNetworkCommand => {
                "Create character is pending a native OpenFusion network command"
            }
            Self::DeleteCharacterNetworkCommand => {
                "Delete character is pending a native OpenFusion network command"
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CharacterSelectionCapability {
    Enabled,
    Pending(CharacterSelectionPending),
}

impl CharacterSelectionCapability {
    pub const fn enabled(self) -> bool {
        matches!(self, Self::Enabled)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CharacterPreviewStatus {
    #[default]
    PlayerAssemblyPending,
    Ready,
    Unavailable,
}

#[derive(Debug, Default, Resource)]
pub(super) struct CharacterSelectionBackgroundClock {
    pub(super) elapsed_seconds: f32,
    pub(super) was_visible: bool,
}

impl CharacterSelectionBackgroundClock {
    pub(super) fn update(&mut self, visible: bool, delta_seconds: f32) -> f32 {
        if !visible {
            self.elapsed_seconds = 0.0;
            self.was_visible = false;
            return 0.0;
        }
        if self.was_visible {
            self.elapsed_seconds += delta_seconds.max(0.0);
        } else {
            self.elapsed_seconds = 0.0;
            self.was_visible = true;
        }
        self.elapsed_seconds
    }
}

#[derive(Resource)]
pub(super) struct CharacterSelectionRandom(pub(super) u64);

impl Default for CharacterSelectionRandom {
    fn default() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0x4353_4255_5454_4f4e, |duration| duration.as_nanos() as u64);
        Self(seed | 1)
    }
}

impl CharacterSelectionRandom {
    pub(super) fn below(&mut self, exclusive_maximum: usize) -> usize {
        if exclusive_maximum <= 1 {
            return 0;
        }
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0 as usize % exclusive_maximum
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CharacterSelectionUiAction {
    SelectSlot(usize),
    EnterSelected,
    /// Opens creation for the one-based OpenFusion character slot.
    CreateCharacter {
        slot: usize,
    },
    DeleteSelected {
        pc_uid: i64,
    },
    ToggleMusic(bool),
    ToggleFullscreen,
    Quit,
}

#[derive(Default, Resource)]
pub struct CharacterSelectionUiOutbox {
    pub(super) actions: VecDeque<CharacterSelectionUiAction>,
}

impl CharacterSelectionUiOutbox {
    pub fn push(&mut self, action: CharacterSelectionUiAction) {
        self.actions.push_back(action);
    }

    pub fn drain(&mut self) -> impl Iterator<Item = CharacterSelectionUiAction> + '_ {
        self.actions.drain(..)
    }
}

#[derive(Default)]
pub struct NativeCharacterSelectionUiPlugin;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub enum CharacterSelectionUiSet {
    Assets,
    Layout,
    Interaction,
    Bind,
    Audio,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, SystemSet)]
pub(super) struct CharacterSelectionStartupSet;

impl Plugin for NativeCharacterSelectionUiPlugin {
    fn build(&self, app: &mut App) {
        let deferred_in_production = app
            .world()
            .get_resource::<State<crate::ui_startup::NativeUiStartupPhase>>()
            .is_some_and(|phase| phase.get() == &crate::ui_startup::NativeUiStartupPhase::Deferred);

        app.init_resource::<CharacterSelectionUiModel>()
            .init_resource::<CharacterSelectionUiOutbox>()
            .init_resource::<CharacterSelectionBackgroundClock>()
            .init_resource::<CharacterSelectionRandom>()
            .configure_sets(
                Update,
                (
                    CharacterSelectionUiSet::Assets,
                    CharacterSelectionUiSet::Layout,
                    CharacterSelectionUiSet::Interaction,
                    CharacterSelectionUiSet::Bind,
                )
                    .chain()
                    .in_set(CharacterSelectionStartupSet),
            )
            .configure_sets(
                Update,
                CharacterSelectionUiSet::Audio.after(CharacterSelectionUiSet::Bind),
            )
            .add_systems(
                Update,
                update_character_selection_asset_status.in_set(CharacterSelectionUiSet::Assets),
            )
            .add_systems(
                Update,
                update_character_selection_layout.in_set(CharacterSelectionUiSet::Layout),
            )
            .add_systems(
                Update,
                (
                    handle_character_selection_interactions,
                    edit_character_delete_name,
                )
                    .chain()
                    .in_set(CharacterSelectionUiSet::Interaction),
            )
            .add_systems(
                Update,
                (
                    bind_character_selection_background_and_slots,
                    bind_character_selection_slot_text,
                    bind_character_selection_decorations,
                    bind_character_selection_control_visibility,
                    bind_character_selection_preview_loading,
                    bind_character_selection_controls,
                    bind_character_selection_delete_modal,
                )
                    .in_set(CharacterSelectionUiSet::Bind)
                    .before(LocalizationSet::Apply),
            )
            .add_systems(
                Update,
                control_character_selection_music.in_set(CharacterSelectionUiSet::Audio),
            )
            // The deferred startup set stops running as soon as gameplay UI
            // takes ownership. Camera activity must still observe the
            // selection model becoming hidden on that transition; otherwise
            // the selection base camera remains active beside the gameplay UI
            // camera at order 100 and Bevy reports/renders the ambiguity every
            // frame in the open world.
            .add_systems(Update, sync_character_selection_camera_activity);

        if deferred_in_production {
            app.configure_sets(
                Update,
                CharacterSelectionStartupSet.run_if(in_state(
                    crate::ui_startup::NativeUiStartupPhase::CharacterSelection,
                )),
            )
            .add_systems(
                OnEnter(crate::ui_startup::NativeUiStartupPhase::CharacterSelection),
                spawn_character_selection_ui,
            );
        } else {
            app.add_systems(Startup, spawn_character_selection_ui);
        }
    }
}

#[derive(Clone, Resource)]
pub(super) struct CharacterSelectionAssets {
    pub(super) chrome: Handle<Image>,
    pub(super) backgrounds: [[Handle<Image>; CHARACTER_SELECTION_BACKGROUND_FRAME_COUNT]; 5],
    pub(super) slot_empty: Handle<Image>,
    pub(super) slot_normal: Handle<Image>,
    pub(super) slot_over: Handle<Image>,
    pub(super) slot_locked: Handle<Image>,
    pub(super) lock: Handle<Image>,
    pub(super) enter: Handle<Image>,
    pub(super) enter_over: Handle<Image>,
    pub(super) fullscreen: Handle<Image>,
    pub(super) fullscreen_over: Handle<Image>,
    pub(super) windowed: Handle<Image>,
    pub(super) windowed_over: Handle<Image>,
    pub(super) music_toggle_off: Handle<Image>,
    pub(super) music_toggle_on: Handle<Image>,
    pub(super) red_button: Handle<Image>,
    pub(super) red_button_over: Handle<Image>,
    pub(super) blue_button: Handle<Image>,
    pub(super) blue_button_over: Handle<Image>,
    pub(super) delete_backdrop: Handle<Image>,
    pub(super) delete_window: Handle<Image>,
    pub(super) cancel_normal: Handle<Image>,
    pub(super) rotate_left: Handle<Image>,
    pub(super) rotate_left_over: Handle<Image>,
    pub(super) rotate_right: Handle<Image>,
    pub(super) rotate_right_over: Handle<Image>,
    pub(super) disk_back: Handle<Image>,
    pub(super) disk_front: Handle<Image>,
    pub(super) button_sounds: [Handle<AudioSource>; 5],
    pub(super) delete_yes_sound: Handle<AudioSource>,
    pub(super) delete_no_sound: Handle<AudioSource>,
    pub(super) music: Handle<AudioSource>,
    pub(super) chalet_font: Handle<Font>,
    pub(super) jeffe_font: Handle<Font>,
}

impl CharacterSelectionAssets {
    pub(super) fn load(asset_server: &AssetServer) -> Self {
        Self {
            chrome: asset_server.load(CHARACTER_SELECTION_CHROME_PATH),
            backgrounds: std::array::from_fn(|location| {
                let location = CharacterLocationBackground::ALL[location];
                std::array::from_fn(|frame| asset_server.load(location.asset_path(frame + 1)))
            }),
            slot_empty: asset_server.load(CHARACTER_SELECTION_SLOT_EMPTY_PATH),
            slot_normal: asset_server.load(CHARACTER_SELECTION_SLOT_NORMAL_PATH),
            slot_over: asset_server.load(CHARACTER_SELECTION_SLOT_OVER_PATH),
            slot_locked: asset_server.load(CHARACTER_SELECTION_SLOT_LOCKED_PATH),
            lock: asset_server.load(CHARACTER_SELECTION_LOCK_PATH),
            enter: asset_server.load(CHARACTER_SELECTION_ENTER_PATH),
            enter_over: asset_server.load(CHARACTER_SELECTION_ENTER_OVER_PATH),
            fullscreen: asset_server.load(CHARACTER_SELECTION_FULLSCREEN_PATH),
            fullscreen_over: asset_server.load(CHARACTER_SELECTION_FULLSCREEN_OVER_PATH),
            windowed: asset_server.load(CHARACTER_SELECTION_WINDOWED_PATH),
            windowed_over: asset_server.load(CHARACTER_SELECTION_WINDOWED_OVER_PATH),
            music_toggle_off: asset_server.load(CHARACTER_SELECTION_MUSIC_TOGGLE_OFF_PATH),
            music_toggle_on: asset_server.load(CHARACTER_SELECTION_MUSIC_TOGGLE_ON_PATH),
            red_button: asset_server.load(CHARACTER_SELECTION_RED_BUTTON_PATH),
            red_button_over: asset_server.load(CHARACTER_SELECTION_RED_BUTTON_OVER_PATH),
            blue_button: asset_server.load(CHARACTER_SELECTION_BLUE_BUTTON_PATH),
            blue_button_over: asset_server.load(CHARACTER_SELECTION_BLUE_BUTTON_OVER_PATH),
            delete_backdrop: asset_server.load(CHARACTER_SELECTION_DELETE_BACKDROP_PATH),
            delete_window: asset_server.load(CHARACTER_SELECTION_DELETE_WINDOW_PATH),
            cancel_normal: asset_server.load(CHARACTER_SELECTION_CANCEL_NORMAL_PATH),
            rotate_left: asset_server.load(CHARACTER_SELECTION_ROTATE_LEFT_PATH),
            rotate_left_over: asset_server.load(CHARACTER_SELECTION_ROTATE_LEFT_OVER_PATH),
            rotate_right: asset_server.load(CHARACTER_SELECTION_ROTATE_RIGHT_PATH),
            rotate_right_over: asset_server.load(CHARACTER_SELECTION_ROTATE_RIGHT_OVER_PATH),
            disk_back: asset_server.load(CHARACTER_SELECTION_DISK_BACK_PATH),
            disk_front: asset_server.load(CHARACTER_SELECTION_DISK_FRONT_PATH),
            button_sounds: CHARACTER_SELECTION_BUTTON_SOUND_PATHS
                .map(|path| asset_server.load(path)),
            delete_yes_sound: asset_server.load(CHARACTER_SELECTION_DELETE_YES_SOUND_PATH),
            delete_no_sound: asset_server.load(CHARACTER_SELECTION_DELETE_NO_SOUND_PATH),
            music: asset_server.load(CHARACTER_SELECTION_MUSIC_PATH),
            chalet_font: asset_server.load(CHARACTER_SELECTION_CHALET_FONT_PATH),
            jeffe_font: asset_server.load(CHARACTER_SELECTION_JEFFE_FONT_PATH),
        }
    }

    pub(super) fn status(&self, asset_server: &AssetServer) -> CharacterSelectionAssetStatus {
        let mut loading = false;
        macro_rules! observe {
            ($path:expr, $handle:expr) => {
                match asset_server.load_state($handle.id()) {
                    LoadState::Failed(_) => {
                        return CharacterSelectionAssetStatus::Failed {
                            path: $path.to_owned(),
                        };
                    }
                    LoadState::Loaded => {}
                    LoadState::NotLoaded | LoadState::Loading => loading = true,
                }
            };
        }

        observe!(CHARACTER_SELECTION_CHROME_PATH, self.chrome);
        for (location_index, backgrounds) in self.backgrounds.iter().enumerate() {
            let location = CharacterLocationBackground::ALL[location_index];
            for (frame, handle) in backgrounds.iter().enumerate() {
                let path = location.asset_path(frame + 1);
                observe!(&path, handle);
            }
        }
        for (path, handle) in [
            (CHARACTER_SELECTION_SLOT_EMPTY_PATH, &self.slot_empty),
            (CHARACTER_SELECTION_SLOT_NORMAL_PATH, &self.slot_normal),
            (CHARACTER_SELECTION_SLOT_OVER_PATH, &self.slot_over),
            (CHARACTER_SELECTION_SLOT_LOCKED_PATH, &self.slot_locked),
            (CHARACTER_SELECTION_LOCK_PATH, &self.lock),
            (CHARACTER_SELECTION_ENTER_PATH, &self.enter),
            (CHARACTER_SELECTION_ENTER_OVER_PATH, &self.enter_over),
            (CHARACTER_SELECTION_FULLSCREEN_PATH, &self.fullscreen),
            (
                CHARACTER_SELECTION_FULLSCREEN_OVER_PATH,
                &self.fullscreen_over,
            ),
            (CHARACTER_SELECTION_WINDOWED_PATH, &self.windowed),
            (CHARACTER_SELECTION_WINDOWED_OVER_PATH, &self.windowed_over),
            (
                CHARACTER_SELECTION_MUSIC_TOGGLE_OFF_PATH,
                &self.music_toggle_off,
            ),
            (
                CHARACTER_SELECTION_MUSIC_TOGGLE_ON_PATH,
                &self.music_toggle_on,
            ),
            (CHARACTER_SELECTION_RED_BUTTON_PATH, &self.red_button),
            (
                CHARACTER_SELECTION_RED_BUTTON_OVER_PATH,
                &self.red_button_over,
            ),
            (CHARACTER_SELECTION_BLUE_BUTTON_PATH, &self.blue_button),
            (
                CHARACTER_SELECTION_BLUE_BUTTON_OVER_PATH,
                &self.blue_button_over,
            ),
            (
                CHARACTER_SELECTION_DELETE_BACKDROP_PATH,
                &self.delete_backdrop,
            ),
            (CHARACTER_SELECTION_DELETE_WINDOW_PATH, &self.delete_window),
            (CHARACTER_SELECTION_CANCEL_NORMAL_PATH, &self.cancel_normal),
            (CHARACTER_SELECTION_ROTATE_LEFT_PATH, &self.rotate_left),
            (
                CHARACTER_SELECTION_ROTATE_LEFT_OVER_PATH,
                &self.rotate_left_over,
            ),
            (CHARACTER_SELECTION_ROTATE_RIGHT_PATH, &self.rotate_right),
            (
                CHARACTER_SELECTION_ROTATE_RIGHT_OVER_PATH,
                &self.rotate_right_over,
            ),
            (CHARACTER_SELECTION_DISK_BACK_PATH, &self.disk_back),
            (CHARACTER_SELECTION_DISK_FRONT_PATH, &self.disk_front),
        ] {
            observe!(path, handle);
        }
        for (path, handle) in CHARACTER_SELECTION_BUTTON_SOUND_PATHS
            .iter()
            .copied()
            .zip(self.button_sounds.iter())
        {
            observe!(path, handle);
        }
        for (path, handle) in [
            (
                CHARACTER_SELECTION_DELETE_YES_SOUND_PATH,
                &self.delete_yes_sound,
            ),
            (
                CHARACTER_SELECTION_DELETE_NO_SOUND_PATH,
                &self.delete_no_sound,
            ),
            (CHARACTER_SELECTION_MUSIC_PATH, &self.music),
        ] {
            observe!(path, handle);
        }
        observe!(CHARACTER_SELECTION_CHALET_FONT_PATH, self.chalet_font);
        observe!(CHARACTER_SELECTION_JEFFE_FONT_PATH, self.jeffe_font);

        if loading {
            CharacterSelectionAssetStatus::Loading
        } else {
            CharacterSelectionAssetStatus::Ready
        }
    }

    pub(super) fn background(&self, location: CharacterLocationBackground, frame: usize) -> Handle<Image> {
        self.backgrounds[location as usize][frame].clone()
    }
}

#[derive(Component)]
pub struct NativeCharacterSelectionRoot;

#[derive(Component)]
pub(super) struct SelectionChrome;

#[derive(Component)]
pub(super) struct SelectionAvatarGroup;

#[derive(Component)]
pub(super) struct SelectionPreviewArea;

#[derive(Component)]
pub(super) struct SelectionPreviewLoading;

#[derive(Component)]
pub(super) struct SelectionRotateLeft;

#[derive(Component)]
pub(super) struct SelectionRotateRight;

#[derive(Component)]
pub(super) struct SelectionPanel;

#[derive(Component)]
pub(super) struct SelectionQuit;

#[derive(Component)]
pub(super) struct SelectionFullscreen;

#[derive(Component)]
pub(super) struct SelectionSlotName(pub(super) usize);

#[derive(Component)]
pub(super) struct SelectionSlotLevel(pub(super) usize);

#[derive(Component)]
pub(super) struct SelectionSlotLocation(pub(super) usize);

#[derive(Component)]
pub(super) struct SelectionSlotEmptyLabel(pub(super) usize);

#[derive(Component)]
pub(super) struct SelectionSlotPortraitArea;

#[derive(Component)]
pub(super) struct SelectionSlotDiskBack(pub(super) usize);

#[derive(Component)]
pub(super) struct SelectionSlotDiskFront(pub(super) usize);

#[derive(Component)]
pub(super) struct SelectionSlotLock(pub(super) usize);

#[derive(Component)]
pub(super) struct SelectionAvatarName;

#[derive(Component)]
pub(super) struct SelectionEnter;

#[derive(Component)]
pub(super) struct SelectionEnterText;

#[derive(Component)]
pub(super) struct SelectionCreate;

#[derive(Component)]
pub(super) struct SelectionCreateText;

#[derive(Component)]
pub(super) struct SelectionDelete;

#[derive(Component)]
pub(super) struct SelectionDeleteText;

#[derive(Component)]
pub(super) struct SelectionMusicToggle;

#[derive(Component)]
pub(super) struct SelectionQuitText;

#[derive(Component)]
pub(super) struct SelectionMusic;

#[derive(Component)]
pub(super) struct SelectionDeleteModal;

#[derive(Component)]
pub(super) struct SelectionBaseCamera;

#[derive(Component)]
pub(super) struct SelectionModalCamera;

#[derive(Component)]
pub(super) struct SelectionPortraitOverlayCamera;

#[derive(Component)]
pub(super) struct SelectionPortraitOverlayRoot;

#[derive(Component)]
pub(super) struct SelectionDeletePanel;

#[derive(Component)]
pub(super) struct SelectionDeleteFieldText;

#[derive(Component)]
pub(super) struct SelectionDeleteCancel;

#[derive(Component)]
pub(super) struct SelectionDeleteCancelText;

#[derive(Component)]
pub(super) struct SelectionDeleteConfirm;

#[derive(Component)]
pub(super) struct SelectionDeleteConfirmText;

pub(super) fn character_selection_text_font(
    assets: &CharacterSelectionAssets,
    style: CharacterSelectionTextStyle,
) -> (TextFont, LineHeight) {
    let spec = style.spec();
    let font = match spec.font_role {
        CharacterSelectionFontRole::Jeffe => assets.jeffe_font.clone(),
        CharacterSelectionFontRole::Chalet => assets.chalet_font.clone(),
    };
    (
        TextFont {
            font: (font).into(),
            font_size: (spec.font_size).into(),
            ..default()
        },
        LineHeight::Px(spec.line_height),
    )
}

pub(super) fn character_selection_text_color(style: CharacterSelectionTextStyle) -> TextColor {
    let [red, green, blue, alpha] = style.spec().normal_color;
    TextColor(Color::srgba(red, green, blue, alpha))
}

pub(super) fn character_selection_text_node(
    rect: LegacySelectionRect,
    style: CharacterSelectionTextStyle,
) -> Node {
    let spec = style.spec();
    let [left, right, top, bottom] = spec.padding;
    let mut node = absolute_node(LegacySelectionRect::new(
        rect.x,
        rect.y + spec.y_offset,
        rect.width,
        rect.height,
    ));
    node.align_items = AlignItems::Center;
    node.justify_content = match spec.anchor {
        CharacterSelectionTextAnchor::MiddleLeft => JustifyContent::Start,
        CharacterSelectionTextAnchor::MiddleCenter => JustifyContent::Center,
    };
    node.padding = UiRect::new(px(left), px(right), px(top), px(bottom));
    node
}

pub(super) fn bind_character_selection_background_and_slots(
    model: Res<CharacterSelectionUiModel>,
    assets: Res<CharacterSelectionAssets>,
    mut frames: Query<(&SelectionBackgroundFrame, &mut ImageNode), Without<SelectionSlotButton>>,
    mut slot_images: Query<
        (&SelectionSlotButton, &Interaction, &mut ImageNode),
        Without<SelectionBackgroundFrame>,
    >,
) {
    let background = model.selected_background();
    for (frame, mut image) in &mut frames {
        image.image = assets.background(background, frame.0);
    }
    for (button, interaction, mut image) in &mut slot_images {
        let slot = &model.slots[button.0];
        image.image = match slot {
            CharacterSlotUi::Empty => assets.slot_empty.clone(),
            CharacterSlotUi::SubscriptionLocked
            | CharacterSlotUi::SubscriptionLockedOccupied(_) => assets.slot_locked.clone(),
            CharacterSlotUi::Occupied(_) => {
                if model.selected_slot == Some(button.0)
                    || model.hovered_slot == Some(button.0)
                    || *interaction == Interaction::Hovered
                {
                    assets.slot_over.clone()
                } else {
                    assets.slot_normal.clone()
                }
            }
        };
    }
}
