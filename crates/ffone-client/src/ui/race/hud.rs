//! Small race clock and confirmed pod counter, separate from the end result.
use crate::localization::{LocalizationSet, LocalizedText};
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Default, PartialEq, Resource)]
pub struct RaceHudState {
    pub awaiting_start: bool,
    pub running: bool,
    pub elapsed_seconds: i32,
    pub remaining_seconds: i32,
    pub pods: i32,
    pub confirmed_pickups: usize,
}

#[derive(Component)]
struct RaceHudRoot;

#[derive(Component)]
struct RaceHudCopy;

#[derive(Component)]
struct RaceHudPod;

#[derive(Default)]
struct PodFeedback {
    confirmed_pickups: usize,
    remaining: f32,
}

#[derive(SystemSet, Clone, Debug, Hash, Eq, PartialEq)]
pub enum RaceHudSet {
    Bind,
}

pub struct RaceHudPlugin;

impl Plugin for RaceHudPlugin {
    fn build(&self, app: &mut App) {
        crate::ui_startup::init_native_ui_startup_phase(app);
        app.init_resource::<RaceHudState>()
            .add_systems(crate::ui_startup::NativeGameplayUiStartup, spawn_race_hud)
            .add_systems(
                Update,
                bind_race_hud
                    .in_set(RaceHudSet::Bind)
                    .before(LocalizationSet::Apply)
                    .in_set(crate::ui_startup::NativeUiStartupSet),
            );
    }
}

fn spawn_race_hud(mut commands: Commands, assets: Res<AssetServer>) {
    let copy = LocalizedText::new("ui.race.hud.ready", "RACE READY");
    commands
        .spawn((
            RaceHudRoot,
            Node {
                position_type: PositionType::Absolute,
                right: px(210),
                top: px(16),
                width: px(300),
                height: px(48),
                display: Display::None,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                column_gap: px(8),
                ..default()
            },
            BackgroundColor(Color::srgba(0.03, 0.08, 0.13, 0.82)),
            GlobalZIndex(8_100),
            Pickable::IGNORE,
        ))
        .with_children(|root| {
            root.spawn((
                RaceHudPod,
                Node { width: px(24), height: px(28), border: UiRect::all(px(2)),
                    border_radius: BorderRadius::all(percent(50)), ..default() },
                BorderColor::all(Color::srgba(0.4, 1.0, 0.35, 0.85)),
                BackgroundColor(Color::srgb(0.05, 0.55, 0.12)),
                Pickable::IGNORE,
            )).with_children(|pod| {
                pod.spawn((
                    Node { position_type: PositionType::Absolute, left: px(4), top: px(4),
                        width: px(7), height: px(7), border_radius: BorderRadius::all(percent(50)), ..default() },
                    BackgroundColor(Color::srgba(0.8, 1.0, 0.6, 0.85)),
                    Pickable::IGNORE,
                ));
            });
            root.spawn((
                RaceHudCopy,
                Text::new("RACE READY"),
                copy,
                TextFont {
                    font: assets.load(super::RACE_JEFFE_FONT_PATH).into(),
                    font_size: 18.0.into(),
                    ..default()
                },
                TextColor(Color::WHITE),
                TextLayout::new(Justify::Center, LineBreak::NoWrap),
                Pickable::IGNORE,
            ));
        });
}

fn bind_race_hud(
    state: Res<RaceHudState>,
    time: Res<Time>,
    mut feedback: Local<PodFeedback>,
    mut roots: Query<&mut Node, With<RaceHudRoot>>,
    mut pods: Query<&mut Node, (With<RaceHudPod>, Without<RaceHudRoot>)>,
    mut copy: Query<&mut LocalizedText, With<RaceHudCopy>>,
) {
    feedback.remaining = (feedback.remaining - time.delta_secs()).max(0.0);
    if state.running && state.confirmed_pickups > feedback.confirmed_pickups {
        feedback.remaining = 1.0;
    }
    feedback.confirmed_pickups = state.confirmed_pickups;
    if !state.running { feedback.remaining = 0.0; }
    for mut node in &mut pods {
        let offset = (feedback.remaining * std::f32::consts::TAU * 5.0).sin()
            * feedback.remaining * 5.0;
        let left = px(offset);
        let display = if state.running { Display::Flex } else { Display::None };
        if node.left != left { node.left = left; }
        if node.display != display { node.display = display; }
    }
    for mut node in &mut roots {
        let display = if state.awaiting_start || state.running {
            Display::Flex
        } else {
            Display::None
        };
        if node.display != display { node.display = display; }
    }
    let value = if state.running {
        let seconds = state.elapsed_seconds.max(0);
        LocalizedText::new("ui.race.hud.progress", "TIME {time}  PODS {pods}")
            .with_arg("time", format!("{:02}:{:02}", seconds / 60, seconds % 60))
            .with_arg("pods", state.pods.to_string())
    } else {
        LocalizedText::new("ui.race.hud.ready", "RACE READY")
    };
    for mut text in &mut copy {
        if *text != value { *text = value.clone(); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ready_running_and_finished_states_bind_one_localized_panel() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(AssetPlugin::default())
            .init_asset::<Font>()
            .init_asset::<Image>()
            .add_plugins(RaceHudPlugin);
        app.update();
        let world = app.world_mut();
        let mut root = world.query_filtered::<&Node, With<RaceHudRoot>>();
        assert_eq!(root.single(world).unwrap().display, Display::None);

        world.resource_mut::<RaceHudState>().awaiting_start = true;
        app.update();
        let world = app.world_mut();
        assert_eq!(root.single(world).unwrap().display, Display::Flex);

        *world.resource_mut::<RaceHudState>() = RaceHudState {
            running: true,
            elapsed_seconds: 65,
            pods: 3,
            ..default()
        };
        app.update();
        let world = app.world_mut();
        let mut copy = world.query_filtered::<&LocalizedText, With<RaceHudCopy>>();
        let copy = copy.single(world).unwrap();
        assert_eq!(copy.key, "ui.race.hud.progress");
        assert_eq!(copy.args.get("time").map(String::as_str), Some("01:05"));
        assert_eq!(copy.args.get("pods").map(String::as_str), Some("3"));

        app.insert_resource(bevy::time::TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_millis(50),
        ));
        app.world_mut().resource_mut::<RaceHudState>().confirmed_pickups = 1;
        app.update();
        app.update();
        let world = app.world_mut();
        let mut pod = world.query_filtered::<&Node, With<RaceHudPod>>();
        assert_ne!(pod.single(world).unwrap().left, px(0));
        for _ in 0..22 { app.update(); }
        let world = app.world_mut();
        assert_eq!(pod.single(world).unwrap().left, px(0));

        *world.resource_mut::<RaceHudState>() = RaceHudState::default();
        app.update();
        let world = app.world_mut();
        assert_eq!(root.single(world).unwrap().display, Display::None);
    }
}
