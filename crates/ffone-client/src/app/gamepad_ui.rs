//! Controller focus feeds the same Interaction components as pointer input.
use super::*;
use bevy::input::{
    ButtonState,
    gamepad::GamepadButton,
    keyboard::{Key, KeyboardInput},
};
use bevy::ui::{CalculatedClip, ComputedStackIndex};
use bevy::window::PrimaryWindow;
use ffone_client::gameplay_ui::GameplayControllerMenuInput;
use ffone_client::ui::shared::controller::*;

#[derive(bevy::ecs::system::SystemParam)]
pub(super) struct PadUiContext<'w, 's> {
    input: ResMut<'w, ControllerUiInput>,
    tutorial: Res<'w, TutorialSession>,
    launcher: Res<'w, LauncherUiModel>,
    hints: Query<
        'w,
        's,
        (
            Has<ControllerUiClose>,
            Has<ControllerUiDefault>,
            Has<ControllerUiRepeat>,
            Option<&'static ControllerUiTab>,
        ),
    >,
    ignored: Query<'w, 's, (), With<ControllerUiIgnore>>,
    scopes: Query<'w, 's, &'static ControllerUiScope>,
    boundaries: Query<'w, 's, (), With<ControllerUiBoundary>>,
}

#[derive(Resource, Default)]
pub(super) struct PadUiFocus {
    pub(super) entity: Option<Entity>,
    direction: Vec2,
    next_repeat: f64,
    applied: Option<(Entity, Interaction)>,
    applied_tick: Option<bevy::ecs::change_detection::Tick>,
    held_entity: Option<Entity>,
    next_confirm_repeat: f64,
    pub(super) escape_injected: bool,
}

/// Remove our synthetic pointer state without creating another click edge.
/// Bevy's pointer focus runs between this pass and navigate_gamepad_ui.
pub(super) fn prepare_gamepad_ui(
    focus: Res<PadUiFocus>,
    mut interactions: Query<&mut Interaction>,
) {
    if let Some((entity, previous)) = focus.applied
        && let Ok(mut interaction) = interactions.get_mut(entity)
        && *interaction == previous
    {
        *interaction.bypass_change_detection() = Interaction::None;
    }
}

#[derive(Component)]
pub(super) struct PadUiHighlight(Entity);

pub(super) fn highlight_gamepad_ui(
    mut commands: Commands,
    focus: Res<PadUiFocus>,
    buttons: Query<(), With<Button>>,
    mut highlighted: Query<(Entity, &PadUiHighlight, &mut Interaction)>,
) {
    for (entity, original, mut interaction) in &mut highlighted {
        if Some(entity) == focus.entity {
            continue;
        }
        let mut target = commands.entity(entity);
        target.remove::<PadUiHighlight>();
        commands.entity(original.0).despawn();
        if *interaction == Interaction::None {
            interaction.set_changed();
        }
    }
    if let Some(entity) = focus.entity
        && !highlighted.contains(entity)
        && buttons.contains(entity)
    {
        // A final child paints above item/Nano images which would otherwise
        // cover an inset outline drawn on their parent slot.
        let overlay = commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(0),
                    top: px(0),
                    right: px(0),
                    bottom: px(0),
                    border: UiRect::all(px(3)),
                    ..default()
                },
                BorderColor::all(Color::srgb(0.15, 0.8, 1.0)),
                BackgroundColor(Color::srgba(0.15, 0.8, 1.0, 0.16)),
                ZIndex(i32::MAX),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
                ChildOf(entity),
            ))
            .id();
        commands.entity(entity).insert(PadUiHighlight(overlay));
    }
}

#[derive(Clone, Copy)]
struct Candidate {
    entity: Entity,
    center: Vec2,
    owner: Entity,
    z: i32,
    stack: u32,
    scope: Option<ControllerUiScope>,
    priority: i8,
    repeat: bool,
    tab: Option<u8>,
}

fn choose_direction(candidates: &[Candidate], current: Entity, direction: Vec2) -> Option<Entity> {
    let origin = candidates.iter().find(|c| c.entity == current)?.center;
    candidates
        .iter()
        .filter_map(|c| {
            let delta = c.center - origin;
            let forward = delta.dot(direction);
            if forward <= 1.0 {
                return None;
            }
            let side = delta.perp_dot(direction).abs();
            Some((c.entity, forward + side * 3.0))
        })
        .min_by(|a, b| {
            a.1.total_cmp(&b.1)
                .then_with(|| a.0.index().cmp(&b.0.index()))
        })
        .map(|(entity, _)| entity)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn navigate_gamepad_ui(
    time: Res<Time>,
    state: Res<State<ClientState>>,
    mut pad: ResMut<GamepadActionState>,
    models: GameplayModalModels,
    mission: Res<MissionUiModel>,
    mut menu: ResMut<GameplayControllerMenuInput>,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    mut key_events: MessageWriter<KeyboardInput>,
    windows: Query<Entity, With<PrimaryWindow>>,
    mut focus: ResMut<PadUiFocus>,
    hierarchy: Query<(
        &Node,
        Option<&ChildOf>,
        Option<&GlobalZIndex>,
        Option<&Visibility>,
    )>,
    buttons: Query<
        (
            Entity,
            &ComputedNode,
            &UiGlobalTransform,
            &ComputedStackIndex,
            Option<&CalculatedClip>,
            Option<&Pickable>,
        ),
        With<Button>,
    >,
    mut interactions: Query<&mut Interaction, With<Button>>,
    mut context: PadUiContext,
) {
    context.input.confirmed = None;
    let previous = focus.applied.take();
    let escape_event = |state| KeyboardInput {
        key_code: KeyCode::Escape,
        logical_key: Key::Escape,
        state,
        text: None,
        repeat: false,
        window: windows.single().unwrap_or(Entity::PLACEHOLDER),
    };
    if focus.escape_injected {
        keyboard.clear_just_pressed(KeyCode::Escape);
        keyboard.release(KeyCode::Escape);
        key_events.write(escape_event(ButtonState::Released));
        focus.escape_injected = false;
    }
    if !pad.connected() {
        focus.entity = None;
        focus.held_entity = None;
        focus.direction = Vec2::ZERO;
        focus.next_repeat = 0.0;
        menu.0 = false;
        return;
    }
    let world = matches!(*state.get(), ClientState::World | ClientState::Tutorial);
    let chat_menu = models
        .gameplay_ui
        .as_ref()
        .is_some_and(|m| m.visible && m.chat.active);
    let nanocom_open = mission.nanocom_menu_presented() || models.nanocom_popup();
    let modal = mission.stops_auto_run()
        || models.buddy_modal()
        || models.system_popup()
        || models.quit_modal()
        || models.option_modal()
        || models.resurrect_modal()
        || models.upsell_modal()
        || models.guide_modal()
        || models.game_guide_modal()
        || models.bank_modal()
        || models.vendor_modal()
        || models.rule_modal()
        || models.nano_free_tuning_modal()
        || models.user_equip_modal()
        || models.pc2pc_modal()
        || models.world_map_modal()
        || models.transportation_modal()
        || models.race_modal()
        || models.email_modal()
        || models.combi_modal()
        || models.barber_modal()
        || models.enchant_modal()
        || models.cashmall_modal()
        || models.user_store_modal();
    menu.0 =
        *state.get() == ClientState::World && !modal && pad.just_pressed(LegacyOptionAction::Menu);
    let cutscene = matches!(
        *state.get(),
        ClientState::CharacterCreateIntro | ClientState::TutorialIntro
    ) || *state.get() == ClientState::Tutorial
        && context.tutorial.scene != TutorialScene::None;
    if (cutscene || context.launcher.visible()) && !modal
        || world && !modal && !nanocom_open && !chat_menu
    {
        focus.entity = None;
        focus.held_entity = None;
        return;
    }
    // Capture owns all buttons, including South and East.
    if models
        .option_ui
        .as_ref()
        .is_some_and(|m| m.visible && m.key_capture.is_some())
    {
        focus.entity = None;
        pad.consume_ui_actions();
        return;
    }
    let confirm = pad.buttons.just_pressed(GamepadButton::South);
    let held = pad.buttons.pressed(GamepadButton::South);
    let tab_direction = if pad.buttons.just_pressed(GamepadButton::LeftTrigger) {
        -1
    } else if pad.buttons.just_pressed(GamepadButton::RightTrigger) {
        1
    } else {
        0
    };
    let cancel = pad.buttons.just_pressed(GamepadButton::East);
    let dpad = Vec2::new(
        (pad.buttons.pressed(GamepadButton::DPadRight) as u8 as f32)
            - (pad.buttons.pressed(GamepadButton::DPadLeft) as u8 as f32),
        (pad.buttons.pressed(GamepadButton::DPadDown) as u8 as f32)
            - (pad.buttons.pressed(GamepadButton::DPadUp) as u8 as f32),
    );
    let stick = pad.ui_stick_direction();
    let direction = Vec2::new(
        if dpad.x != 0.0 { dpad.x } else { stick.x },
        if dpad.y != 0.0 { dpad.y } else { stick.y },
    );
    pad.consume_ui_actions();
    if cancel && !keyboard.pressed(KeyCode::Escape) {
        keyboard.press(KeyCode::Escape);
        key_events.write(escape_event(ButtonState::Pressed));
        focus.escape_injected = true;
    }
    let mut candidates = Vec::new();
    for (entity, node, transform, stack, clip, pickable) in &buttons {
        if context.ignored.contains(entity)
            || node.size().min_element() <= 0.0
            || pickable.is_some_and(|p| !p.is_hoverable)
        {
            continue;
        }
        let center = transform.translation;
        if clip.is_some_and(|c| !c.clip.contains(center)) {
            continue;
        }
        let mut ancestor = Some(entity);
        let mut owner = entity;
        let mut z = i32::MIN;
        let mut visible = true;
        let mut scope = None;
        let mut boundary = None;
        while let Some(id) = ancestor {
            let Ok((node, parent, global_z, visibility)) = hierarchy.get(id) else {
                break;
            };
            if node.display == Display::None || visibility == Some(&Visibility::Hidden) {
                visible = false;
                break;
            }
            if scope.is_none() {
                scope = context.scopes.get(id).ok().copied();
            }
            if boundary.is_none() && context.boundaries.contains(id) {
                boundary = Some(id);
            }
            if z == i32::MIN {
                owner = id;
                if let Some(global_z) = global_z {
                    z = global_z.0;
                }
            }
            ancestor = parent.map(ChildOf::parent);
        }
        if visible {
            let (close, preferred, repeat, tab) = context.hints.get(entity).unwrap();
            candidates.push(Candidate {
                entity,
                center,
                owner: boundary.unwrap_or(owner),
                z,
                stack: stack.0,
                scope: if boundary.is_some() { None } else { scope },
                priority: if preferred {
                    -1
                } else if close {
                    1
                } else {
                    0
                },
                repeat,
                tab: tab.map(|tab| tab.0),
            });
        }
    }
    // Keep focus in the foremost window rather than visible HUD controls beneath it.
    if let Some(top) = candidates.iter().max_by_key(|c| (c.z, c.stack)).copied() {
        candidates.retain(|c| c.owner == top.owner || c.scope.is_some() && c.scope == top.scope);
    }
    candidates.sort_by(|a, b| {
        a.priority
            .cmp(&b.priority)
            .then_with(|| a.center.y.total_cmp(&b.center.y))
            .then_with(|| a.center.x.total_cmp(&b.center.x))
            .then_with(|| a.entity.index().cmp(&b.entity.index()))
    });
    if !candidates.iter().any(|c| Some(c.entity) == focus.entity) {
        focus.entity = candidates.first().map(|c| c.entity);
    }
    let now = time.elapsed_secs_f64();
    if direction != Vec2::ZERO && (direction != focus.direction || now >= focus.next_repeat) {
        if let Some(current) = focus.entity {
            focus.entity =
                choose_direction(&candidates, current, direction.normalize()).or(Some(current));
        }
        focus.next_repeat = now
            + if direction != focus.direction {
                0.35
            } else {
                0.12
            };
    }
    focus.direction = direction;
    let tab_target = if tab_direction != 0 {
        candidates
            .iter()
            .filter(|c| c.tab.is_some())
            .min_by_key(|c| {
                if tab_direction < 0 {
                    i16::from(c.tab.unwrap())
                } else {
                    -i16::from(c.tab.unwrap())
                }
            })
            .map(|c| c.entity)
    } else {
        None
    };
    if let Some(entity) = tab_target {
        focus.entity = Some(entity);
    }
    if confirm {
        focus.held_entity = focus.entity;
        context.input.confirmed = focus.entity;
        focus.next_confirm_repeat = now + 0.35;
    } else if !held || focus.held_entity != focus.entity {
        focus.held_entity = None;
    }
    if let Some(entity) = focus.entity
        && let Ok(mut interaction) = interactions.get_mut(entity)
    {
        let pressed = held && focus.held_entity == Some(entity) || tab_target == Some(entity);
        let next = if pressed {
            Interaction::Pressed
        } else {
            Interaction::Hovered
        };
        let repeat = pressed
            && candidates.iter().any(|c| c.entity == entity && c.repeat)
            && now >= focus.next_confirm_repeat;
        // A held control stays Pressed for continuous readers. Edge readers see
        // only a new press or the explicit repeat cadence, never pointer resets.
        if *interaction != Interaction::Pressed || pressed {
            if previous == Some((entity, next)) && !repeat {
                *interaction.bypass_change_detection() = next;
                if let Some(tick) = focus.applied_tick {
                    interaction.set_last_changed(tick);
                }
            } else {
                *interaction = next;
            }
            focus.applied = Some((entity, next));
            focus.applied_tick = Some(interaction.last_changed());
        }
        if repeat {
            focus.next_confirm_repeat = now + 0.12;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn directional_focus_prefers_same_row_and_skips_behind() {
        let c = |i, x, y| Candidate {
            entity: Entity::from_raw_u32(i).unwrap(),
            center: Vec2::new(x, y),
            owner: Entity::from_raw_u32(0).unwrap(),
            z: 0,
            stack: 0,
            scope: None,
            priority: 0,
            repeat: false,
            tab: None,
        };
        let buttons = [
            c(1, 0.0, 0.0),
            c(2, 30.0, 100.0),
            c(3, 70.0, 0.0),
            c(4, -10.0, 0.0),
        ];
        assert_eq!(
            choose_direction(&buttons, buttons[0].entity, Vec2::X),
            Some(buttons[2].entity)
        );
        assert_eq!(
            choose_direction(&buttons, buttons[0].entity, Vec2::Y),
            Some(buttons[1].entity)
        );
        assert_eq!(
            choose_direction(&buttons, buttons[0].entity, Vec2::NEG_X),
            Some(buttons[3].entity)
        );
    }
}

#[cfg(test)]
mod interaction_tests;
