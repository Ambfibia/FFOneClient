//! The published minimap raster uses a 4x4 image atlas over the 8192-unit XY world.
use super::*;

pub(super) const TILE_SIZE: f32 = 512.;
const IMAGE_EXTENT: f32 = 2048.;
const WORLD_EXTENT: f32 = 8192.;
pub(super) fn image_origin(number: u8) -> Vec3 {
    let index = number.saturating_sub(1) as i32;
    Vec3::new(
        -(index % 4) as f32 * IMAGE_EXTENT,
        0.,
        (4 - index / 4) as f32 * IMAGE_EXTENT,
    )
}

pub(super) fn tile_at(position: Vec3) -> Option<[i32; 2]> {
    position.is_finite().then(|| {
        [
            (-position.x / TILE_SIZE).floor() as i32,
            (position.z / TILE_SIZE).floor() as i32,
        ]
    })
}
pub(super) fn tile_id(tile: [i32; 2]) -> String {
    format!("map_{:02}_{:02}", tile[0], tile[1])
}
#[cfg(test)]
pub(super) fn tile_center(tile: [i32; 2], height: f32) -> Vec3 {
    Vec3::new(
        -(tile[0] as f32 + 0.5) * TILE_SIZE,
        height,
        (tile[1] as f32 + 0.5) * TILE_SIZE,
    )
}
pub(super) fn neighbours(center: [i32; 2]) -> Vec<[i32; 2]> {
    let mut result: Vec<_> = (-1..=1)
        .flat_map(|y| (-1..=1).map(move |x| [center[0] + x, center[1] + y]))
        .collect();
    result.sort_by_key(|t| {
        (
            (t[0] - center[0]).pow(2) + (t[1] - center[1]).pow(2),
            t[1],
            t[0],
        )
    });
    result
}
pub(super) fn viewport(e: &WorldEditor) -> (Vec3, f32) {
    if e.map_picker {
        (e.atlas_center, e.atlas_zoom)
    } else {
        (e.center, e.zoom)
    }
}
pub(super) fn project(center: Vec3, zoom: f32, position: Vec3, size: Vec2) -> Vec2 {
    size * 0.5 + Vec2::new(center.x - position.x, center.z - position.z) * zoom
}
pub(super) fn unproject(center: Vec3, zoom: f32, point: Vec2, size: Vec2) -> Vec3 {
    let xy = (point - size * 0.5) / zoom;
    center + Vec3::new(-xy.x, 0., -xy.y)
}
pub(super) fn fit(e: &mut WorldEditor, size: Vec2) {
    let center = Vec3::new(-WORLD_EXTENT * 0.5, e.center.y, WORLD_EXTENT * 0.5);
    let zoom = ((size.min_element() - 24.).max(1.) / WORLD_EXTENT).max(0.001);
    if e.map_picker {
        e.atlas_center = center;
        e.atlas_zoom = zoom;
    } else {
        e.center = center;
        e.zoom = zoom;
    }
    e.revision += 1;
}
pub(super) fn images(
    p: &mut ChildSpawnerCommands,
    e: &WorldEditor,
    size: Vec2,
    server: &AssetServer,
    handles: &mut BTreeMap<u8, Handle<Image>>,
) {
    let (center, zoom) = viewport(e);
    for row in 0..4 {
        for col in 0..4 {
            let number = (row * 4 + col + 1) as u8;
            let top_left = image_origin(number);
            let at = project(center, zoom, top_left, size);
            let extent = IMAGE_EXTENT * zoom;
            if at.x > size.x || at.y > size.y || at.x + extent < 0. || at.y + extent < 0. {
                continue;
            }
            let handle = handles.entry(number).or_insert_with(|| {
                server.load(format!(
                    "ui/en/gameplay/minimap/tiles/all_minimap_{number:02}.png"
                ))
            });
            p.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: px(at.x),
                    top: px(at.y),
                    width: px(extent),
                    height: px(extent),
                    ..default()
                },
                ImageNode::new(handle.clone()),
            ));
        }
    }
}
pub(super) fn grid(p: &mut ChildSpawnerCommands, e: &WorldEditor, f: &EditorFonts, size: Vec2) {
    let (center, zoom) = viewport(e);
    let extent = TILE_SIZE * zoom;
    for &tile in &e.available_tiles {
        let at = project(
            center,
            zoom,
            Vec3::new(
                -tile[0] as f32 * TILE_SIZE,
                0.,
                (tile[1] + 1) as f32 * TILE_SIZE,
            ),
            size,
        );
        if at.x > size.x || at.y > size.y || at.x + extent < 0. || at.y + extent < 0. {
            continue;
        }
        let selected = e.region == Some(tile);
        let nearby = e
            .region
            .is_some_and(|r| (tile[0] - r[0]).abs() <= 1 && (tile[1] - r[1]).abs() <= 1);
        p.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: px(at.x),
                top: px(at.y),
                width: px(extent),
                height: px(extent),
                border: UiRect::all(px(if selected { 2. } else { 1. })),
                ..default()
            },
            BorderColor::all(if selected {
                Color::srgb(1., 0.75, 0.15)
            } else {
                Color::srgba(0.7, 0.85, 1., 0.35)
            }),
            BackgroundColor(if nearby {
                Color::srgba(1., 0.75, 0.15, 0.12)
            } else {
                Color::NONE
            }),
        ))
        .with_children(|p| {
            if extent >= 38. {
                let mut text = editor_text(
                    f,
                    "ui.editor.world.value",
                    "{value}",
                    12.,
                    Color::WHITE,
                    false,
                );
                text.localized = LocalizedText::new("ui.editor.world.value", "{value}")
                    .with_arg("value", format!("{:02}_{:02}", tile[0], tile[1]));
                // Keep a square's label visible when its upper corner is outside the canvas.
                p.spawn((text, Node {
                    position_type: PositionType::Absolute,
                    left: px((4. - at.x).max(4.)),
                    top: px((4. - at.y).max(4.)),
                    ..default()
                }));
            }
        });
    }
}
