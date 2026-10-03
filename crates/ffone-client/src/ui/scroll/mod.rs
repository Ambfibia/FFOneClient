//! Pointer ownership for the bank/shop scrollbars, including their inventory pane.
use crate::{bank_ui::*, user_equip_ui::*, vendor_ui::*};
use bevy::{prelude::*, window::PrimaryWindow};

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Owner {
    Bank,
    Vendor,
    BankInventory,
    VendorInventory,
}
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Part {
    Track,
    Thumb,
    Up,
    Down,
}
#[derive(Default)]
struct Pointer {
    held: Option<(Owner, Part)>,
    anchor: f32,
    value: f32,
    repeat_at: f32,
    page_direction: f32,
    identity: (i32, i32, usize),
}
pub(crate) fn install(app: &mut App) {
    if !app.is_plugin_added::<Plugin>() {
        app.add_plugins(Plugin);
    }
}
struct Plugin;
impl bevy::app::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (spawn_inventory_bars, input, bind_inventory_thumbs)
                .chain()
                .after(BankUiSet::Interaction)
                .after(VendorUiSet::Interaction)
                .before(BankUiSet::Bind)
                .before(VendorUiSet::Bind),
        );
    }
}
pub(crate) fn control(owner: Owner, part: Part) -> impl Bundle {
    (
        owner,
        part,
        Button,
        Pickable::default(),
        bevy::ui::FocusPolicy::Block,
    )
}
#[derive(Component)]
struct InventoryBarInstalled;
fn spawn_inventory_bars(
    mut commands: Commands,
    assets: If<Res<AssetServer>>,
    banks: Query<(Entity, &BankUiElement), (Added<BankUiElement>, Without<InventoryBarInstalled>)>,
    vendors: Query<
        (Entity, &VendorUiElement),
        (Added<VendorUiElement>, Without<InventoryBarInstalled>),
    >,
) {
    let parents = banks
        .iter()
        .filter(|(_, e)| **e == BankUiElement::PcStuffPanel)
        .map(|(e, _)| (e, Owner::BankInventory))
        .chain(
            vendors
                .iter()
                .filter(|(_, e)| **e == VendorUiElement::PcStuffPanel)
                .map(|(e, _)| (e, Owner::VendorInventory)),
        );
    for (entity, owner) in parents {
        commands
            .entity(entity)
            .insert(InventoryBarInstalled)
            .with_children(|p| {
                for (part, rect, path) in [
                    (
                        Part::Track,
                        USER_EQUIP_SCROLL_TRACK_RECT,
                        USER_EQUIP_SCROLL_TRACK_PATH,
                    ),
                    (
                        Part::Thumb,
                        UserEquipUiRect::new(355., 48., 15., 15.),
                        USER_EQUIP_SCROLL_THUMB_PATH,
                    ),
                    (
                        Part::Up,
                        USER_EQUIP_SCROLL_UP_RECT,
                        USER_EQUIP_SCROLL_UP_PATH,
                    ),
                    (
                        Part::Down,
                        USER_EQUIP_SCROLL_DOWN_RECT,
                        USER_EQUIP_SCROLL_DOWN_PATH,
                    ),
                ] {
                    p.spawn((
                        Name::new(format!("{owner:?} scroll {part:?}")),
                        control(owner, part),
                        Node {
                            position_type: PositionType::Absolute,
                            left: px(rect.left),
                            top: px(rect.top),
                            width: px(rect.width),
                            height: px(rect.height),
                            ..default()
                        },
                        ImageNode {
                            image: assets.load(path),
                            image_mode: if matches!(part, Part::Track | Part::Thumb) {
                                NodeImageMode::Sliced(TextureSlicer {
                                    border: BorderRect {
                                        min_inset: Vec2::new(2., 4.),
                                        max_inset: Vec2::new(2., 4.),
                                    },
                                    center_scale_mode: SliceScaleMode::Stretch,
                                    sides_scale_mode: SliceScaleMode::Stretch,
                                    max_corner_scale: 1.,
                                })
                            } else {
                                NodeImageMode::Auto
                            },
                            ..default()
                        },
                        ZIndex(3),
                    ));
                }
            });
    }
}
// Native IMGUI adapter: the 12 px thumb padding is retained in addition to
// the visible fraction. The same extent must drive rendering and pointer travel.
pub(crate) fn thumb_extent(track: f32, viewport: f32, maximum: f32) -> f32 {
    if maximum <= 0.0 {
        return track;
    }
    (12.0 + viewport * (track - 12.0).max(0.0) / (maximum + viewport))
        .clamp(12.0_f32.min(track), track)
}
fn drag_value(start: f32, delta: f32, maximum: f32, travel: f32) -> f32 {
    if travel <= 0. || !delta.is_finite() {
        return start.clamp(0., maximum);
    }
    (start + delta / travel * maximum).clamp(0., maximum)
}
fn page_direction(cursor: f32, top: f32, extent: f32) -> f32 {
    if cursor < top {
        -1.
    } else if cursor > top + extent {
        1.
    } else {
        0.
    }
}
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn input(
    time: Res<Time>,
    mouse: Option<Res<ButtonInput<MouseButton>>>,
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    parts: Query<(
        &Owner,
        &Part,
        &Interaction,
        &ComputedNode,
        &UiGlobalTransform,
    )>,
    mut bank: Option<ResMut<BankUiState>>,
    bank_modal: Option<Res<BankModalState>>,
    bank_projection: Option<Res<BankModeProjection0104>>,
    bank_search: Option<Res<BankSearch>>,
    mut vendor: Option<ResMut<VendorUiState>>,
    vendor_modal: Option<Res<VendorModalState>>,
    vendor_projection: Option<Res<VendorModeProjection0104>>,
    mut pointer: Local<Pointer>,
) {
    let Some(mouse) = mouse else {
        *pointer = default();
        return;
    };
    let Ok(window) = windows.single() else {
        *pointer = default();
        return;
    };
    let Some(cursor) = window.physical_cursor_position() else {
        *pointer = default();
        return;
    };
    if !window.focused
        || !mouse.pressed(MouseButton::Left)
        || keyboard
            .as_ref()
            .is_some_and(|k| k.just_pressed(KeyCode::Escape))
    {
        *pointer = default();
        return;
    }
    let hovered = parts
        .iter()
        .filter(|(_, _, i, _, _)| **i != Interaction::None)
        .min_by_key(|(_, p, _, _, _)| if **p == Part::Thumb { 0 } else { 1 })
        .map(|(o, p, _, _, _)| (*o, *p));
    if mouse.just_pressed(MouseButton::Left) {
        pointer.held = hovered;
        pointer.repeat_at = time.elapsed_secs() + 0.25;
    }
    let Some((owner, part)) = pointer.held else {
        return;
    };
    let (allowed, current, maximum, page, identity) = match owner {
        Owner::Bank | Owner::BankInventory => {
            let (Some(state), Some(modal), Some(projection)) =
                (bank.as_ref(), bank_modal.as_ref(), bank_projection.as_ref())
            else {
                *pointer = default();
                return;
            };
            let inventory = owner == Owner::BankInventory;
            (
                state.input_capabilities(**modal).scroll,
                if inventory {
                    state.inventory_scroll_y
                } else {
                    state.bank_scroll_y
                },
                if inventory {
                    user_equip_scroll_max()
                } else {
                    bank_search
                        .as_ref()
                        .map_or_else(bank_scroll_max, |s| s.maximum())
                },
                if inventory {
                    500.
                } else {
                    BANK_VIEWPORT_HEIGHT
                },
                (projection.owner_pc_id, projection.npc_id, 0),
            )
        }
        Owner::Vendor | Owner::VendorInventory => {
            let (Some(state), Some(modal), Some(projection)) = (
                vendor.as_ref(),
                vendor_modal.as_ref(),
                vendor_projection.as_ref(),
            ) else {
                *pointer = default();
                return;
            };
            let inventory = owner == Owner::VendorInventory;
            (
                state.input_capabilities(**modal).scroll,
                if inventory {
                    state.inventory_scroll_y
                } else {
                    state.vendor_scroll_y
                },
                if inventory {
                    user_equip_scroll_max()
                } else {
                    vendor_scroll_max(projection.rows_for_tab(state.tab))
                },
                if inventory {
                    500.
                } else {
                    VENDOR_LIST_VIEWPORT_RECT.height
                },
                (
                    projection.owner_pc_id,
                    projection.session.requested_npc_id,
                    state.tab as usize,
                ),
            )
        }
    };
    if !allowed || maximum <= 0. {
        *pointer = default();
        return;
    }
    let Some((_, _, _, track, transform)) = parts
        .iter()
        .find(|(o, p, _, _, _)| **o == owner && **p == Part::Track)
    else {
        *pointer = default();
        return;
    };
    let scale = track.inverse_scale_factor();
    let cursor_y = (cursor.y - transform.translation.y + track.size().y / 2.) * scale;
    let height = track.size().y * scale;
    let extent = thumb_extent(height, page, maximum);
    let travel = (height - extent).max(0.);
    if mouse.just_pressed(MouseButton::Left) {
        pointer.anchor = cursor_y;
        pointer.value = current;
        pointer.identity = identity;
        pointer.page_direction = page_direction(cursor_y, current / maximum * travel, extent);
    } else if pointer.identity != identity {
        *pointer = default();
        return;
    }
    let value = if part == Part::Thumb {
        drag_value(pointer.value, cursor_y - pointer.anchor, maximum, travel)
    } else {
        if hovered != pointer.held
            || (!mouse.just_pressed(MouseButton::Left) && time.elapsed_secs() < pointer.repeat_at)
        {
            return;
        }
        if !mouse.just_pressed(MouseButton::Left) {
            pointer.repeat_at = time.elapsed_secs() + 0.03;
        }
        let top = current / maximum * travel;
        let delta = match part {
            Part::Up => -10.,
            Part::Down => 10.,
            Part::Track if page_direction(cursor_y, top, extent) == pointer.page_direction => {
                page * 0.9 * pointer.page_direction
            }
            Part::Track => {
                pointer.held = None;
                0.
            }
            _ => 0.,
        };
        (current + delta).clamp(0., maximum)
    };
    match owner {
        Owner::Bank => {
            let state = bank.as_mut().unwrap();
            if state.bank_scroll_y != value {
                state.bank_scroll_y = value;
            }
        }
        Owner::BankInventory => {
            let state = bank.as_mut().unwrap();
            if state.inventory_scroll_y != value {
                state.inventory_scroll_y = value;
            }
        }
        Owner::Vendor => {
            let state = vendor.as_mut().unwrap();
            if state.vendor_scroll_y != value {
                state.vendor_scroll_y = value;
            }
        }
        Owner::VendorInventory => {
            let state = vendor.as_mut().unwrap();
            if state.inventory_scroll_y != value {
                state.inventory_scroll_y = value;
            }
        }
    }
}
fn bind_inventory_thumbs(
    bank: Option<Res<BankUiState>>,
    vendor: Option<Res<VendorUiState>>,
    mut parts: Query<(&Owner, &Part, &mut Node)>,
) {
    for (owner, part, mut node) in &mut parts {
        if *part != Part::Thumb {
            continue;
        }
        let value = match owner {
            Owner::BankInventory => bank.as_ref().map(|s| s.inventory_scroll_y),
            Owner::VendorInventory => vendor.as_ref().map(|s| s.inventory_scroll_y),
            _ => None,
        };
        if let Some(value) = value {
            let extent = thumb_extent(476., 500., user_equip_scroll_max());
            let height = px(extent);
            if node.height != height {
                node.height = height;
            }
            let top = px(48. + (476. - extent) * value / user_equip_scroll_max());
            if node.top != top {
                node.top = top;
            }
        }
    }
}
#[cfg(test)]
mod tests;
