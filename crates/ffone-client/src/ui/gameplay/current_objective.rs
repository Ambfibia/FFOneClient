//! Current objective panel model, spawning and localized binding.

use super::assets::GameplayUiAssets;
use super::hud::GameplayUiRect;
use super::model::GameplayMenuTransition;
use super::model::GameplayUiModel;
use super::text::hud_font;
use crate::localization::{
    Language, Localization, LocalizedText, UiTextAutoFit, localized_tabledata_npc_name,
    localized_tabledata_quest_item_name, localized_tutorial_mission_text,
};
use bevy::{prelude::*, text::LineHeight};

/// Exact closed-Nanocom current-objective area. `cnGUINanocom` starts from
/// serialized `MenuRect(0,-7,178,294)`, offsets it by the live
/// `WindowRect(7,-4,187,183)`, then trims thirty pixels on both axes.
pub const CURRENT_OBJECTIVE_RECT: GameplayUiRect = GameplayUiRect::new(29.0, 186.0, 148.0, 264.0);

/// Selected active mission copy drawn below the minimap while the Nanocom
/// menu is closed and the Current Objective display option is enabled.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CurrentObjectiveUi {
    pub visible: bool,
    /// Stable tutorial TableData task identity used to localize the title and
    /// objective independently from their English fallback copy.
    pub task_id: Option<i32>,
    pub title: String,
    pub body: String,
    pub remaining_time_seconds: Option<i64>,
    pub enemies: Vec<CurrentObjectiveProgressUi>,
    pub quest_items: Vec<CurrentObjectiveProgressUi>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CurrentObjectiveProgressUi {
    pub content_id: i32,
    pub name: String,
    pub complete: i32,
    pub needed: i32,
}

#[derive(Component)]
pub(super) struct CurrentObjectiveRoot;
#[derive(Component)]
pub(super) struct CurrentObjectiveTitle;
#[derive(Component)]
pub(super) struct CurrentObjectiveBody;

pub(super) fn spawn_current_objective(
    parent: &mut ChildSpawnerCommands,
    assets: &GameplayUiAssets,
) {
    parent
        .spawn((
            Node {
                display: Display::None,
                flex_direction: FlexDirection::Column,
                row_gap: px(10),
                overflow: Overflow::clip(),
                ..CURRENT_OBJECTIVE_RECT.node()
            },
            CurrentObjectiveRoot,
            ZIndex(8),
        ))
        .with_children(|objective| {
            let title_font = hud_font(&assets.jeffe_font, 14.0);
            let title_fit = UiTextAutoFit::new(
                CURRENT_OBJECTIVE_RECT.width,
                70.0,
                &(title_font.clone(), LineHeight::default()),
            );
            // `FusionFallHUDSkin.centerbox`: JEFFE___14, UpperLeft,
            // word-wrap, zero padding. The clean client draws a second pass
            // one pixel up/left, which is equivalent to a +1/+1 black shadow.
            objective.spawn((
                Node {
                    width: percent(100),
                    flex_shrink: 0.0,
                    ..default()
                },
                Text::new(""),
                LocalizedText::new("ui.hud.current_objective.title", "{title}")
                    .with_arg("title", ""),
                title_font,
                title_fit,
                TextColor(Color::srgb(1.0, 0.921_568_6, 0.015_686_28)),
                TextLayout::new(Justify::Left, LineBreak::WordBoundary),
                TextShadow {
                    offset: Vec2::splat(1.0),
                    color: Color::BLACK,
                },
                CurrentObjectiveTitle,
            ));
            let body_font = hud_font(&assets.chalet_font, 12.0);
            let body_fit = UiTextAutoFit::new(
                CURRENT_OBJECTIVE_RECT.width,
                184.0,
                &(body_font.clone(), LineHeight::default()),
            );
            // `FusionFallHUDSkin.imagewindow`: the skin's default
            // ChaletBook-Regular Small, UpperLeft, word-wrap, zero padding.
            objective.spawn((
                Node {
                    width: percent(100),
                    flex_shrink: 0.0,
                    ..default()
                },
                Text::new(""),
                LocalizedText::new("ui.hud.current_objective.body", "{objective}{progress}")
                    .with_arg("objective", "")
                    .with_arg("progress", ""),
                body_font,
                body_fit,
                TextColor(Color::WHITE),
                TextLayout::new(Justify::Left, LineBreak::WordBoundary),
                TextShadow {
                    offset: Vec2::splat(1.0),
                    color: Color::BLACK,
                },
                CurrentObjectiveBody,
            ));
        });
}

pub(super) fn localized_current_objective_body(
    objective: &CurrentObjectiveUi,
    localization: &Localization,
    language: &Language,
) -> LocalizedText {
    let base = objective.task_id.map_or_else(
        || objective.body.clone(),
        |task_id| {
            localization.text(
                language,
                &localized_tutorial_mission_text(task_id, "objective", &objective.body),
            )
        },
    );
    let mut progress = String::new();
    if let Some(remaining) = objective.remaining_time_seconds {
        let label = localization.text(
            language,
            &LocalizedText::new("ui.hud.current_objective.remaining_time", "Remaining time"),
        );
        progress.push_str(&format!("\n{label}: {remaining}"));
    }
    if !objective.enemies.is_empty() {
        progress.push('\n');
        for row in &objective.enemies {
            let name = localization.text(
                language,
                &localized_tabledata_npc_name(row.content_id, &row.name),
            );
            progress.push_str(&format!("{name} : {}/{}\n", row.complete, row.needed));
        }
    }
    if !objective.quest_items.is_empty() {
        progress.push('\n');
        for row in &objective.quest_items {
            let name = localization.text(
                language,
                &localized_tabledata_quest_item_name(row.content_id, &row.name),
            );
            progress.push_str(&format!("{name} : {}/{}\n", row.complete, row.needed));
        }
    }
    LocalizedText::new("ui.hud.current_objective.body", "{objective}{progress}")
        .with_arg("objective", base)
        .with_arg("progress", progress)
}

pub(super) fn bind_current_objective(
    model: Res<GameplayUiModel>,
    transition: Res<GameplayMenuTransition>,
    localization: Res<Localization>,
    language: Res<Language>,
    mut root: Single<&mut Node, With<CurrentObjectiveRoot>>,
    mut title: Single<
        &mut LocalizedText,
        (With<CurrentObjectiveTitle>, Without<CurrentObjectiveBody>),
    >,
    mut body: Single<
        &mut LocalizedText,
        (With<CurrentObjectiveBody>, Without<CurrentObjectiveTitle>),
    >,
) {
    if !model.is_changed() && !transition.is_changed() && !language.is_changed() {
        return;
    }
    // This is the closed-NanoCom objective copy. Its rectangle overlaps the
    // sliding right menu, whose background is partly transparent.
    root.display = if model.visible
        && model.current_objective.visible
        && !transition.visible_or_transitioning()
    {
        Display::Flex
    } else {
        Display::None
    };

    let localized_title = model.current_objective.task_id.map_or_else(
        || {
            LocalizedText::new("ui.hud.current_objective.title", "{title}")
                .with_arg("title", &model.current_objective.title)
        },
        |task_id| localized_tutorial_mission_text(task_id, "title", &model.current_objective.title),
    );
    **title = localized_title;

    **body = localized_current_objective_body(&model.current_objective, &localization, &language);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn objective_copy_tracks_right_menu_through_repeated_enter_transitions() {
        let asset_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/game");
        for locale in ["en", "ru"] {
            let (localization, language) = Localization::open(&asset_root, locale).unwrap();
            for scale in [0.8, 1.0, 1.25] {
                let mut app = App::new();
                let mut model = GameplayUiModel::default();
                model.visible = true;
                model.ui_scale = scale;
                model.current_objective.visible = true;
                let (title, body) = if locale == "ru" {
                    (
                        "Длинное название задания, которое переносится на несколько строк",
                        "Длинное описание цели задания и прогресса, которое занимает несколько строк под меню",
                    )
                } else {
                    (
                        "A long mission title that wraps across the objective panel",
                        "A long objective with several lines of progress that extends behind the NanoCom menu",
                    )
                };
                model.current_objective.title = title.into();
                model.current_objective.body = body.into();
                app.insert_resource(model)
                    .insert_resource(GameplayMenuTransition::default())
                    .insert_resource(localization.clone())
                    .insert_resource(language.clone())
                    .add_systems(Update, bind_current_objective);
                let objective = app
                    .world_mut()
                    .spawn((Node::default(), CurrentObjectiveRoot))
                    .id();
                app.world_mut()
                    .spawn((LocalizedText::new("", ""), CurrentObjectiveTitle));
                app.world_mut()
                    .spawn((LocalizedText::new("", ""), CurrentObjectiveBody));

                let display = |app: &App| app.world().get::<Node>(objective).unwrap().display;
                app.update();
                assert_eq!(
                    display(&app),
                    Display::Flex,
                    "{locale}/{scale}: closed menu"
                );
                for _ in 0..3 {
                    app.world_mut()
                        .resource_mut::<GameplayMenuTransition>()
                        .set_open(true);
                    app.update();
                    assert_eq!(
                        display(&app),
                        Display::None,
                        "{locale}/{scale}: opening/open menu"
                    );
                    app.world_mut()
                        .resource_mut::<GameplayMenuTransition>()
                        .set_open(false);
                    app.update();
                    assert_eq!(
                        display(&app),
                        Display::None,
                        "{locale}/{scale}: closing menu"
                    );
                    app.world_mut()
                        .resource_mut::<GameplayMenuTransition>()
                        .advance(0.5);
                    app.update();
                    assert_eq!(
                        display(&app),
                        Display::Flex,
                        "{locale}/{scale}: settled closed menu"
                    );
                }
            }
        }
    }
}
