//! NPC barker and player free-chat speech bubbles.

use super::assets::GameplayUiAssets;
use super::combat_target::GameplayHudNpcQuery;
use super::model::GameplayUiModel;
use crate::{
    avatar_action::LegacyAvatarActionState,
    entity_lifecycle::{
        NetworkNpcAppearance0104, NetworkNpcBarkerEvent0104, NetworkNpcBarkerEventQueue0104,
    },
    legacy_npc_nano_animation::LegacyNanoStandRandomStream,
    localization::{
        LocalizedText, localized_tabledata_mission_barker, localized_tabledata_npc_barker,
        localized_tabledata_npc_greeting, localized_tabledata_npc_skill_barker,
    },
    movement::LegacyOrbitCamera,
    tutorial_actors::TutorialActor,
    tutorial_mission_content::{GameplayNpcUiDefinition, TutorialMissionContent},
    world::{
        AuthoredColliderWorldBounds, AuthoredTriMeshCollider,
        authored_collider_blocks_segment_with_bounds,
    },
};
use bevy::{
    prelude::*,
    sprite::{BorderRect, SliceScaleMode, TextureSlicer},
    text::LineHeight,
    ui::widget::NodeImageMode,
};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

pub const NPC_BARKER_BOX_PATH: &str = "ui/en/gameplay/speech/barker_box.png";
pub const NPC_BARKER_TAIL_PATH: &str = "ui/en/gameplay/speech/barker_tail.png";
pub const NPC_QUEST_BOX_PATH: &str = "ui/en/gameplay/speech/quick-chat-box.png";
pub const NPC_QUEST_TAIL_PATH: &str = "ui/en/gameplay/speech/quick-chat-tail.png";
pub const NPC_BARKER_BOX_BYTES: u64 = 538;
pub const NPC_BARKER_BOX_SHA256: &str =
    "6a7bf01d0a6dd32c36bd0bb1a9b0f5734b156e7697d4812ca523d7caf553d2b0";
pub const NPC_BARKER_TAIL_BYTES: u64 = 460;
pub const NPC_BARKER_TAIL_SHA256: &str =
    "824a145fceae46ac6e176cc074d120be3cf284ab99f73a7d31dc4da0a34eac38";
pub const PLAYER_FREECHAT_BOX_PATH: &str = "ui/en/gameplay/speech/freechat_box.png";
pub const PLAYER_FREECHAT_TAIL_PATH: &str = "ui/en/gameplay/speech/freechat_tail.png";
pub const PLAYER_FREECHAT_BOX_BYTES: u64 = 566;
pub const PLAYER_FREECHAT_BOX_SHA256: &str =
    "da0815f27b239d415fb82c59d1d3a97bab61f19214b0afff7f96a30cb8aeaf15";
pub const PLAYER_FREECHAT_TAIL_BYTES: u64 = 269;
pub const PLAYER_FREECHAT_TAIL_SHA256: &str =
    "e919c96166aa41707ad3f767a2e88ea43b9e98bbeb751cc4e0e77701cd5bf2ea";
pub const NPC_BARKER_PERIOD_SECONDS: f64 = 15.0;
pub const NPC_BARKER_SHARED_COOLDOWN_SECONDS: f64 = 600.0;
pub const NPC_BARKER_DISTANCE: f32 = 10.0;
// Reading an existing line is independent of the radius that starts random chatter.
pub const NPC_BUBBLE_DISTANCE: f32 = 30.0;
// Keep an already nearby speaker eligible across small movement at the limit.
const NPC_BARKER_HIDE_DISTANCE: f32 = NPC_BUBBLE_DISTANCE + 1.0;

pub(super) fn npc_bubble_in_range(player: Vec3, npc: Vec3, was_in_range: bool) -> bool {
    let limit = if was_in_range {
        NPC_BARKER_HIDE_DISTANCE
    } else {
        NPC_BUBBLE_DISTANCE
    };
    player.is_finite() && npc.is_finite() && player.distance_squared(npc) <= limit * limit
}
pub const NPC_BARKER_CHANCE_PERCENT: u32 = 10;
/// Clean `PrintName.DrawBarker` inherits the resident world-label IMGUI pass
/// depth, so every ordinary window is allowed to cover it. Keep this root
/// below the default Bevy UI root depth while still rendering it over 3D.
pub const NPC_BARKER_GLOBAL_Z_INDEX: i32 = -1_000;
pub const NPC_BARKER_MIN_WIDTH: f32 = 100.0;
pub const NPC_BARKER_MAX_WIDTH: f32 = 300.0;
pub const NPC_BARKER_MIN_LIFETIME_SECONDS: f64 = 7.0;
pub(super) const NPC_BARKER_HEIGHT_LIFETIME_DIVISOR: f64 = 5.0;
// `bbBox`/`fbBox` also use centered JEFFE___14 and therefore share the same
// fixed-raster horizontal/vertical adapter as the chat controls.
pub(super) const NPC_BARKER_JEFFE_FONT_SIZE: f32 = 14.0;
pub(super) const NPC_BARKER_JEFFE_LINE_HEIGHT: f32 = 13.71;
pub(super) const NPC_BARKER_JEFFE_VERTICAL_SCALE: f32 = 0.7;
pub(super) const NPC_BARKER_TAIL_SIZE: Vec2 = Vec2::new(5.0, 20.0);

#[derive(Clone, Debug, PartialEq)]
pub(super) struct NpcBarkerBubbleLine {
    pub(super) localized: LocalizedText,
    pub(super) kind: NpcBubbleKind,
    pub(super) started_at: f64,
    pub(super) max_lifetime: f64,
    pub(super) serial: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum NpcBubbleKind {
    Ordinary,
    Quest,
}

fn npc_bubble_images(
    assets: &GameplayUiAssets,
    kind: NpcBubbleKind,
) -> (&Handle<Image>, &Handle<Image>) {
    match kind {
        NpcBubbleKind::Ordinary => (&assets.npc_barker_box, &assets.npc_barker_tail),
        NpcBubbleKind::Quest => (&assets.npc_quest_box, &assets.npc_quest_tail),
    }
}

#[derive(Clone, Debug)]
pub(super) struct NpcBarkerActorState {
    pub(super) npc_type: i32,
    /// Clean `fBarkerTime`: initialized to Random.Range(0, period), then set
    /// to global `Time.time` after each periodic test.
    pub(super) f_barker_time: f64,
    pub(super) lines: VecDeque<NpcBarkerBubbleLine>,
}

#[derive(Clone, Debug)]
pub(super) struct PendingNpcGreeting {
    pub(super) owner: Entity,
    pub(super) npc_type: i32,
    pub(super) localized: LocalizedText,
    pub(super) interrupt: bool,
}

/// One accepted NPC speech edge, before the independent balloon display gate.
/// The history consumer resolves both semantic texts in the selected text locale.
#[derive(Clone, Debug, Message)]
pub struct NpcChatEvent {
    pub npc_type: i32,
    pub message: LocalizedText,
}

/// Native owner of clean `PrintName.ChatList` state for normal-world NPCs.
/// Gameplay interaction code only requests a greeting; this resource keeps
/// timing, FIFO ordering, random barkers and world-space UI ownership here.
#[derive(Debug, Resource)]
pub struct NpcBarkerBubbleRuntime {
    pub(super) actors: BTreeMap<Entity, NpcBarkerActorState>,
    pub(super) pending_greetings: VecDeque<PendingNpcGreeting>,
    pub(super) next_serial: u64,
    /// Canonical source text, shared across NPC types, rows and text locales.
    pub(super) background_last_spoken: BTreeMap<String, f64>,
}

impl Default for NpcBarkerBubbleRuntime {
    fn default() -> Self {
        Self {
            actors: BTreeMap::new(),
            pending_greetings: VecDeque::new(),
            next_serial: 1,
            background_last_spoken: BTreeMap::new(),
        }
    }
}

impl NpcBarkerBubbleRuntime {
    /// Mandatory quest speech replaces older bubbles and bypasses ambient cooldown.
    pub fn request_quest_dialogue(
        &mut self,
        owner: Entity,
        npc_type: i32,
        localized: LocalizedText,
    ) {
        self.pending_greetings.push_back(PendingNpcGreeting {
            owner,
            npc_type,
            localized,
            interrupt: true,
        });
    }

    pub(super) fn publish_quest_speech(
        &mut self,
        owner: Entity,
        localized: LocalizedText,
        now: f64,
        balloon_visible: bool,
        chat: &mut MessageWriter<NpcChatEvent>,
    ) {
        self.publish_priority_speech(owner, localized, now, balloon_visible, chat, NpcBubbleKind::Quest);
    }

    fn publish_priority_speech(
        &mut self,
        owner: Entity,
        localized: LocalizedText,
        now: f64,
        balloon_visible: bool,
        chat: &mut MessageWriter<NpcChatEvent>,
        kind: NpcBubbleKind,
    ) {
        if !chat_string_is_visible(&localized.fallback) {
            return;
        }
        if let Some(actor) = self.actors.get_mut(&owner) {
            actor.lines.clear();
        }
        self.publish_speech_with_kind(
            owner,
            localized,
            now,
            balloon_visible,
            chat,
            kind,
        );
    }

    pub(super) fn publish_background_speech(
        &mut self,
        owner: Entity,
        localized: LocalizedText,
        now: f64,
        balloon_visible: bool,
        chat: &mut MessageWriter<NpcChatEvent>,
    ) {
        if !chat_string_is_visible(&localized.fallback) || !self.actors.contains_key(&owner) {
            return;
        }
        self.background_last_spoken
            .retain(|_, spoken| now - *spoken < NPC_BARKER_SHARED_COOLDOWN_SECONDS);
        // Different TableData rows can contain the same line. Source text is
        // stable across EN/RU switching; normalize only incidental whitespace.
        let key = localized
            .fallback
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if self.background_last_spoken.contains_key(&key) {
            return;
        }
        self.background_last_spoken.insert(key, now);
        self.publish_speech(owner, localized, now, balloon_visible, chat);
    }

    pub(super) fn publish_speech(
        &mut self,
        owner: Entity,
        localized: LocalizedText,
        now: f64,
        balloon_visible: bool,
        chat: &mut MessageWriter<NpcChatEvent>,
    ) {
        self.publish_speech_with_kind(
            owner,
            localized,
            now,
            balloon_visible,
            chat,
            NpcBubbleKind::Ordinary,
        );
    }

    fn publish_speech_with_kind(
        &mut self,
        owner: Entity,
        localized: LocalizedText,
        now: f64,
        balloon_visible: bool,
        chat: &mut MessageWriter<NpcChatEvent>,
        kind: NpcBubbleKind,
    ) {
        if !chat_string_is_visible(&localized.fallback) {
            return;
        }
        let Some(actor) = self.actors.get(&owner) else {
            return;
        };
        chat.write(NpcChatEvent {
            npc_type: actor.npc_type,
            message: localized.clone(),
        });
        if balloon_visible {
            self.enqueue_with_kind(owner, localized, now, kind);
        }
    }

    /// Queues the exact clean `NpcGreetingBubble` line. Empty and one-space
    /// strings are rejected by `PrintName.SetChatBubble` before UI creation.
    pub fn request_greeting(&mut self, owner: Entity, definition: &GameplayNpcUiDefinition) {
        if !chat_string_is_visible(&definition.greeting) {
            return;
        }
        self.pending_greetings.push_back(PendingNpcGreeting {
            owner,
            npc_type: definition.npc_type,
            localized: localized_tabledata_npc_greeting(
                definition.greeting_string_id,
                &definition.greeting,
            ),
            interrupt: false,
        });
    }

    pub(super) fn enqueue(&mut self, owner: Entity, localized: LocalizedText, now: f64) {
        self.enqueue_with_kind(owner, localized, now, NpcBubbleKind::Ordinary);
    }

    fn enqueue_with_kind(
        &mut self,
        owner: Entity,
        localized: LocalizedText,
        now: f64,
        kind: NpcBubbleKind,
    ) {
        if !chat_string_is_visible(&localized.fallback) {
            return;
        }
        let serial = self.next_serial;
        self.next_serial = self.next_serial.wrapping_add(1).max(1);
        if let Some(actor) = self.actors.get_mut(&owner) {
            actor.lines.push_back(NpcBarkerBubbleLine {
                localized,
                kind,
                started_at: now,
                max_lifetime: NPC_BARKER_MIN_LIFETIME_SECONDS,
                serial,
            });
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct PlayerFreeChatBubbleLine {
    pub(super) localized: LocalizedText,
    pub(super) started_at: f64,
    pub(super) max_lifetime: f64,
    pub(super) serial: u64,
}

#[derive(Clone, Debug)]
pub(super) struct PendingPlayerFreeChatBubble {
    pub(super) owner: Entity,
    pub(super) message: String,
}

/// Clean player `PrintName.ChatList` owner for ordinary FreeChat. Unlike an
/// NPC barker FIFO, every accepted player message clears and replaces that
/// actor's current bubble.
#[derive(Debug, Resource)]
pub struct PlayerFreeChatBubbleRuntime {
    pub(super) actors: BTreeMap<Entity, PlayerFreeChatBubbleLine>,
    pub(super) pending: VecDeque<PendingPlayerFreeChatBubble>,
    pub(super) next_serial: u64,
}

impl Default for PlayerFreeChatBubbleRuntime {
    fn default() -> Self {
        Self {
            actors: BTreeMap::new(),
            pending: VecDeque::new(),
            next_serial: 1,
        }
    }
}

impl PlayerFreeChatBubbleRuntime {
    /// Queues raw player-authored text through the required passthrough
    /// localization template. Empty and exactly-one-space messages follow the
    /// source `PrintName.SetChatBubble` rejection gate.
    pub fn request_message(&mut self, owner: Entity, message: impl Into<String>) {
        let message = message.into();
        if !chat_string_is_visible(&message) {
            return;
        }
        self.pending
            .push_back(PendingPlayerFreeChatBubble { owner, message });
    }

    #[must_use]
    pub fn pending_len(&self) -> usize {
        self.pending.len()
    }

    #[must_use]
    pub fn has_pending_message(&self, owner: Entity, message: &str) -> bool {
        self.pending
            .iter()
            .any(|pending| pending.owner == owner && pending.message == message)
    }

    pub(super) fn replace(&mut self, owner: Entity, message: String, now: f64) {
        let serial = self.next_serial;
        self.next_serial = self.next_serial.wrapping_add(1).max(1);
        self.actors.insert(
            owner,
            PlayerFreeChatBubbleLine {
                localized: LocalizedText::new("ui.content.passthrough", "{text}")
                    .with_arg("text", message),
                started_at: now,
                max_lifetime: NPC_BARKER_MIN_LIFETIME_SECONDS,
                serial,
            },
        );
    }
}

pub(super) fn chat_string_is_visible(value: &str) -> bool {
    !value.is_empty() && value != " "
}

pub(super) fn barker_attempt_succeeds(percent_draw: u32, distance: f32) -> bool {
    percent_draw < NPC_BARKER_CHANCE_PERCENT
        && distance.is_finite()
        && distance < NPC_BARKER_DISTANCE
}

pub(super) fn barker_lifetime(height: f32) -> f64 {
    NPC_BARKER_MIN_LIFETIME_SECONDS.max(f64::from(height) / NPC_BARKER_HEIGHT_LIFETIME_DIVISOR)
}

/// `Camera::world_to_viewport` returns logical window coordinates, while
/// Bevy 0.17 stores `ComputedNode::size` in physical pixels. Bubble placement
/// and the clean lifetime formula both operate in the logical IMGUI space.
pub(super) fn speech_bubble_logical_size(computed: &ComputedNode) -> Vec2 {
    let inverse_scale_factor = computed.inverse_scale_factor;
    if inverse_scale_factor.is_finite() && inverse_scale_factor > 0.0 {
        computed.size() * inverse_scale_factor
    } else {
        computed.size()
    }
}

pub(super) fn speech_bubble_top_left(viewport: Vec2, logical_size: Vec2, y_offset: f32) -> Vec2 {
    Vec2::new(
        viewport.x - logical_size.x.max(NPC_BARKER_MIN_WIDTH) * 0.5,
        viewport.y - logical_size.y - y_offset,
    )
}

pub(super) fn advance_autonomous_npc_barker(
    now: f64,
    player_position: Vec3,
    npc_position: Vec3,
    barker: &crate::tutorial_mission_content::GameplayNpcBarkerDefinition,
    actor: &mut NpcBarkerActorState,
    random: &mut LegacyNanoStandRandomStream,
) -> Option<LocalizedText> {
    if now - actor.f_barker_time <= NPC_BARKER_PERIOD_SECONDS {
        return None;
    }
    actor.f_barker_time = now;

    // Clean draws the chance before checking distance, then draws the field
    // index only after both gates pass.
    let percent_draw = random.next_index(100) as u32;
    let distance = player_position.distance(npc_position);
    if !barker_attempt_succeeds(percent_draw, distance) {
        return None;
    }
    let field_index = random.next_index(4);
    Some(localized_tabledata_npc_barker(
        barker.string_id,
        field_index,
        &barker.lines[field_index],
    ))
}

#[derive(Component)]
pub(super) struct NpcBarkerBubbleLayer;
#[derive(Component)]
pub(super) struct NpcBarkerBubbleUi {
    pub(super) owner: Entity,
    pub(super) text_entity: Entity,
    pub(super) tail_entity: Entity,
    pub(super) serial: u64,
    pub(super) in_range: bool,
}
#[derive(Component)]
pub(super) struct PlayerFreeChatBubbleUi {
    pub(super) owner: Entity,
    pub(super) text_entity: Entity,
    pub(super) serial: u64,
}

pub(super) fn advance_npc_barker_bubbles(
    time: Res<Time>,
    model: Res<GameplayUiModel>,
    content: Option<Res<TutorialMissionContent>>,
    network_npcs: Query<(Entity, &GlobalTransform, &NetworkNpcAppearance0104)>,
    tutorial_npcs: Query<(Entity, &GlobalTransform, &TutorialActor)>,
    players: Query<&GlobalTransform, With<LegacyAvatarActionState>>,
    mut network_barkers: Option<ResMut<NetworkNpcBarkerEventQueue0104>>,
    mut random: ResMut<LegacyNanoStandRandomStream>,
    mut runtime: ResMut<NpcBarkerBubbleRuntime>,
    mut chat: MessageWriter<NpcChatEvent>,
) {
    let now = time.elapsed_secs_f64();
    let Some(content) = content.as_deref() else {
        runtime.actors.clear();
        runtime.pending_greetings.clear();
        if let Some(events) = network_barkers.as_mut() {
            events.clear();
        }
        return;
    };
    let mut live = network_npcs
        .iter()
        .filter_map(|(entity, transform, appearance)| {
            let definition = content.gameplay_npc(appearance.0.npc_type)?;
            Some((
                entity,
                appearance.0.npc_type,
                transform.translation(),
                definition.barker.clone(),
                Some(appearance.0.npc_id),
            ))
        })
        .collect::<Vec<_>>();
    live.extend(
        tutorial_npcs
            .iter()
            .filter(|(_, _, actor)| actor.is_alive())
            .filter_map(|(entity, transform, actor)| {
                let definition = content.gameplay_npc(actor.npc_type)?;
                Some((
                    entity,
                    actor.npc_type,
                    transform.translation(),
                    definition.barker.clone(),
                    None,
                ))
            }),
    );
    let live_entities = live
        .iter()
        .map(|(entity, _, _, _, _)| *entity)
        .collect::<BTreeSet<_>>();

    for (owner, npc_type, _, _, _) in &live {
        let must_reset = runtime
            .actors
            .get(owner)
            .is_none_or(|actor| actor.npc_type != *npc_type);
        if must_reset {
            let f_barker_time = f64::from(random.next_unit_f32()) * NPC_BARKER_PERIOD_SECONDS;
            runtime.actors.insert(
                *owner,
                NpcBarkerActorState {
                    npc_type: *npc_type,
                    f_barker_time,
                    lines: VecDeque::new(),
                },
            );
        }
    }
    runtime
        .actors
        .retain(|owner, _| live_entities.contains(owner));

    while let Some(pending) = runtime.pending_greetings.pop_front() {
        if runtime
            .actors
            .get(&pending.owner)
            .is_some_and(|actor| actor.npc_type == pending.npc_type)
        {
            if pending.interrupt {
                runtime.publish_quest_speech(
                    pending.owner,
                    pending.localized,
                    now,
                    model.balloon_chat_visible,
                    &mut chat,
                );
            } else {
                runtime.publish_speech(
                    pending.owner,
                    pending.localized,
                    now,
                    model.balloon_chat_visible,
                    &mut chat,
                );
            }
        }
    }

    if let Some(events) = network_barkers.as_mut() {
        for event in events.take_all() {
            let interrupt = matches!(&event, NetworkNpcBarkerEvent0104::Mission(_));
            let resolved = match event {
                NetworkNpcBarkerEvent0104::Mission(packet) => live
                    .iter()
                    .find(|(_, _, _, _, network_id)| *network_id == Some(packet.npc_id))
                    .and_then(|(owner, _, _, _, _)| {
                        content
                            .gameplay_mission_name_string(packet.mission_string_id)
                            .map(|fallback| {
                                (
                                    *owner,
                                    localized_tabledata_mission_barker(
                                        packet.mission_string_id,
                                        fallback,
                                    ),
                                )
                            })
                    }),
                NetworkNpcBarkerEvent0104::SkillReady(signal) => live
                    .iter()
                    .find(|(_, _, _, _, network_id)| *network_id == Some(signal.npc_id))
                    .and_then(|(owner, npc_type, _, _, _)| {
                        let definition = match signal.kind {
                            ffone_protocol::NpcSkillSignalKind0104::Ready => {
                                signal.skill_id.and_then(|skill_id| {
                                    content.gameplay_npc_skill_barker(*npc_type, skill_id)
                                })
                            }
                            ffone_protocol::NpcSkillSignalKind0104::CorruptionReady => {
                                content.gameplay_npc_corruption_barker(*npc_type)
                            }
                            _ => None,
                        }?;
                        Some((
                            *owner,
                            localized_tabledata_npc_skill_barker(
                                definition.string_id,
                                &definition.text,
                            ),
                        ))
                    }),
            };
            if let Some((owner, localized)) = resolved {
                if interrupt {
                    // BARKER uses mission strings for in-world shouts. Only an
                    // explicit dialogue request is yellow.
                    runtime.publish_priority_speech(
                        owner,
                        localized,
                        now,
                        model.balloon_chat_visible,
                        &mut chat,
                        NpcBubbleKind::Ordinary,
                    );
                } else {
                    runtime.publish_speech(
                        owner,
                        localized,
                        now,
                        model.balloon_chat_visible,
                        &mut chat,
                    );
                }
            }
        }
    }

    // PrintName removes only ChatList[0] in one LateUpdate. Retain that FIFO
    // behavior when several click/barker lines were queued close together.
    for actor in runtime.actors.values_mut() {
        if actor
            .lines
            .front()
            .is_some_and(|line| now - line.started_at > line.max_lifetime)
        {
            actor.lines.pop_front();
        }
    }

    let Some(player_position) = players.iter().next().map(GlobalTransform::translation) else {
        return;
    };
    for (owner, _, npc_position, barker, _) in live {
        let Some(barker) = barker else {
            continue;
        };
        let localized = runtime.actors.get_mut(&owner).and_then(|actor| {
            advance_autonomous_npc_barker(
                now,
                player_position,
                npc_position,
                &barker,
                actor,
                &mut random,
            )
        });
        if let Some(localized) = localized {
            runtime.publish_background_speech(
                owner,
                localized,
                now,
                model.balloon_chat_visible,
                &mut chat,
            );
        }
    }
}

pub(super) fn spawn_npc_barker_bubble(
    commands: &mut Commands,
    layer: Entity,
    assets: &GameplayUiAssets,
    owner: Entity,
    line: &NpcBarkerBubbleLine,
) {
    let text_entity = commands
        .spawn((
            Node {
                max_width: px(NPC_BARKER_MAX_WIDTH - 20.0),
                ..default()
            },
            Text::new(""),
            (
                TextFont {
                    font: (assets.jeffe_font.clone()).into(),
                    font_size: (NPC_BARKER_JEFFE_FONT_SIZE).into(),
                    ..default()
                },
                LineHeight::Px(NPC_BARKER_JEFFE_LINE_HEIGHT),
            ),
            UiTransform::from_scale(Vec2::new(1.0, NPC_BARKER_JEFFE_VERTICAL_SCALE)),
            TextColor(Color::BLACK),
            TextLayout::new(Justify::Center, LineBreak::WordBoundary),
            line.localized.clone(),
        ))
        .id();
    let tail = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                top: percent(100),
                width: px(NPC_BARKER_TAIL_SIZE.x),
                height: px(NPC_BARKER_TAIL_SIZE.y),
                margin: UiRect {
                    left: px(-4),
                    ..default()
                },
                ..default()
            },
            UiTransform::default(),
            ImageNode {
                image: npc_bubble_images(assets, line.kind).1.clone(),
                image_mode: NodeImageMode::Stretch,
                ..default()
            },
        ))
        .id();
    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                min_width: px(NPC_BARKER_MIN_WIDTH),
                max_width: px(NPC_BARKER_MAX_WIDTH),
                min_height: px(19),
                display: Display::None,
                padding: UiRect::all(px(10)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ImageNode {
                image: npc_bubble_images(assets, line.kind).0.clone(),
                visual_box: bevy::ui::VisualBox::BorderBox,
                image_mode: NodeImageMode::Sliced(TextureSlicer {
                    border: BorderRect::all(6.0),
                    center_scale_mode: SliceScaleMode::Stretch,
                    sides_scale_mode: SliceScaleMode::Stretch,
                    max_corner_scale: 1.0,
                }),
                ..default()
            },
            Pickable::IGNORE,
            NpcBarkerBubbleUi {
                owner,
                text_entity,
                tail_entity: tail,
                serial: line.serial,
                in_range: false,
            },
        ))
        .id();
    commands.entity(root).add_child(text_entity).add_child(tail);
    commands.entity(layer).add_child(root);
}

pub(super) fn bind_npc_barker_bubbles(
    mut commands: Commands,
    model: Res<GameplayUiModel>,
    assets: Res<GameplayUiAssets>,
    mut runtime: ResMut<NpcBarkerBubbleRuntime>,
    layer: Query<Entity, With<NpcBarkerBubbleLayer>>,
    players: Query<(Entity, &LegacyAvatarActionState, &GlobalTransform)>,
    cameras: Query<(&Camera, &GlobalTransform, &LegacyOrbitCamera), With<Camera3d>>,
    targets: GameplayHudNpcQuery,
    colliders: Query<(
        Entity,
        &GlobalTransform,
        &AuthoredTriMeshCollider,
        &AuthoredColliderWorldBounds,
    )>,
    parents: Query<&ChildOf>,
    mut bubbles: Query<(
        Entity,
        &ComputedNode,
        &mut Node,
        &mut UiTransform,
        &mut ImageNode,
        &mut NpcBarkerBubbleUi,
    )>,
) {
    let projection = players.iter().find_map(|(entity, _, player)| {
        cameras
            .iter()
            .find(|(_, _, orbit)| orbit.target == entity)
            .map(|(camera, transform, _)| (camera, transform, player.translation()))
    });
    let mut rendered = BTreeSet::new();
    for (bubble_entity, computed, mut node, mut transform, mut image, mut bubble) in &mut bubbles {
        rendered.insert(bubble.owner);
        let Some(line) = runtime
            .actors
            .get_mut(&bubble.owner)
            .and_then(|actor| actor.lines.front_mut())
        else {
            commands.entity(bubble_entity).despawn();
            continue;
        };
        let logical_size = speech_bubble_logical_size(computed);
        if logical_size.y > 0.0 {
            line.max_lifetime = barker_lifetime(logical_size.y);
        }
        if bubble.serial != line.serial {
            commands
                .entity(bubble.text_entity)
                .insert(line.localized.clone());
            let (box_image, tail_image) = npc_bubble_images(&assets, line.kind);
            image.image = box_image.clone();
            commands.entity(bubble.tail_entity).insert(ImageNode {
                image: tail_image.clone(),
                image_mode: NodeImageMode::Stretch,
                ..default()
            });
            bubble.serial = line.serial;
        }
        node.display = Display::None;
        if !model.balloon_chat_visible {
            continue;
        }
        let Some((camera, camera_transform, player_position)) = projection else {
            continue;
        };
        let Some((npc_transform, npc)) = targets.get(bubble.owner) else {
            bubble.in_range = false;
            continue;
        };
        bubble.in_range = npc_bubble_in_range(
            player_position,
            npc_transform.translation(),
            bubble.in_range,
        );
        if !bubble.in_range {
            continue;
        }
        let world_position = npc.projected_world_position(npc_transform.translation(), 0.8);
        let Ok(viewport) = camera.world_to_viewport(camera_transform, world_position) else {
            continue;
        };
        if camera
            .logical_viewport_rect()
            .is_none_or(|rect| !rect.contains(viewport))
        {
            continue;
        }
        if colliders.iter().any(|(entity, global, collider, bounds)| {
            if !authored_collider_blocks_segment_with_bounds(
                collider,
                global,
                bounds,
                camera_transform.translation(),
                world_position,
            ) {
                return false;
            }
            // A speaker's own model collider must not occlude its head.
            let mut ancestor = entity;
            loop {
                if ancestor == bubble.owner {
                    return false;
                }
                let Ok(parent) = parents.get(ancestor) else {
                    break;
                };
                ancestor = parent.parent();
            }
            true
        }) {
            continue;
        }
        // Clean offsets the projected PrintName bubble another 15 pixels up
        // only while `bRDrawName` is true. NPC names are disabled by clean's
        // defaults, so the ordinary bubble bottom remains at the projection.
        let name_offset = if model.npc_names_visible { 15.0 } else { 0.0 };
        let top_left = speech_bubble_top_left(viewport, logical_size, name_offset);
        node.left = px(top_left.x);
        node.top = px(top_left.y);
        // `ScaleAroundRect(ChatRect)` scales the box and its tail around the
        // rectangle centre. A Bevy UI transform on this root has that pivot.
        let ui_scale = if model.ui_scale.is_finite() && model.ui_scale > 0.0 {
            model.ui_scale
        } else {
            1.0
        };
        transform.scale = Vec2::splat(ui_scale);
        node.display = Display::Flex;
    }

    let Ok(layer) = layer.single() else {
        return;
    };
    for (&owner, actor) in &runtime.actors {
        if rendered.contains(&owner) {
            continue;
        }
        let Some(line) = actor.lines.front() else {
            continue;
        };
        spawn_npc_barker_bubble(&mut commands, layer, &assets, owner, line);
    }
}

pub(super) fn advance_player_freechat_bubbles(
    time: Res<Time>,
    model: Res<GameplayUiModel>,
    live_transforms: Query<(), With<GlobalTransform>>,
    mut runtime: ResMut<PlayerFreeChatBubbleRuntime>,
) {
    let now = time.elapsed_secs_f64();
    runtime.actors.retain(|owner, line| {
        live_transforms.get(*owner).is_ok() && now - line.started_at <= line.max_lifetime
    });
    while let Some(pending) = runtime.pending.pop_front() {
        if model.visible && model.balloon_chat_visible && live_transforms.get(pending.owner).is_ok()
        {
            // A player owns no `NpcMoveController`, so clean
            // `SetChatBubble` clears ChatList before inserting this row.
            runtime.replace(pending.owner, pending.message, now);
        }
    }
}

pub(super) fn spawn_player_freechat_bubble(
    commands: &mut Commands,
    layer: Entity,
    assets: &GameplayUiAssets,
    owner: Entity,
    line: &PlayerFreeChatBubbleLine,
) {
    let text_entity = commands
        .spawn((
            Node {
                max_width: px(NPC_BARKER_MAX_WIDTH - 20.0),
                ..default()
            },
            Text::new(""),
            (
                TextFont {
                    font: (assets.jeffe_font.clone()).into(),
                    font_size: (NPC_BARKER_JEFFE_FONT_SIZE).into(),
                    ..default()
                },
                LineHeight::Px(NPC_BARKER_JEFFE_LINE_HEIGHT),
            ),
            UiTransform::from_scale(Vec2::new(1.0, NPC_BARKER_JEFFE_VERTICAL_SCALE)),
            TextColor(Color::BLACK),
            TextLayout::new(Justify::Center, LineBreak::WordBoundary),
            line.localized.clone(),
        ))
        .id();
    let tail = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                top: percent(100),
                width: px(NPC_BARKER_TAIL_SIZE.x),
                height: px(NPC_BARKER_TAIL_SIZE.y),
                margin: UiRect {
                    left: px(-4),
                    ..default()
                },
                ..default()
            },
            UiTransform::default(),
            ImageNode {
                image: assets.player_freechat_tail.clone(),
                image_mode: NodeImageMode::Stretch,
                ..default()
            },
        ))
        .id();
    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                min_width: px(NPC_BARKER_MIN_WIDTH),
                max_width: px(NPC_BARKER_MAX_WIDTH),
                min_height: px(19),
                padding: UiRect::all(px(10)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ImageNode {
                image: assets.player_freechat_box.clone(),
                visual_box: bevy::ui::VisualBox::BorderBox,
                image_mode: NodeImageMode::Sliced(TextureSlicer {
                    border: BorderRect::all(6.0),
                    center_scale_mode: SliceScaleMode::Stretch,
                    sides_scale_mode: SliceScaleMode::Stretch,
                    max_corner_scale: 1.0,
                }),
                ..default()
            },
            Pickable::IGNORE,
            PlayerFreeChatBubbleUi {
                owner,
                text_entity,
                serial: line.serial,
            },
        ))
        .id();
    commands.entity(root).add_child(text_entity).add_child(tail);
    commands.entity(layer).add_child(root);
}

pub(super) fn bind_player_freechat_bubbles(
    mut commands: Commands,
    model: Res<GameplayUiModel>,
    assets: Res<GameplayUiAssets>,
    mut runtime: ResMut<PlayerFreeChatBubbleRuntime>,
    layer: Query<Entity, With<NpcBarkerBubbleLayer>>,
    players: Query<(Entity, &LegacyAvatarActionState)>,
    cameras: Query<(&Camera, &GlobalTransform, &LegacyOrbitCamera), With<Camera3d>>,
    targets: GameplayHudNpcQuery,
    mut bubbles: Query<(
        Entity,
        &ComputedNode,
        &mut Node,
        &mut UiTransform,
        &mut PlayerFreeChatBubbleUi,
    )>,
) {
    let projection = players.iter().find_map(|(entity, _)| {
        cameras
            .iter()
            .find(|(_, _, orbit)| orbit.target == entity)
            .map(|(camera, transform, _)| (camera, transform))
    });
    let mut rendered = BTreeSet::new();
    for (bubble_entity, computed, mut node, mut transform, mut bubble) in &mut bubbles {
        rendered.insert(bubble.owner);
        let Some(line) = runtime.actors.get_mut(&bubble.owner) else {
            commands.entity(bubble_entity).despawn();
            continue;
        };
        let logical_size = speech_bubble_logical_size(computed);
        if logical_size.y > 0.0 {
            line.max_lifetime = barker_lifetime(logical_size.y);
        }
        if bubble.serial != line.serial {
            commands
                .entity(bubble.text_entity)
                .insert(line.localized.clone());
            bubble.serial = line.serial;
        }
        node.display = Display::None;
        if !model.visible || !model.balloon_chat_visible {
            continue;
        }
        let Some((camera, camera_transform)) = projection else {
            continue;
        };
        let Some(world_position) = targets.projected_world_position(bubble.owner, 0.9) else {
            continue;
        };
        let Ok(viewport) = camera.world_to_viewport(camera_transform, world_position) else {
            continue;
        };
        let top_left = speech_bubble_top_left(viewport, logical_size, 0.0);
        node.left = px(top_left.x);
        node.top = px(top_left.y);
        let ui_scale = if model.ui_scale.is_finite() && model.ui_scale > 0.0 {
            model.ui_scale
        } else {
            1.0
        };
        transform.scale = Vec2::splat(ui_scale);
        node.display = Display::Flex;
    }

    let Ok(layer) = layer.single() else {
        return;
    };
    for (&owner, line) in &runtime.actors {
        if !rendered.contains(&owner) {
            spawn_player_freechat_bubble(&mut commands, layer, &assets, owner, line);
        }
    }
}
