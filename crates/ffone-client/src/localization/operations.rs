use super::*;

pub(super) fn fit_region_height(height: f32, font_size: f32, line_height: LineHeight) -> f32 {
    // Some source rectangles are only vertical anchors (e.g. 5 px for a
    // 12 px label). Preserve their intentional single-line overflow.
    height.max(match line_height {
        LineHeight::Px(value) => value,
        LineHeight::RelativeToFont(value) => value * font_size,
    })
}

pub(super) fn fixed_text_region(node: &Node) -> Option<Vec2> {
    let (Val::Px(width), Val::Px(height)) = (node.width, node.height) else {
        return None;
    };
    let inset = |value| match value {
        Val::Px(value) => Some(value),
        Val::Percent(0.0) | Val::Auto => Some(0.0),
        _ => None,
    };
    let width = width
        - inset(node.padding.left)?
        - inset(node.padding.right)?
        - inset(node.border.left)?
        - inset(node.border.right)?;
    let height = height
        - inset(node.padding.top)?
        - inset(node.padding.bottom)?
        - inset(node.border.top)?
        - inset(node.border.bottom)?;
    (width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0)
        .then_some(Vec2::new(width, height))
}

/// Only fixed rectangles are eligible. A parent's whole content area belongs
/// to a label only when that label is its sole, in-flow child. Lists, chat,
/// input fields and content-sized containers must not size themselves from
/// their own measured text and feed that size back into fitting.
pub(super) fn admit_bounded_ui_text(
    mut commands: Commands,
    texts: Query<
        (Entity, &Node, &TextFont, &LineHeight, Option<&ChildOf>),
        (
            With<LocalizedText>,
            With<Text>,
            Without<UiTextAutoFit>,
            Without<crate::text_edit::EditVisual>,
            Without<UiTextNoAutoFit>,
        ),
    >,
    parents: Query<(&Node, &Children)>,
) {
    for (entity, node, font, line_height, parent) in &texts {
        let region = fixed_text_region(node)
            .map(|bounds| (entity, bounds))
            .or_else(|| {
                let parent = parent?.parent();
                let (parent_node, children) = parents.get(parent).ok()?;
                if children.len() != 1 || node.position_type == PositionType::Absolute {
                    return None;
                }
                // Margins and explicit child sizing reserve a different rectangle.
                if node.margin != UiRect::default()
                    || !matches!(node.width, Val::Auto | Val::Percent(100.0))
                    || !matches!(node.height, Val::Auto | Val::Percent(100.0))
                {
                    return None;
                }
                fixed_text_region(parent_node).map(|bounds| (parent, bounds))
            });
        if let Some((owner, bounds)) = region {
            commands.entity(entity).insert((
                UiTextAutoFit::new(
                    bounds.x,
                    fit_region_height(
                        bounds.y,
                        font.font_size.eval(Vec2::ZERO, 16.0),
                        *line_height,
                    ),
                    &(font.clone(), *line_height),
                ),
                UiTextFitRegion(owner),
            ));
        }
    }
}

pub(super) fn auto_fit_font_size(
    current_font_size: f32,
    min_font_size: f32,
    bounds: Vec2,
    measured: Vec2,
) -> f32 {
    if !current_font_size.is_finite()
        || !min_font_size.is_finite()
        || !bounds.is_finite()
        || !measured.is_finite()
        || bounds.x <= 0.0
        || bounds.y <= 0.0
        || measured.x <= 0.0
        || measured.y <= 0.0
    {
        return current_font_size;
    }
    let scale = (bounds.x / measured.x).min(bounds.y / measured.y).min(1.0);
    if scale >= 1.0 {
        return current_font_size;
    }
    // A small safety margin avoids a one-pixel clip from font rasterization.
    (current_font_size * scale * 0.98)
        .clamp(min_font_size, current_font_size)
        .floor()
        .max(min_font_size)
}

pub(super) fn scaled_line_height(base: LineHeight, scale: f32) -> LineHeight {
    match base {
        LineHeight::Px(value) => LineHeight::Px(value * scale),
        LineHeight::RelativeToFont(value) => LineHeight::RelativeToFont(value),
    }
}

pub(super) fn auto_fit_measurement(buffer: &parley::Layout<TextBrush>, layout: &TextLayoutInfo) -> Vec2 {
    if buffer.is_empty() || !layout.scale_factor.is_finite() || layout.scale_factor <= 0.0 {
        return layout.size;
    }
    // Bevy rounds TextLayoutInfo up to whole physical pixels. Comparing that
    // allocation with a fractional line height falsely shrinks fitting labels.
    // Parley's decoration height also clamps negative leading to zero. Native
    // labels intentionally use tight leading, so fit their typographic lines
    // rather than the taller selection boxes. Width remains the full advance.
    let height = buffer.lines().map(|line| line.metrics().line_height).sum();
    Vec2::new(buffer.full_width(), height) / layout.scale_factor
}

pub(super) fn auto_fit_ui_text(
    mut texts: Query<(
        &Text,
        &TextLayoutInfo,
        &ComputedTextBlock,
        &ComputedNode,
        &mut TextFont,
        &mut LineHeight,
        &mut UiTextAutoFit,
    )>,
) {
    for (text, layout, block, computed_node, mut font, mut line_height, mut fit) in &mut texts {
        // Hidden / not-yet-laid-out nodes have a zero content box. Bevy can
        // still shape their text at minimum width; that is not visible overflow.
        // Never persist a smaller font from that provisional layout.
        if computed_node.is_empty() {
            continue;
        }
        if fit.measured_text != text.0 {
            fit.measured_text.clone_from(&text.0);
            if (font.font_size.eval(Vec2::ZERO, 16.0) - fit.max_font_size).abs() > 0.01 {
                font.font_size = fit.max_font_size.into();
                *line_height = fit.max_line_height;
                // The current layout was measured with the previous size.
                // Re-evaluate after Bevy lays out the restored source font.
                continue;
            }
        }
        if text.0.is_empty() || layout.size.x <= 0.0 || layout.size.y <= 0.0 {
            continue;
        }
        let target = auto_fit_font_size(
            font.font_size.eval(Vec2::ZERO, 16.0),
            fit.min_font_size,
            Vec2::new(fit.max_width, fit.max_height),
            auto_fit_measurement(block.buffer(), layout),
        );
        if target + 0.01 < font.font_size.eval(Vec2::ZERO, 16.0) {
            font.font_size = target.into();
            *line_height = scaled_line_height(
                fit.max_line_height,
                target / fit.max_font_size.max(f32::EPSILON),
            );
        }
    }
}

pub(super) fn unique_fallback_source_keys(fallback: &BTreeMap<String, String>) -> BTreeMap<String, String> {
    let mut unique = BTreeMap::<String, Option<String>>::new();
    for (key, source) in fallback {
        unique
            .entry(source.clone())
            .and_modify(|entry| *entry = None)
            .or_insert_with(|| Some(key.clone()));
    }
    unique
        .into_iter()
        .filter_map(|(source, key)| key.map(|key| (source, key)))
        .collect()
}

/// Converts the remaining legacy source-text API to a stable semantic key.
/// New call sites should construct [`LocalizedText`] with the key directly.
pub fn semantic_key_for_source(text: &str) -> Option<&'static str> {
    Some(match text {
        "USERNAME :" => "ui.login.username",
        "PASSWORD :" => "ui.login.password",
        "LOG IN" => "ui.login.submit",
        "DISCORD COMMUNITY" => "ui.login.discord",
        "HOW DO I REGISTER?" => "ui.login.register",
        "SETTINGS" => "ui.login.settings",
        "To Register:
Choose an Username and Password into the appropriate boxes and press Log In. Make sure to remember these as there is no account recovery!" => {
            "ui.login.registration_instructions"
        }
        "Press SPACE BAR to skip." => "ui.tutorial.skip",
        "SELECT A CHARACTER" => "ui.character_select.title",
        "EMPTY" => "ui.character_select.empty",
        "UNLIMITED ACCESS ONLY" => "ui.character_select.unlimited_only",
        "DELETE CHARACTER" => "ui.character_select.delete",
        "CREATE CHARACTER" => "ui.character_select.create",
        "ENTER THE GAME" => "ui.character_select.enter",
        "QUIT" => "ui.common.quit",
        "Please enter your character's first name to delete" => "ui.character_select.delete_prompt",
        "CANCEL" => "ui.common.cancel",
        "DELETE" => "ui.common.delete",
        "CHARACTER CREATION" => "ui.character_create.title",
        "RANDOM" => "ui.character_create.random",
        "1. CHOOSE BODY" => "ui.character_create.step.body",
        "BOY" => "ui.character_create.boy",
        "GIRL" => "ui.character_create.girl",
        "MEDIUM" => "ui.character_create.medium",
        "SKIN" => "ui.character_create.skin",
        "2. CHOOSE FEATURES" => "ui.character_create.step.features",
        "HAIR 2" => "ui.character_create.hair",
        "FACE" => "ui.character_create.face_label",
        "FACE 2" => "ui.character_create.face",
        "EYE COLOR" => "ui.character_create.eye_color",
        "3. CHOOSE CLOTHES" => "ui.character_create.step.clothes",
        "CONTINUE" => "ui.common.continue",
        "EXIT" => "ui.common.exit",
        "GENERATE A NAME" => "ui.character_create.generate_name",
        "CREATE YOUR OWN" => "ui.character_create.custom_name",
        "-OR-" => "ui.common.or",
        "Names must have at least two parts.\nSingle-word names will not be accepted." => {
            "ui.character_create.generated_name_rules"
        }
        "WANT TO CREATE YOUR OWN NAME?" => "ui.character_create.custom_name_question",
        "Type in the name you wish to use and\nclick 'CONTINUE'. Your custom name\nwill be submitted for review." => {
            "ui.character_create.custom_name_help"
        }
        "Names must have at least two parts.\nCustom first names cannot exceed 8 characters.\nCustom last names cannot exceed 16 characters." => {
            "ui.character_create.custom_name_rules"
        }
        "Press ENTER to access chat and menus." => "ui.hud.chat.open_hint",
        "SEND" => "ui.hud.chat.send",
        "ALL" => "ui.hud.chat.channel.all",
        "GROUP" => "ui.hud.chat.channel.group",
        "BUDDY" => "ui.hud.chat.channel.buddy",
        "Enter your account name and password." => "status.login.credentials",
        "Enter your account name." => "status.login.username",
        "Enter your password." => "status.login.password",
        "Disconnected from OpenFusion" => "status.login.disconnected",
        "Subscription only" => "status.character_select.subscription_only",
        "No available character slot" => "status.character_select.no_available_slot",
        "No selectable character to delete" => "status.character_select.no_selectable_character",
        "Create character is pending a native OpenFusion network command" => {
            "status.character_select.create_pending"
        }
        "Delete character is pending a native OpenFusion network command" => {
            "status.character_select.delete_pending"
        }
        _ => return None,
    })
}

pub(super) fn tutorial_instruction_key(source: &str) -> Option<&'static str> {
    TUTORIAL_INSTRUCTION_KEYS
        .iter()
        .find_map(|(legacy_source, key)| (*legacy_source == source).then_some(*key))
}

pub(super) fn template_args(template: &str) -> BTreeSet<String> {
    let mut args = BTreeSet::new();
    let mut remainder = template;
    while let Some(open) = remainder.find('{') {
        remainder = &remainder[open + 1..];
        let Some(close) = remainder.find('}') else {
            break;
        };
        let name = &remainder[..close];
        if !name.is_empty()
            && name
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || character == '_')
        {
            args.insert(name.to_owned());
        }
        remainder = &remainder[close + 1..];
    }
    args
}
