//! Reuse the gameplay minimap's authored icon catalog without recolouring its artwork.
use super::*;
use ffone_client::ui::gameplay::{MinimapMarkerIcon, MinimapTableMarkerIcon};

#[derive(Default)]
pub(super) struct Images {
    pub atlas: BTreeMap<u8, Handle<Image>>,
    pub terrain: BTreeMap<String, Handle<Image>>,
    icons: BTreeMap<&'static str, Handle<Image>>,
}
pub(super) fn placeholder_colour(red:bool)->Color {
    if red {Color::srgb(1.,0.15,0.2)} else {Color::srgb(0.25,0.76,1.)}
}
pub(super) fn icon(
    kind: usize,
    npc_type: i64,
    table: &BTreeMap<i64, i32>,
) -> Option<MinimapMarkerIcon> {
    if kind > 2 {
        return None;
    }
    // These tutorial actors have their own gameplay minimap branch, independent
    // of the ordinary NPC table icon (app/mission_indicators.rs).
    match npc_type {
        2671..=2673 => return Some(MinimapMarkerIcon::ShowNpc),
        2674..=2677 => return Some(MinimapMarkerIcon::Mob),
        2678 => return Some(MinimapMarkerIcon::Fusion),
        _ => {}
    }
    let index = *table.get(&npc_type)?;
    (index > 0)
        .then(|| MinimapTableMarkerIcon::from_table_data(index))
        .flatten()
        .map(MinimapMarkerIcon::TableData)
}
pub(super) fn draw(
    p: &mut ChildSpawnerCommands,
    entry: &model::Placement,
    at: Vec2,
    selected: bool,
    e: &WorldEditor,
    server: &AssetServer,
    images: &mut Images,
) {
    let red=entry.kind<3&&e.object_npc_types.contains(&entry.type_id);
    let icon = if red {None} else {icon(entry.kind, entry.type_id, &e.npc_map_icons)};
    let colour = if red {placeholder_colour(true)} else if selected {
        Color::srgb(1., 0.78, 0.22)
    } else {
        [
            Color::srgb(0.25, 0.76, 1.),
            Color::srgb(1., 0.35, 0.4),
            Color::srgb(0.95, 0.5, 0.9),
            Color::srgb(0.45, 1., 0.5),
            Color::srgb(0.65, 0.75, 0.85),
        ][entry.kind]
    };
    let scale = if selected {
        1.
    } else {
        (e.zoom / 0.8).sqrt().clamp(0.3, 1.)
    };
    let size = icon.map(|i| i.dimensions() * scale).unwrap_or_else(|| {
        Vec2::splat(if selected {
            10.
        } else {
            (10. * (e.zoom / 0.8).sqrt()).clamp(2., 10.)
        })
    });
    let layer = if selected { 2 } else { 1 };
    if let Some(icon) = icon {
        if selected {
            p.spawn((
                Marker,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(at.x - size.x * 0.5 - 3.),
                    top: px(at.y - size.y * 0.5 - 3.),
                    width: px(size.x + 6.),
                    height: px(size.y + 6.),
                    border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(px(5)),
                    ..default()
                },
                BorderColor::all(colour),
                ZIndex(layer),
            ));
        }
        let handle = images
            .icons
            .entry(icon.asset_path())
            .or_insert_with(|| server.load(icon.asset_path()));
        p.spawn((
            Marker,
            Node {
                position_type: PositionType::Absolute,
                left: px(at.x - size.x * 0.5),
                top: px(at.y - size.y * 0.5),
                width: px(size.x),
                height: px(size.y),
                ..default()
            },
            ImageNode::new(handle.clone()),
            ZIndex(layer),
        ));
    } else {
        // Keep placements with no declared minimap icon editable, including
        // location helpers; never infer their icon from friendliness or team.
        p.spawn((
            Marker,
            Node {
                position_type: PositionType::Absolute,
                left: px(at.x - size.x * 0.5),
                top: px(at.y - size.y * 0.5),
                width: px(size.x),
                height: px(size.y),
                border_radius: BorderRadius::all(px(size.x * 0.5)),
                ..default()
            },
            BackgroundColor(colour),
            ZIndex(layer),
        ));
    }
    if selected || (icon.is_none() && e.zoom >= 0.3) {
        let direction = Vec2::new(
            -entry.angle.to_radians().sin(),
            entry.angle.to_radians().cos(),
        );
        for step in 1..7 {
            let tip = at + direction * (size.max_element() * 0.5 + step as f32 * 2.);
            p.spawn((
                Marker,
                Node {
                    position_type: PositionType::Absolute,
                    left: px(tip.x - 1.),
                    top: px(tip.y - 1.),
                    width: px(3),
                    height: px(3),
                    ..default()
                },
                BackgroundColor(colour),
                ZIndex(layer),
            ));
        }
    }
}
