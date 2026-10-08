//! Cursor-anchored menus block click-through without dimming the workspace.
use super::*;
use bevy::ui::FocusPolicy;
use mission_skin::{CYAN, MUTED, TEXT};
use mission_workspace::{Command, Context};

#[derive(Component)]
pub(super) struct Panel {
    pub anchor: Vec2,
    pub interactive: bool,
}

pub(super) fn draw(
    commands: &mut Commands,
    f: &EditorFonts,
    e: &XdtEditor,
    l: &Localization,
    lang: &Language,
    width: f32,
    height: f32,
) {
    if e.workspace.context.is_none() && e.workspace.help.is_none() {
        return;
    }
    let interactive = e.workspace.context.is_some();
    let anchor = e
        .workspace
        .context
        .as_ref()
        .map_or(Vec2::new(width - 360., 170.), |(at, _)| *at);
    let focus = if interactive {
        FocusPolicy::Block
    } else {
        FocusPolicy::Pass
    };
    commands.spawn((Root, Node {
        position_type: PositionType::Absolute, width: percent(100), height: percent(100),
        ..default()
    }, GlobalZIndex(700), focus)).with_children(|root| {
        root.spawn((Panel { anchor, interactive }, Node {
            position_type: PositionType::Absolute, left: px(anchor.x), top: px(anchor.y),
            width: px(if interactive { 280. } else { 340. }), max_width: px((width - 16.).max(0.)),
            max_height: px((height - 16.).max(0.)), overflow: Overflow::clip(),
            padding: UiRect::all(px(6)), border: UiRect::all(px(1)),
            border_radius: BorderRadius::all(px(6)), flex_direction: FlexDirection::Column,
            row_gap: px(2), ..default()
        }, focus, BackgroundColor(Color::srgb(0.025, 0.055, 0.10)),
            BorderColor::all(Color::srgb(0.14, 0.34, 0.52))))
        .with_children(|p| {
            if let Some(field) = &e.workspace.help {
                caption(p, f, schema::field_name(field, l, lang), 16., CYAN);
                caption(p, f, schema::help(field, l, lang), 14., TEXT);
                caption(p, f, field, 12., MUTED);
            } else if let Some((_, kind)) = &e.workspace.context {
                match kind {
                    Context::Canvas => {
                        if !e.graph_stages || e.workspace.pending_link.as_ref().is_some_and(|(_, field)| field == mission_graph::REQUIRE) {
                            item(p, f, Command::NewMission, "mission.new_mission", "New mission");
                        } else {
                            item(p, f, Command::NewStage, "mission.create_element", "Create new stage");
                        }
                        item(p, f, Command::Center, "mission.center", "Center selection");
                        if e.graph_stages {
                            if let Some(row) = e.mission_row() { event_items(p, f, row); }
                        }
                    }
                    Context::Node(row) => {
                        caption(p, f, format!("ID: {}", e.rows().get(*row).map(|v| display(&v["m_iHTaskID"])).unwrap_or_default()), 12., MUTED);
                        if e.delete_confirm {
                            caption(p, f, presentation::tr(l, lang, "mission.delete_help", "Delete this stage and clear incoming task links? Shared texts, NPCs and rewards will be kept."), 14., TEXT);
                            item(p, f, Command::ConfirmDelete, "confirm_delete", "Confirm delete");
                        } else {
                            item(p, f, Command::NewStage, "mission.add_stage", "+ Stage");
                            item(p, f, Command::Duplicate, "duplicate", "Duplicate");
                            item(p, f, Command::FirstStage(*row), "mission.make_first", "Make first stage");
                            item(p, f, Command::ChooseStagePosition(*row), "mission.stage_position", "Choose stage number…");
                            item(p, f, Command::LastStage(*row), "mission.make_last", "Make last stage");
                            item(p,f,Command::EditObjective(*row),"mission.edit_objective","Edit objective title");
                            event_items(p, f, *row);
                            item(p, f, Command::CopyId, "mission.copy_id", "Copy ID");
                            item(p, f, Command::Delete, "delete", "Delete");
                        }
                    }
                    Context::StagePosition(row) => {
                        let mission=e.rows()[*row]["m_iHMissionID"].clone();
                        let count=e.rows().iter().filter(|r|r["m_iHMissionID"]==mission).count();
                        p.spawn((ScrollRegion(4),Node{max_height:px(400),overflow:Overflow::scroll_y(),flex_direction:FlexDirection::Column,..default()})).with_children(|p|{
                            for position in 0..count {
                                view::dynamic_button(p,f,Action::Mission(Command::StagePosition(*row,position)),format!("{} {}",presentation::tr(l,lang,"mission.stage_number","Stage"),position+1),false,0.);
                            }
                        });
                    }
                    Context::Mission(row) => {
                        let id = e.rows().get(*row).map(|v| display(&v["m_iHMissionID"])).unwrap_or_default();
                        caption(p, f, format!("ID: {id}"), 12., MUTED);
                        item(p, f, Command::OpenMission(*row), "mission.open_mission", "Open mission");
                        item(p, f, Command::Connect(*row, mission_graph::REQUIRE.into()), "mission.link_mission", "Unlock mission →");
                        item(p, f, Command::CopyMissionId(*row), "mission.copy_id", "Copy ID");
                    }
                    Context::Edge(row, field, slot) => {
                        if let Some(value) = e.rows().get(*row) {
                            let (from, to) = if field == mission_graph::REQUIRE {
                                (value[field].get(*slot).unwrap_or(&Value::Null), &value["m_iHMissionID"])
                            } else { (&value["m_iHTaskID"], &value[field]) };
                            caption(p, f, format!("({from}) → ({to})"), 12., MUTED);
                        }
                        if field != mission_graph::REQUIRE {
                            item(p, f, Command::InsertStage, "mission.insert", "Insert stage");
                        }
                        item(p, f, Command::RemoveEdge, "mission.remove_edge", "Remove link");
                    }
                    Context::Field(field) => {
                        item(p, f, Command::Help(field.clone()), "mission.about", "About this field");
                        if e.reference_target(field).is_some_and(|(t, _)| t != e.table) {
                            let command = e.quick_create_command(field);
                            let placeholder = matches!(command, Command::QuickPlaceholder(_));
                            item(p, f, command, if placeholder { "new_placeholder" } else { "new_record" }, if placeholder { "Create placeholder" } else { "Create record" });
                        }
                    }
                }
            }
        });
    });
}

fn event_items(p: &mut ChildSpawnerCommands, f: &EditorFonts, row: usize) {
    for (field, key, text) in [
        ("m_iSTMessageTextID", "mission.nanocom_start", "NanoCom - start"),
        ("m_iSUMessagetextID", "mission.nanocom_end", "NanoCom - end"),
        ("m_iSTDialogBubble", "mission.dialog_start", "Dialogue - start"),
        ("m_iSUDialogBubble", "mission.dialog_end", "Dialogue - end"),
    ] { item(p, f, Command::EditEvent(row, field.into()), key, text); }
    item(p,f,Command::EditEmail(row,"m_iSTMessageTextID".into(),None),"mission.email_start","Email · mission start");
    for (slot,key,text) in [(0,"edd","Email · available · Edd"),(1,"dexter","Email · available · Dexter"),(2,"mojo","Email · available · Mojo"),(3,"ben","Email · available · Ben"),(4,"computress","Email · available · Computress")] {
        item(p,f,Command::EditEmail(row,"m_iMentorEmailID".into(),Some(slot)),&format!("mission.email_{key}"),text);
    }
}

fn caption(
    p: &mut ChildSpawnerCommands,
    f: &EditorFonts,
    value: impl Into<String>,
    size: f32,
    color: Color,
) {
    let mut text = editor_text(f, "ui.editor.xdt.value", "{value}", size, color, false);
    text.localized = text.localized.with_arg("value", value.into());
    p.spawn((
        text,
        Node {
            margin: UiRect::axes(px(8), px(5)),
            flex_shrink: 0.,
            ..default()
        },
        FocusPolicy::Pass,
    ));
}

fn item(
    p: &mut ChildSpawnerCommands,
    f: &EditorFonts,
    command: Command,
    key: &str,
    fallback: &str,
) {
    p.spawn((
        Button,
        Action::Mission(command),
        view::ButtonTint(Color::NONE),
        BackgroundColor(Color::NONE),
        Node {
            width: percent(100),
            min_height: px(34),
            flex_shrink: 0.,
            padding: UiRect::axes(px(10), px(7)),
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(px(3)),
            ..default()
        },
    ))
    .with_children(|p| {
        p.spawn(editor_text(
            f,
            &format!("ui.editor.xdt.{key}"),
            fallback,
            14.,
            TEXT,
            false,
        ));
    });
}

fn clamp_anchor(anchor: Vec2, size: Vec2, window: Vec2) -> Vec2 {
    anchor
        .max(Vec2::splat(8.))
        .min((window - size - Vec2::splat(8.)).max(Vec2::splat(8.)))
}

pub(super) fn place(
    window: Single<&Window>,
    mut panels: Query<(&Panel, &ComputedNode, &mut Node)>,
) {
    for (panel, computed, mut node) in &mut panels {
        let at = clamp_anchor(
            panel.anchor,
            computed.size() / window.resolution.scale_factor(),
            Vec2::new(window.width(), window.height()),
        );
        if node.left != px(at.x) || node.top != px(at.y) {
            node.left = px(at.x);
            node.top = px(at.y);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn menu_stays_inside_each_window_corner() {
        let window = Vec2::new(1180., 720.);
        let size = Vec2::new(280., 180.);
        for anchor in [
            Vec2::ZERO,
            window,
            Vec2::new(1178., 2.),
            Vec2::new(2., 718.),
        ] {
            let at = clamp_anchor(anchor, size, window);
            assert!(at.cmpge(Vec2::splat(8.)).all());
            assert!((at + size).cmple(window - Vec2::splat(8.)).all());
        }
    }
}
