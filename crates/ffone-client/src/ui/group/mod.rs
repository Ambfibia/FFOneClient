//! Passive Retrobution group HUD.
//!
//! `CnGuiGroup_info::OnOwnGUI` paints remote PC rows from the top-left at
//! `y = 100 + visible_index * 50`, skipping the local PC while retaining the
//! original protocol group size for the cooperative-NPC row.  This module
//! preserves that slightly asymmetric layout as a presentation-only Bevy
//! resource/plugin boundary.  It does not own a camera and does not emit
//! gameplay actions.

use bevy::{
    prelude::*,
    text::{LineBreak, LineHeight},
};

use crate::{
    localization::{LocalizationSet, LocalizedText, localized_tabledata_npc_name},
    mission_ui::{MissionUiModel, gameplay_chrome_visible},
};

pub const GROUP_UI_REFERENCE_WIDTH: f32 = 1_280.0;
pub const GROUP_UI_REFERENCE_HEIGHT: f32 = 720.0;
pub const GROUP_PROTOCOL_MAX_PC_MEMBERS: usize = 4;
pub const GROUP_PROTOCOL_MAX_NPC_MEMBERS: usize = 5;
pub const GROUP_LEGACY_VISIBLE_NPC_MEMBERS: usize = 1;

pub const GROUP_INFO_PATH: &str = "ui/en/gameplay/group/group_info.png";
pub const GROUP_FREECHAT_ICON_PATH: &str = "ui/en/gameplay/group/group_freechat_icon.png";
pub const GROUP_NANO_HP_FRAME_PATH: &str = "ui/en/gameplay/group/group_nano_hpframe.png";
pub const GROUP_NPC_CO_OP_PATH: &str = "ui/en/gameplay/group/npc_co_op.png";
pub const GROUP_HP_BAR_PATH: &str = "ui/en/gameplay/group/hp_bar.png";
pub const GROUP_UI_FONT_PATH: &str = "fonts/chaletbook-regular.ttf";
pub const GROUP_UI_PRIMARY_CONTAINER_BYTES: usize = 7_000_415;
pub const GROUP_UI_PRIMARY_CONTAINER_SHA256: &str =
    "59788201962b6a1737b114486c361fe74eef69f507d1d125ca3171377eec602f";
pub const GROUP_UI_GAME_HUD_PATH_ID: i64 = 1_352;
pub const GROUP_UI_COMPONENT_PATH_ID: i64 = 1_563;
pub const GROUP_UI_SCRIPT_PATH_ID: i64 = 1_135;
pub const GROUP_UI_SKIN_PATH_ID: i64 = 1_372;
pub const GROUP_UI_FONT_PATH_ID: i64 = 1_018;
pub const GROUP_UI_FONT_BYTES: usize = 96_832;
pub const GROUP_UI_FONT_SHA256: &str =
    "6383bd9f81e56d61139884d8e42cb7b2146a11dde4efde55c8bff1e4c2c0bbe8";
pub const GROUP_UI_FONT_SIZE: f32 = 12.0;
pub const GROUP_UI_FONT_LINE_HEIGHT: f32 = 12.071_999_55;
pub const GROUP_UI_TEXT_CONTENT_OFFSET_Y: f32 = 0.0;

pub const GROUP_ROW_TOP: f32 = 100.0;
pub const GROUP_ROW_STEP: f32 = 50.0;
pub const GROUP_PC_ROW_LEFT: f32 = 2.0;
pub const GROUP_NPC_ROW_LEFT: f32 = 5.0;
pub const GROUP_UI_Z_INDEX: i32 = 11;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Component)]
pub enum GroupUiTextStyle {
    HudLabel,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroupUiTextStyleSpec {
    pub source_game_object_path_id: i64,
    pub source_component_path_id: i64,
    pub source_script_path_id: i64,
    pub source_skin_path_id: i64,
    pub source_style: &'static str,
    pub source_font_path_id: i64,
    pub font_path: &'static str,
    pub font_size: f32,
    pub line_height: f32,
    pub justify: Justify,
    pub linebreak: LineBreak,
    pub padding: [f32; 4],
    pub text_color: [f32; 4],
    pub y_offset: f32,
}

impl GroupUiTextStyle {
    #[must_use]
    pub const fn spec(self) -> GroupUiTextStyleSpec {
        match self {
            Self::HudLabel => GroupUiTextStyleSpec {
                source_game_object_path_id: GROUP_UI_GAME_HUD_PATH_ID,
                source_component_path_id: GROUP_UI_COMPONENT_PATH_ID,
                source_script_path_id: GROUP_UI_SCRIPT_PATH_ID,
                source_skin_path_id: GROUP_UI_SKIN_PATH_ID,
                source_style: "label",
                source_font_path_id: GROUP_UI_FONT_PATH_ID,
                font_path: GROUP_UI_FONT_PATH,
                font_size: GROUP_UI_FONT_SIZE,
                line_height: GROUP_UI_FONT_LINE_HEIGHT,
                justify: Justify::Left,
                linebreak: LineBreak::WordBoundary,
                padding: [0.0; 4],
                text_color: [0.9, 0.9, 0.9, 1.0],
                y_offset: GROUP_UI_TEXT_CONTENT_OFFSET_Y,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroupUiRect {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}

impl GroupUiRect {
    pub const fn new(left: f32, top: f32, width: f32, height: f32) -> Self {
        Self {
            left,
            top,
            width,
            height,
        }
    }

    fn node(self) -> Node {
        Node {
            position_type: PositionType::Absolute,
            left: px(self.left),
            top: px(self.top),
            width: px(self.width),
            height: px(self.height),
            ..default()
        }
    }
}

// Retrobution passes `WindowRect` to `GUI.BeginGroup` and then reuses the
// same rect for `GUI.DrawTexture`, so the texture retains the serialized
// local +2 X offset inside the already-offset outer group.
pub const GROUP_PC_BACKGROUND_RECT: GroupUiRect = GroupUiRect::new(2.0, 0.0, 198.0, 48.0);
pub const GROUP_PC_NAME_RECT: GroupUiRect = GroupUiRect::new(10.0, -4.0, 180.0, 20.0);
pub const GROUP_PC_LEVEL_RECT: GroupUiRect = GroupUiRect::new(40.0, 25.0, 25.0, 20.0);
pub const GROUP_PC_HP_RECT: GroupUiRect = GroupUiRect::new(7.0, 17.0, 186.0, 4.0);
pub const GROUP_NANO_NAME_RECT: GroupUiRect = GroupUiRect::new(60.0, 15.0, 96.0, 20.0);
pub const GROUP_NANO_FRAME_RECT: GroupUiRect = GroupUiRect::new(60.0, 35.0, 111.0, 6.0);
pub const GROUP_NANO_FILL_RECT: GroupUiRect = GroupUiRect::new(61.0, 36.0, 109.0, 4.0);
pub const GROUP_NANO_SKILL_ICON_RECT: GroupUiRect = GroupUiRect::new(172.0, 21.0, 26.0, 26.0);
pub const GROUP_FREECHAT_ICON_RECT: GroupUiRect = GroupUiRect::new(3.0, 5.0, 18.0, 14.0);
// `Window2Rect` is likewise reused inside its own BeginGroup.
pub const GROUP_NPC_BACKGROUND_RECT: GroupUiRect = GroupUiRect::new(5.0, 0.0, 192.0, 28.0);
pub const GROUP_NPC_NAME_RECT: GroupUiRect = GroupUiRect::new(10.0, -4.0, 180.0, 20.0);
pub const GROUP_NPC_HP_RECT: GroupUiRect = GroupUiRect::new(10.0, 17.0, 180.0, 4.0);

#[must_use]
pub const fn group_pc_row_rect(visible_index: usize) -> GroupUiRect {
    GroupUiRect::new(
        GROUP_PC_ROW_LEFT,
        GROUP_ROW_TOP + visible_index as f32 * GROUP_ROW_STEP,
        GROUP_PC_BACKGROUND_RECT.width,
        GROUP_PC_BACKGROUND_RECT.height,
    )
}

#[must_use]
pub const fn group_npc_row_rect(legacy_group_size: usize) -> GroupUiRect {
    GroupUiRect::new(
        GROUP_NPC_ROW_LEFT,
        GROUP_ROW_TOP + legacy_group_size as f32 * GROUP_ROW_STEP,
        GROUP_NPC_BACKGROUND_RECT.width,
        GROUP_NPC_BACKGROUND_RECT.height,
    )
}

/// Fail-closed progress fraction used by PC HP, cooperative-NPC HP and Nano
/// stamina. Retrobution caps the upper edge at one; native presentation also
/// closes the legacy divide-by-zero/negative edge at zero.
#[must_use]
pub fn group_ui_fraction(current: i32, maximum: i32) -> f32 {
    if current <= 0 || maximum <= 0 {
        return 0.0;
    }
    ((f64::from(current) / f64::from(maximum)).clamp(0.0, 1.0)) as f32
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GroupNanoUi {
    pub name: String,
    pub stamina: i32,
    pub max_stamina: i32,
    /// Optional semantic runtime path for the member Nano's skill icon.
    pub skill_icon_path: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GroupPcMemberUi {
    /// Runtime PC ID used by ALL GROUP chat and buddy block checks.
    pub pc_id: i32,
    pub pc_uid: i64,
    /// Lossless `sPCGroupMemberInfo.iMapType` retained for gameplay gates.
    pub map_type: i32,
    /// Lossless `sPCGroupMemberInfo.iMapNum` retained for gameplay gates.
    pub map_number: i32,
    /// Raw server-axis/centiunit `sPCGroupMemberInfo.iX/iY/iZ`.
    pub position: [i32; 3],
    pub first_name: String,
    pub last_name: String,
    pub level: i16,
    pub hp: i32,
    pub max_hp: i32,
    pub free_chat: bool,
    pub nano: Option<GroupNanoUi>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct GroupNpcMemberUi {
    pub npc_type: i32,
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
}

/// Presentation input. The protocol permits four PC and five NPC members;
/// view construction clamps both collections to that proven closure.
#[derive(Clone, Debug, Default, PartialEq, Eq, Resource)]
pub struct GroupUiModel {
    pub local_pc_uid: Option<i64>,
    pub pc_members: Vec<GroupPcMemberUi>,
    pub npc_members: Vec<GroupNpcMemberUi>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GroupNanoView {
    pub name: String,
    pub stamina_fraction: f32,
    pub skill_icon_path: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GroupPcRowView {
    pub pc_uid: i64,
    pub name: String,
    pub level: String,
    pub hp_fraction: f32,
    pub free_chat: bool,
    pub nano: Option<GroupNanoView>,
    pub rect: GroupUiRect,
}

#[derive(Clone, Debug, PartialEq)]
pub struct GroupNpcRowView {
    pub npc_type: i32,
    pub name: String,
    pub hp_fraction: f32,
    pub rect: GroupUiRect,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GroupUiView {
    pub visible: bool,
    /// The bounded `m_iGroupSize`, including the skipped local user. This is
    /// intentionally not the count of rendered PC rows.
    pub legacy_group_size: usize,
    pub pc_rows: Vec<GroupPcRowView>,
    pub npc_row: Option<GroupNpcRowView>,
}

#[must_use]
pub fn group_ui_view(model: &GroupUiModel) -> GroupUiView {
    let bounded_pc = model
        .pc_members
        .iter()
        .take(GROUP_PROTOCOL_MAX_PC_MEMBERS)
        .collect::<Vec<_>>();
    let legacy_group_size = bounded_pc.len();
    let pc_rows = bounded_pc
        .into_iter()
        .filter(|member| Some(member.pc_uid) != model.local_pc_uid)
        .enumerate()
        .map(|(visible_index, member)| GroupPcRowView {
            pc_uid: member.pc_uid,
            name: group_member_name(member),
            level: member.level.to_string(),
            hp_fraction: group_ui_fraction(member.hp, member.max_hp),
            free_chat: member.free_chat,
            nano: member.nano.as_ref().map(|nano| GroupNanoView {
                name: nano.name.clone(),
                stamina_fraction: group_ui_fraction(nano.stamina, nano.max_stamina),
                skill_icon_path: nano.skill_icon_path.clone(),
            }),
            rect: group_pc_row_rect(visible_index),
        })
        .collect::<Vec<_>>();
    let npc_row = model
        .npc_members
        .iter()
        .take(GROUP_PROTOCOL_MAX_NPC_MEMBERS)
        .next()
        .map(|npc| GroupNpcRowView {
            npc_type: npc.npc_type,
            name: npc.name.clone(),
            hp_fraction: group_ui_fraction(npc.hp, npc.max_hp),
            rect: group_npc_row_rect(legacy_group_size),
        });

    GroupUiView {
        visible: !pc_rows.is_empty() || npc_row.is_some(),
        legacy_group_size,
        pc_rows,
        npc_row,
    }
}

fn group_member_name(member: &GroupPcMemberUi) -> String {
    let first = member.first_name.trim();
    let last = member.last_name.trim();
    let name = match (first.is_empty(), last.is_empty()) {
        (false, false) => format!("{first} {last}"),
        (false, true) => first.to_owned(),
        (true, false) => last.to_owned(),
        (true, true) => String::new(),
    };
    if name.is_empty() {
        format!("Player {}", member.pc_uid)
    } else {
        name
    }
}

#[derive(Clone, Resource)]
struct GroupUiAssets {
    background: Handle<Image>,
    nano_frame: Handle<Image>,
    npc_background: Handle<Image>,
    hp_bar: Handle<Image>,
    font: Handle<Font>,
}

impl GroupUiAssets {
    fn load(asset_server: &AssetServer) -> Self {
        Self {
            background: asset_server.load(GROUP_INFO_PATH),
            nano_frame: asset_server.load(GROUP_NANO_HP_FRAME_PATH),
            npc_background: asset_server.load(GROUP_NPC_CO_OP_PATH),
            hp_bar: asset_server.load(GROUP_HP_BAR_PATH),
            font: asset_server.load(GROUP_UI_FONT_PATH),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Component)]
struct GroupUiRoot;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Component)]
enum GroupUiElement {
    PcRow(usize),
    PcName(usize),
    PcLevel(usize),
    PcHp(usize),
    NanoName(usize),
    NanoFrame(usize),
    NanoFill(usize),
    NanoSkillIcon(usize),
    NpcRow,
    NpcName,
    NpcHp,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, SystemSet)]
pub enum GroupUiSet {
    Bind,
}

pub struct GroupUiPlugin;

impl Plugin for GroupUiPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<crate::damage_bar::DamageBarPlugin>() {
            app.add_plugins(crate::damage_bar::DamageBarPlugin);
        }
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<GroupUiModel>()
            .add_systems(crate::ui_startup::NativeGameplayUiStartup, spawn_group_ui)
            .add_systems(
                Update,
                (bind_group_ui
                    .in_set(GroupUiSet::Bind)
                    .before(LocalizationSet::Apply))
                .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}

fn spawn_group_ui(mut commands: Commands, asset_server: Res<AssetServer>) {
    let assets = GroupUiAssets::load(&asset_server);
    commands.insert_resource(assets.clone());
    commands
        .spawn((
            GroupUiRoot,
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                top: px(0),
                width: px(GROUP_UI_REFERENCE_WIDTH),
                height: px(GROUP_UI_REFERENCE_HEIGHT),
                ..default()
            },
            Visibility::Hidden,
            Pickable::IGNORE,
            ZIndex(GROUP_UI_Z_INDEX),
        ))
        .with_children(|root| {
            for slot in 0..GROUP_PROTOCOL_MAX_PC_MEMBERS {
                spawn_pc_row(root, slot, &assets);
            }
            spawn_npc_row(root, &assets);
        });
}

fn spawn_pc_row(parent: &mut ChildSpawnerCommands, slot: usize, assets: &GroupUiAssets) {
    let mut row_node = group_pc_row_rect(slot).node();
    row_node.display = Display::None;
    parent
        .spawn((GroupUiElement::PcRow(slot), row_node, Pickable::IGNORE))
        .with_children(|row| {
            spawn_image(
                row,
                GROUP_PC_BACKGROUND_RECT,
                assets.background.clone(),
                None,
            );
            spawn_text(
                row,
                GROUP_PC_NAME_RECT,
                assets.font.clone(),
                GroupUiElement::PcName(slot),
            );
            spawn_text(
                row,
                GROUP_PC_LEVEL_RECT,
                assets.font.clone(),
                GroupUiElement::PcLevel(slot),
            );
            spawn_image(
                row,
                GROUP_PC_HP_RECT,
                assets.hp_bar.clone(),
                Some(GroupUiElement::PcHp(slot)),
            );
            // `FreeChatTexture` and `FreeChatRect` are serialized on
            // `CnGuiGroup_info`, but clean Retrobution never references them
            // from `OnOwnGUI`/`DoWindow`. Loading or drawing that icon here
            // would therefore add a UI element absent from the authority.
            spawn_text(
                row,
                GROUP_NANO_NAME_RECT,
                assets.font.clone(),
                GroupUiElement::NanoName(slot),
            );
            spawn_image(
                row,
                GROUP_NANO_FRAME_RECT,
                assets.nano_frame.clone(),
                Some(GroupUiElement::NanoFrame(slot)),
            );
            spawn_image(
                row,
                GROUP_NANO_FILL_RECT,
                assets.hp_bar.clone(),
                Some(GroupUiElement::NanoFill(slot)),
            );
            spawn_image(
                row,
                GROUP_NANO_SKILL_ICON_RECT,
                Handle::default(),
                Some(GroupUiElement::NanoSkillIcon(slot)),
            );
        });
}

fn spawn_npc_row(parent: &mut ChildSpawnerCommands, assets: &GroupUiAssets) {
    let mut node = group_npc_row_rect(0).node();
    node.display = Display::None;
    parent
        .spawn((GroupUiElement::NpcRow, node, Pickable::IGNORE))
        .with_children(|row| {
            spawn_image(
                row,
                GROUP_NPC_BACKGROUND_RECT,
                assets.npc_background.clone(),
                None,
            );
            spawn_text(
                row,
                GROUP_NPC_NAME_RECT,
                assets.font.clone(),
                GroupUiElement::NpcName,
            );
            spawn_image(
                row,
                GROUP_NPC_HP_RECT,
                assets.hp_bar.clone(),
                Some(GroupUiElement::NpcHp),
            );
        });
}

fn spawn_image(
    parent: &mut ChildSpawnerCommands,
    rect: GroupUiRect,
    image: Handle<Image>,
    marker: Option<GroupUiElement>,
) {
    if let Some(marker @ (GroupUiElement::PcHp(_) | GroupUiElement::NpcHp)) = marker {
        crate::damage_bar::spawn_damage_bar(
            parent,
            rect.node(),
            ImageNode {
                image,
                image_mode: NodeImageMode::Stretch,
                ..default()
            },
            marker,
        );
        return;
    }
    let mut entity = parent.spawn((
        rect.node(),
        ImageNode {
            image,
            image_mode: NodeImageMode::Stretch,
            ..default()
        },
        Pickable::IGNORE,
    ));
    if let Some(marker) = marker {
        entity.insert(marker);
    }
}

fn spawn_text(
    parent: &mut ChildSpawnerCommands,
    rect: GroupUiRect,
    font: Handle<Font>,
    marker: GroupUiElement,
) {
    parent.spawn((
        Node {
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Center,
            overflow: Overflow::clip(),
            ..rect.node()
        },
        Text::new(""),
        group_text_localized(marker, ""),
        GroupUiTextStyle::HudLabel,
        (
            TextFont {
                font: (font).into(),
                font_size: (GROUP_UI_FONT_SIZE).into(),
                ..default()
            },
            LineHeight::Px(GROUP_UI_FONT_LINE_HEIGHT),
        ),
        TextColor(Color::srgb(0.9, 0.9, 0.9)),
        TextLayout::new(Justify::Left, LineBreak::WordBoundary),
        marker,
        Pickable::IGNORE,
    ));
}

fn bind_group_ui(
    model: Res<GroupUiModel>,
    mission_ui: Option<Res<MissionUiModel>>,
    asset_server: Res<AssetServer>,
    mut roots: Query<&mut Visibility, With<GroupUiRoot>>,
    mut elements: Query<(
        &GroupUiElement,
        &mut Node,
        Option<&mut ImageNode>,
        Option<&mut LocalizedText>,
        Option<&mut crate::damage_bar::DamageBarOwner>,
    )>,
) {
    let view = group_ui_view(&model);
    for mut visibility in &mut roots {
        *visibility = if gameplay_chrome_visible(view.visible, mission_ui.as_deref()) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    for (element, mut node, image, localized, owner) in &mut elements {
        if let Some(mut owner) = owner {
            owner.0 = match *element {
                GroupUiElement::PcHp(slot) => {
                    view.pc_rows.get(slot).map_or(0, |row| row.pc_uid as u64)
                }
                GroupUiElement::NpcHp => view.npc_row.as_ref().map_or(0, |row| row.npc_type as u64),
                _ => 0,
            };
        }
        match *element {
            GroupUiElement::PcRow(slot) => {
                let row = view.pc_rows.get(slot);
                node.display = if row.is_some() {
                    Display::Flex
                } else {
                    Display::None
                };
                let rect = row
                    .map(|row| row.rect)
                    .unwrap_or_else(|| group_pc_row_rect(slot));
                node.left = px(rect.left);
                node.top = px(rect.top);
            }
            GroupUiElement::PcName(slot) => {
                set_text(
                    localized,
                    *element,
                    view.pc_rows.get(slot).map(|row| row.name.as_str()),
                );
            }
            GroupUiElement::PcLevel(slot) => {
                set_text(
                    localized,
                    *element,
                    view.pc_rows.get(slot).map(|row| row.level.as_str()),
                );
            }
            GroupUiElement::PcHp(slot) => {
                node.width = px(GROUP_PC_HP_RECT.width
                    * view.pc_rows.get(slot).map_or(0.0, |row| row.hp_fraction));
            }
            GroupUiElement::NanoName(slot) => {
                let nano = view.pc_rows.get(slot).and_then(|row| row.nano.as_ref());
                node.display = if nano.is_some() {
                    Display::Flex
                } else {
                    Display::None
                };
                set_text(localized, *element, nano.map(|nano| nano.name.as_str()));
            }
            GroupUiElement::NanoFrame(slot) => {
                node.display = nano_display(&view, slot);
            }
            GroupUiElement::NanoFill(slot) => {
                let nano = view.pc_rows.get(slot).and_then(|row| row.nano.as_ref());
                node.display = if nano.is_some() {
                    Display::Flex
                } else {
                    Display::None
                };
                node.width =
                    px(GROUP_NANO_FILL_RECT.width * nano.map_or(0.0, |nano| nano.stamina_fraction));
            }
            GroupUiElement::NanoSkillIcon(slot) => {
                let path = view
                    .pc_rows
                    .get(slot)
                    .and_then(|row| row.nano.as_ref())
                    .and_then(|nano| nano.skill_icon_path.as_deref());
                node.display = if path.is_some() {
                    Display::Flex
                } else {
                    Display::None
                };
                if let Some(mut image) = image {
                    image.image = path
                        .map(|path| asset_server.load(path.to_owned()))
                        .unwrap_or_default();
                }
            }
            GroupUiElement::NpcRow => {
                node.display = if view.npc_row.is_some() {
                    Display::Flex
                } else {
                    Display::None
                };
                let rect = view
                    .npc_row
                    .as_ref()
                    .map(|row| row.rect)
                    .unwrap_or_else(|| group_npc_row_rect(view.legacy_group_size));
                node.left = px(rect.left);
                node.top = px(rect.top);
            }
            GroupUiElement::NpcName => {
                if let Some(mut localized) = localized {
                    *localized = view.npc_row.as_ref().map_or_else(
                        || group_text_localized(*element, ""),
                        |row| {
                            if row.npc_type > 0 {
                                localized_tabledata_npc_name(row.npc_type, &row.name)
                            } else {
                                group_text_localized(*element, &row.name)
                            }
                        },
                    );
                }
            }
            GroupUiElement::NpcHp => {
                node.width = px(GROUP_NPC_HP_RECT.width
                    * view.npc_row.as_ref().map_or(0.0, |row| row.hp_fraction));
            }
        }
    }
}

fn nano_display(view: &GroupUiView, slot: usize) -> Display {
    if view
        .pc_rows
        .get(slot)
        .and_then(|row| row.nano.as_ref())
        .is_some()
    {
        Display::Flex
    } else {
        Display::None
    }
}

fn group_text_localized(element: GroupUiElement, value: impl Into<String>) -> LocalizedText {
    let value = value.into();
    match element {
        GroupUiElement::PcLevel(_) => {
            LocalizedText::new("ui.group.level", "{level}").with_arg("level", value)
        }
        GroupUiElement::PcName(_) | GroupUiElement::NanoName(_) | GroupUiElement::NpcName => {
            LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", value)
        }
        _ => LocalizedText::new("ui.content.passthrough", "{text}").with_arg("text", value),
    }
}

fn set_text(
    mut localized: Option<Mut<LocalizedText>>,
    element: GroupUiElement,
    value: Option<&str>,
) {
    let Some(ref mut localized) = localized else {
        return;
    };
    let value = value.unwrap_or_default();
    **localized = group_text_localized(element, value);
}

#[cfg(test)]
mod tests;
