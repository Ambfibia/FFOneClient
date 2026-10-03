//! Shared presentation for item-card rental periods, expiry and vendor price.
use crate::localization::{Language, Localization, LocalizationSet, LocalizedText, UiTextAutoFit};
use bevy::prelude::*;
use bevy::text::LineHeight;
use chrono::{DateTime, Local, Utc};
use ffone_protocol::ItemBase0104;

#[derive(Clone, Copy, Component)]
pub(crate) enum CardOwner {
    Bank,
    Vendor,
    Inventory,
}
#[derive(Clone, Copy, Component)]
enum Field {
    RentalTitle,
    Rental,
    Expiry,
    Price,
}
#[derive(Component)]
struct CombinedBadge;
#[derive(Component)]
struct PriceCoin;
#[derive(Clone, Copy)]
pub(crate) struct CardSnapshot {
    pub item: ItemBase0104,
    pub catalog: bool,
    pub price: Option<i32>,
}

pub(crate) fn install(app: &mut App) {
    if !app.is_plugin_added::<ItemCardPlugin>() {
        app.add_plugins(ItemCardPlugin);
    }
}
struct ItemCardPlugin;
impl Plugin for ItemCardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            bind.after(crate::vendor_ui::VendorUiSet::Bind)
                .after(crate::bank_ui::BankUiSet::Bind)
                .after(crate::user_equip_ui::UserEquipUiSet::Bind)
                .before(LocalizationSet::Apply),
        );
    }
}
pub(crate) fn spawn(parent: &mut ChildSpawnerCommands, owner: CardOwner, font: &Handle<Font>) {
    if matches!(owner, CardOwner::Inventory) {
        parent.spawn((
            CombinedBadge,
            Node {
                position_type: PositionType::Absolute,
                left: px(52.),
                top: px(52.),
                width: px(26.),
                height: px(26.),
                display: Display::None,
                ..default()
            },
            ImageNode::default(),
            ZIndex(3),
            Pickable::IGNORE,
            bevy::ui::FocusPolicy::Pass,
        ));
    }
    for (field, x, y, w, h, size, key, fallback) in [
        (
            Field::RentalTitle,
            10.,
            138.,
            200.,
            20.,
            12.,
            "ui.card.rental_period",
            "RENTAL PERIOD",
        ),
        (
            Field::Rental,
            5.,
            174.,
            300.,
            60.,
            14.,
            "ui.card.rental_value",
            "{value}",
        ),
        (
            Field::Expiry,
            82.,
            72.,
            210.,
            20.,
            8.,
            "ui.card.expiry_compact",
            "{date} {time}",
        ),
    ] {
        let text_font = (
            TextFont {
                font: (font.clone()).into(),
                font_size: (size).into(),
                ..default()
            },
            LineHeight::Px(if size == 14. { 20. } else { 13.71 }),
        );
        parent.spawn((
            owner,
            field,
            Node {
                position_type: PositionType::Absolute,
                left: px(x),
                top: px(y),
                width: px(w),
                height: px(h),
                display: Display::None,
                ..default()
            },
            ZIndex(2),
            Text::default(),
            LocalizedText::new(key, fallback),
            UiTextAutoFit::new(w, h, &text_font),
            text_font,
            TextColor(if matches!(field, Field::Rental | Field::Price) {
                Color::srgb(1., 1., 0.)
            } else {
                Color::WHITE
            }),
            TextLayout::default().with_justify(if matches!(field, Field::Rental | Field::Price) {
                Justify::Center
            } else {
                Justify::Left
            }),
            Pickable::IGNORE,
            bevy::ui::FocusPolicy::Pass,
        ));
    }
    parent
        .spawn((
            owner,
            Field::Price,
            Node {
                position_type: PositionType::Absolute,
                left: px(3.),
                top: px(343.),
                width: px(303.),
                height: px(32.),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: px(5.),
                display: Display::None,
                ..default()
            },
            ZIndex(2),
            LocalizedText::new("ui.card.price_value", "{price}"),
            Pickable::IGNORE,
            bevy::ui::FocusPolicy::Pass,
        ))
        .with_children(|row| {
            row.spawn((
                owner,
                Field::Price,
                Node::default(),
                Text::default(),
                LocalizedText::new("ui.card.price_value", "{price}"),
                (
                    TextFont {
                        font: (font.clone()).into(),
                        font_size: (14.).into(),
                        ..default()
                    },
                    LineHeight::Px(20.),
                ),
                TextColor(Color::srgb(1., 1., 0.)),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
            row.spawn((
                PriceCoin,
                Node {
                    width: px(32.),
                    height: px(32.),
                    ..default()
                },
                ImageNode::default(),
                Pickable::IGNORE,
                bevy::ui::FocusPolicy::Pass,
            ));
        });
}
fn local_date(timestamp: i32) -> (String, String) {
    let dt = DateTime::<Utc>::from_timestamp(i64::from(timestamp), 0)
        .unwrap_or(DateTime::<Utc>::UNIX_EPOCH)
        .with_timezone(&Local);
    (
        dt.format("%d.%m.%Y").to_string(),
        dt.format("%H:%M UTC%:z").to_string(),
    )
}
fn duration_parts(seconds: i32) -> [(i32, &'static str, &'static str); 4] {
    [
        (seconds / 86400, "ui.card.days", "{value} DAYS"),
        ((seconds % 86400) / 3600, "ui.card.hours", "{value} HOURS"),
        ((seconds % 3600) / 60, "ui.card.minutes", "{value} MINUTES"),
        (seconds % 60, "ui.card.seconds", "{value} SECONDS"),
    ]
}
fn rental(snapshot: CardSnapshot, loc: &Localization, language: &Language) -> LocalizedText {
    if snapshot.catalog {
        let value = duration_parts(snapshot.item.time_limit)
            .into_iter()
            .filter(|(n, _, _)| *n > 0)
            .map(|(n, key, fallback)| {
                loc.text(
                    language,
                    &LocalizedText::new(key, fallback).with_arg("value", n.to_string()),
                )
            })
            .collect::<Vec<_>>()
            .join(" ");
        LocalizedText::new(
            "ui.card.rental_from_purchase",
            "{duration}\nFrom time of purchase",
        )
        .with_arg("duration", value)
    } else {
        let (date, time) = local_date(snapshot.item.time_limit);
        LocalizedText::new("ui.card.rental_expires", "Expires {date}\nAt {time}")
            .with_arg("date", date)
            .with_arg("time", time)
    }
}
fn bind(
    bank: Option<Res<crate::bank_ui::BankItemPopupState>>,
    vendor: Option<Res<crate::vendor_ui::VendorItemPopupState>>,
    inventory: Option<Res<crate::user_equip_ui::UserEquipItemPopupState>>,
    projection: Option<Res<crate::user_equip_ui::UserEquipItemModeProjection>>,
    loc: Option<Res<Localization>>,
    language: Option<Res<Language>>,
    mut fields: Query<(&CardOwner, &Field, &mut Node, &mut LocalizedText), Without<CombinedBadge>>,
    mut badges: Query<
        (&mut Node, &mut ImageNode),
        (With<CombinedBadge>, Without<Field>, Without<PriceCoin>),
    >,
    mut coins: Query<&mut ImageNode, (With<PriceCoin>, Without<CombinedBadge>)>,
    server: Option<Res<AssetServer>>,
) {
    let (Some(loc), Some(language)) = (loc, language) else {
        return;
    };
    let inventory_offset = match inventory.as_ref().and_then(|p| p.selected()) {
        Some(crate::user_equip_ui::UserEquipSlotEndpoint::Equipment { .. }) => 13.,
        Some(endpoint) => {
            if projection
                .as_ref()
                .and_then(|p| p.item_at(endpoint))
                .is_some_and(|p| matches!(p.item.item_type, 7 | 9))
            {
                0.
            } else {
                14.
            }
        }
        None => 0.,
    };
    if let Some(server) = server.as_ref() {
        for mut image in &mut coins {
            let handle = server.load(crate::vendor_ui::VENDOR_TAROS_ICON_PATH);
            if image.image != handle {
                image.image = handle;
            }
        }
    }
    for (mut node, mut image) in &mut badges {
        let top = px(52. + inventory_offset);
        if node.top != top {
            node.top = top;
        }
        let combined = inventory
            .as_ref()
            .and_then(|p| p.selected())
            .and_then(|e| projection.as_ref()?.item_at(e))
            .is_some_and(|p| {
                (0..=3).contains(&p.item.item_type) && (p.item.option >> 16) as i16 > 0
            });
        let display = if combined {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
        if let Some(server) = server.as_ref() {
            let handle = server.load(crate::user_equip_ui::USER_EQUIP_COMBINED_PATH);
            if image.image != handle {
                image.image = handle;
            }
        }
    }
    for (owner, field, mut node, mut text) in &mut fields {
        if matches!(owner, CardOwner::Inventory) && !matches!(field, Field::Price) {
            let top = px(match field {
                Field::RentalTitle => 138.,
                Field::Rental => 174.,
                Field::Expiry => 72.,
                Field::Price => 348.,
            } + inventory_offset);
            if node.top != top {
                node.top = top;
            }
        }
        let selected = match owner {
            CardOwner::Bank => bank.as_ref().and_then(|p| p.card_snapshot()),
            CardOwner::Vendor => vendor.as_ref().and_then(|p| p.card_snapshot()),
            CardOwner::Inventory => inventory
                .as_ref()
                .and_then(|p| p.selected())
                .and_then(|e| projection.as_ref()?.item_at(e))
                .map(|p| CardSnapshot {
                    item: p.item,
                    catalog: false,
                    price: None,
                }),
        };
        let value = selected.and_then(|s| match field {
            Field::RentalTitle if s.item.item_type == 10 && s.item.time_limit > 0 => {
                Some(LocalizedText::new("ui.card.rental_period", "RENTAL PERIOD"))
            }
            Field::Rental if s.item.item_type == 10 && s.item.time_limit > 0 => {
                Some(rental(s, &loc, &language))
            }
            Field::Expiry if s.item.item_type != 10 && s.item.time_limit > 0 => {
                let (date, time) = local_date(s.item.time_limit);
                Some(
                    LocalizedText::new("ui.card.expiry_compact", "{date} {time}")
                        .with_arg("date", date)
                        .with_arg("time", time),
                )
            }
            Field::Price => s.price.map(|price| {
                LocalizedText::new("ui.card.price_value", "{price}")
                    .with_arg("price", price.to_string())
            }),
            _ => None,
        });
        let display = if value.is_some() {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display {
            node.display = display;
        }
        if let Some(value) = value {
            if *text != value {
                *text = value;
            }
        }
    }
}
#[cfg(test)]
mod tests;
