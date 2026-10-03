//! Constant-field GPU regression for the production glow graph.
//! A normalized blur preserves field energy; alpha=1 suppresses glow entirely.
use bevy::{
    core_pipeline::tonemapping::Tonemapping,
    prelude::*,
    render::view::screenshot::{Screenshot, ScreenshotCaptured},
    window::WindowResolution,
};
use ffone_client::legacy_glow::{LegacyGlowPlugin, LegacyGlowSettings};

#[derive(Resource)]
struct Probe {
    frames: u32,
    alpha: f32,
    intensity: f32,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let alpha = args.get(1).map(|s| s.parse().unwrap()).unwrap_or(0.0);
    let intensity = args.get(2).map(|s| s.parse().unwrap()).unwrap_or(1.8);
    App::new()
        .insert_resource(Probe {
            frames: 0,
            alpha,
            intensity,
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Glow energy regression".into(),
                resolution: WindowResolution::new(256, 256),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(LegacyGlowPlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, capture)
        .run();
}

fn setup(mut commands: Commands, probe: Res<Probe>) {
    let mut glow = LegacyGlowSettings::default();
    glow.parameters.x = probe.intensity;
    commands.spawn((
        Camera3d::default(),
        Camera {
            clear_color: Color::srgba(0.2, 0.4, 0.6, probe.alpha).into(),
            ..default()
        },
        Tonemapping::None,
        Msaa::Off,
        glow,
    ));
}

fn capture(mut commands: Commands, mut probe: ResMut<Probe>) {
    probe.frames += 1;
    assert!(probe.frames < 600, "GPU capture timed out");
    if probe.frames == 60 {
        commands.spawn(Screenshot::primary_window()).observe(verify);
    }
}

fn verify(event: On<ScreenshotCaptured>, probe: Res<Probe>, mut exit: MessageWriter<AppExit>) {
    let image = event.image.clone().try_into_dynamic().unwrap().to_rgba8();
    let pixel = image.get_pixel(128, 128).0;
    for (channel, source) in [0.2_f32, 0.4, 0.6].into_iter().enumerate() {
        let expected = ((source
            + source * (20.0 / 255.0) * (1.0 - probe.alpha) * 2.0 * probe.intensity.max(0.0))
        .clamp(0.0, 1.0)
            * 255.0)
            .round();
        assert!(
            (pixel[channel] as f32 - expected).abs() <= 4.0,
            "glow energy mismatch: pixel={pixel:?}, channel={channel}, expected={expected}"
        );
    }
    let output = format!(
        "target/performance/glow-energy/alpha-{}-intensity-{}.png",
        probe.alpha, probe.intensity
    );
    std::fs::create_dir_all("target/performance/glow-energy").unwrap();
    image.save(&output).unwrap();
    eprintln!(
        "PASS alpha={} intensity={} pixel={pixel:?}; {output}",
        probe.alpha, probe.intensity
    );
    exit.write(AppExit::Success);
}
