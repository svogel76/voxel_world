//! Keyboard-togglable debug overlay for day/night and render toggles.
//!
//! No egui / command parser — F1 help panel + keybinds (zero new deps).

use bevy::{
    pbr::ScreenSpaceAmbientOcclusion,
    prelude::*,
    text::FontSize,
};

use crate::day_night::{clock_label, elevation_degrees, DayCycle};
use crate::player::PlayerCamera;

#[derive(Resource, Debug, Clone, Copy)]
pub struct DebugConsole {
    pub open: bool,
    pub ssao_enabled: bool,
}

impl Default for DebugConsole {
    fn default() -> Self {
        Self {
            open: false,
            ssao_enabled: true,
        }
    }
}

#[derive(Component)]
struct DebugPanel;

#[derive(Component)]
struct DebugStatusText;

#[derive(Component)]
struct DebugHintText;

pub struct DebugConsolePlugin;

impl Plugin for DebugConsolePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugConsole>()
            .add_systems(Startup, spawn_debug_ui)
            .add_systems(
                Update,
                (
                    handle_debug_keys,
                    update_debug_visibility,
                    update_debug_status,
                )
                    .chain(),
            );
    }
}

fn spawn_debug_ui(mut commands: Commands) {
    commands.spawn((
        Name::new("DebugHint"),
        DebugHintText,
        Text::new("F1 debug"),
        TextFont {
            font_size: FontSize::Px(14.0),
            ..default()
        },
        TextColor(Color::srgb(0.65, 0.68, 0.72)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(8.0),
            left: Val::Px(8.0),
            ..default()
        },
    ));

    commands.spawn((
        Name::new("DebugPanel"),
        DebugPanel,
        Visibility::Hidden,
        Text::new(help_text()),
        TextFont {
            font_size: FontSize::Px(15.0),
            ..default()
        },
        TextColor(Color::srgb(0.88, 0.90, 0.92)),
        BackgroundColor(Color::srgba(0.04, 0.05, 0.07, 0.72)),
        Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(8.0),
            left: Val::Px(8.0),
            padding: UiRect::all(Val::Px(10.0)),
            ..default()
        },
    ))
    .with_child((
        TextSpan::new("\n"),
        TextFont {
            font_size: FontSize::Px(15.0),
            ..default()
        },
        TextColor(Color::srgb(0.95, 0.82, 0.45)),
        DebugStatusText,
    ));
}

fn help_text() -> String {
    [
        "Debug console",
        "F1 / `  toggle",
        "P       pause day cycle (physics still runs)",
        "[ ]     day speed ×0.5 / ×2",
        "T       scrub +2.4h",
        "- =     day length longer / shorter",
        "F       fog on/off",
        "O       SSAO on/off",
        "R       reset day defaults",
    ]
    .join("\n")
}

fn handle_debug_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut console: ResMut<DebugConsole>,
    mut cycle: ResMut<DayCycle>,
    mut commands: Commands,
    camera_q: Query<Entity, With<PlayerCamera>>,
) {
    if keys.just_pressed(KeyCode::F1) || keys.just_pressed(KeyCode::Backquote) {
        console.open = !console.open;
    }

    // Day / render controls work even when the panel is closed.
    if keys.just_pressed(KeyCode::KeyP) {
        cycle.paused = !cycle.paused;
    }
    if keys.just_pressed(KeyCode::BracketLeft) {
        cycle.multiply_speed(0.5);
    }
    if keys.just_pressed(KeyCode::BracketRight) {
        cycle.multiply_speed(2.0);
    }
    if keys.just_pressed(KeyCode::KeyT) {
        cycle.scrub(0.1);
    }
    if keys.just_pressed(KeyCode::Minus) {
        cycle.day_length_secs = (cycle.day_length_secs * 1.5).clamp(30.0, 3600.0);
    }
    if keys.just_pressed(KeyCode::Equal) {
        cycle.day_length_secs = (cycle.day_length_secs / 1.5).clamp(30.0, 3600.0);
    }
    if keys.just_pressed(KeyCode::KeyF) {
        cycle.fog_enabled = !cycle.fog_enabled;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        cycle.reset();
        console.ssao_enabled = true;
        if let Ok(cam) = camera_q.single() {
            commands
                .entity(cam)
                .insert(ScreenSpaceAmbientOcclusion::default());
        }
    }
    if keys.just_pressed(KeyCode::KeyO) {
        console.ssao_enabled = !console.ssao_enabled;
        if let Ok(cam) = camera_q.single() {
            if console.ssao_enabled {
                commands
                    .entity(cam)
                    .insert(ScreenSpaceAmbientOcclusion::default());
            } else {
                commands
                    .entity(cam)
                    .remove::<ScreenSpaceAmbientOcclusion>();
            }
        }
    }
}

fn update_debug_visibility(
    console: Res<DebugConsole>,
    mut panel_q: Query<&mut Visibility, With<DebugPanel>>,
    mut hint_q: Query<&mut Visibility, (With<DebugHintText>, Without<DebugPanel>)>,
) {
    if !console.is_changed() && !console.is_added() {
        return;
    }
    let panel_vis = if console.open {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    let hint_vis = if console.open {
        Visibility::Hidden
    } else {
        Visibility::Visible
    };
    for mut vis in &mut panel_q {
        *vis = panel_vis;
    }
    for mut vis in &mut hint_q {
        *vis = hint_vis;
    }
}

fn update_debug_status(
    cycle: Res<DayCycle>,
    console: Res<DebugConsole>,
    mut status_q: Query<&mut TextSpan, With<DebugStatusText>>,
) {
    if !cycle.is_changed() && !console.is_changed() && !cycle.is_added() {
        return;
    }
    let paused = if cycle.paused { "paused" } else { "running" };
    let fog = if cycle.fog_enabled { "on" } else { "off" };
    let ssao = if console.ssao_enabled { "on" } else { "off" };
    let elev = elevation_degrees(cycle.time_of_day);
    let line = format!(
        "\nclock {}  elev {elev:>5.1}°  speed ×{:.2}  day {:.0}s  ({paused})\nfog {fog}  ssao {ssao}",
        clock_label(cycle.time_of_day),
        cycle.speed,
        cycle.day_length_secs,
    );
    for mut span in &mut status_q {
        **span = line.clone();
    }
}
