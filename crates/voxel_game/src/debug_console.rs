//! Keyboard-togglable debug overlay for day/night and render toggles.
//!
//! No egui / command parser — F1 help panel + keybinds (zero new deps).
//! Keybind state changes live in [`apply_debug_action`] so they stay unit-testable
//! without spinning up a full Bevy `App`.

use bevy::{
    pbr::ScreenSpaceAmbientOcclusion,
    prelude::*,
    text::FontSize,
};

use crate::day_night::{clock_label, elevation_degrees, DayCycle};
use crate::player::PlayerCamera;

const DAY_LENGTH_MIN_SECS: f32 = 30.0;
const DAY_LENGTH_MAX_SECS: f32 = 3600.0;
const DAY_LENGTH_STEP: f32 = 1.5;
const SCRUB_STEP: f32 = 0.1;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
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

/// Pure debug actions (mapped from keybinds in [`handle_debug_keys`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugAction {
    ToggleOpen,
    TogglePause,
    SpeedHalf,
    SpeedDouble,
    ScrubForward,
    DayLonger,
    DayShorter,
    ToggleFog,
    ToggleSsao,
    Reset,
}

/// Side-effect request for SSAO component insert/remove on the player camera.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SsaoRequest {
    None,
    Enable,
    Disable,
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

    commands
        .spawn((
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

/// Apply one debug action to console + day cycle state.
///
/// Returns whether the player camera should gain/lose SSAO. Bevy entity work
/// stays in [`handle_debug_keys`].
pub fn apply_debug_action(
    action: DebugAction,
    console: &mut DebugConsole,
    cycle: &mut DayCycle,
) -> SsaoRequest {
    match action {
        DebugAction::ToggleOpen => {
            console.open = !console.open;
            SsaoRequest::None
        }
        DebugAction::TogglePause => {
            cycle.paused = !cycle.paused;
            SsaoRequest::None
        }
        DebugAction::SpeedHalf => {
            cycle.multiply_speed(0.5);
            SsaoRequest::None
        }
        DebugAction::SpeedDouble => {
            cycle.multiply_speed(2.0);
            SsaoRequest::None
        }
        DebugAction::ScrubForward => {
            cycle.scrub(SCRUB_STEP);
            SsaoRequest::None
        }
        DebugAction::DayLonger => {
            cycle.day_length_secs = adjust_day_length(cycle.day_length_secs, true);
            SsaoRequest::None
        }
        DebugAction::DayShorter => {
            cycle.day_length_secs = adjust_day_length(cycle.day_length_secs, false);
            SsaoRequest::None
        }
        DebugAction::ToggleFog => {
            cycle.fog_enabled = !cycle.fog_enabled;
            SsaoRequest::None
        }
        DebugAction::ToggleSsao => {
            console.ssao_enabled = !console.ssao_enabled;
            if console.ssao_enabled {
                SsaoRequest::Enable
            } else {
                SsaoRequest::Disable
            }
        }
        DebugAction::Reset => {
            cycle.reset();
            console.ssao_enabled = true;
            SsaoRequest::Enable
        }
    }
}

fn adjust_day_length(secs: f32, longer: bool) -> f32 {
    let next = if longer {
        secs * DAY_LENGTH_STEP
    } else {
        secs / DAY_LENGTH_STEP
    };
    next.clamp(DAY_LENGTH_MIN_SECS, DAY_LENGTH_MAX_SECS)
}

fn status_line(cycle: &DayCycle, console: &DebugConsole) -> String {
    let paused = if cycle.paused { "paused" } else { "running" };
    let fog = if cycle.fog_enabled { "on" } else { "off" };
    let ssao = if console.ssao_enabled { "on" } else { "off" };
    let elev = elevation_degrees(cycle.time_of_day);
    format!(
        "\nclock {}  elev {elev:>5.1}°  speed ×{:.2}  day {:.0}s  ({paused})\nfog {fog}  ssao {ssao}",
        clock_label(cycle.time_of_day),
        cycle.speed,
        cycle.day_length_secs,
    )
}

fn panel_and_hint_visibility(open: bool) -> (Visibility, Visibility) {
    if open {
        (Visibility::Visible, Visibility::Hidden)
    } else {
        (Visibility::Hidden, Visibility::Visible)
    }
}

fn handle_debug_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut console: ResMut<DebugConsole>,
    mut cycle: ResMut<DayCycle>,
    mut commands: Commands,
    camera_q: Query<Entity, With<PlayerCamera>>,
) {
    let mut actions = Vec::new();
    if keys.just_pressed(KeyCode::F1) || keys.just_pressed(KeyCode::Backquote) {
        actions.push(DebugAction::ToggleOpen);
    }
    if keys.just_pressed(KeyCode::KeyP) {
        actions.push(DebugAction::TogglePause);
    }
    if keys.just_pressed(KeyCode::BracketLeft) {
        actions.push(DebugAction::SpeedHalf);
    }
    if keys.just_pressed(KeyCode::BracketRight) {
        actions.push(DebugAction::SpeedDouble);
    }
    if keys.just_pressed(KeyCode::KeyT) {
        actions.push(DebugAction::ScrubForward);
    }
    if keys.just_pressed(KeyCode::Minus) {
        actions.push(DebugAction::DayLonger);
    }
    if keys.just_pressed(KeyCode::Equal) {
        actions.push(DebugAction::DayShorter);
    }
    if keys.just_pressed(KeyCode::KeyF) {
        actions.push(DebugAction::ToggleFog);
    }
    if keys.just_pressed(KeyCode::KeyR) {
        actions.push(DebugAction::Reset);
    }
    if keys.just_pressed(KeyCode::KeyO) {
        actions.push(DebugAction::ToggleSsao);
    }

    for action in actions {
        let ssao = apply_debug_action(action, &mut console, &mut cycle);
        apply_ssao_request(ssao, &mut commands, &camera_q);
    }
}

fn apply_ssao_request(
    request: SsaoRequest,
    commands: &mut Commands,
    camera_q: &Query<Entity, With<PlayerCamera>>,
) {
    let Ok(cam) = camera_q.single() else {
        return;
    };
    match request {
        SsaoRequest::None => {}
        SsaoRequest::Enable => {
            commands
                .entity(cam)
                .insert(ScreenSpaceAmbientOcclusion::default());
        }
        SsaoRequest::Disable => {
            commands
                .entity(cam)
                .remove::<ScreenSpaceAmbientOcclusion>();
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
    let (panel_vis, hint_vis) = panel_and_hint_visibility(console.open);
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
    let line = status_line(&cycle, &console);
    for mut span in &mut status_q {
        **span = line.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::day_night::{DEFAULT_DAY_LENGTH_SECS, DEFAULT_TIME_OF_DAY};

    #[test]
    fn toggle_open_flips_flag() {
        let mut console = DebugConsole::default();
        let mut cycle = DayCycle::default();
        assert!(!console.open);
        apply_debug_action(DebugAction::ToggleOpen, &mut console, &mut cycle);
        assert!(console.open);
        apply_debug_action(DebugAction::ToggleOpen, &mut console, &mut cycle);
        assert!(!console.open);
    }

    #[test]
    fn pause_and_fog_toggles_do_not_touch_ssao() {
        let mut console = DebugConsole::default();
        let mut cycle = DayCycle::default();
        assert_eq!(
            apply_debug_action(DebugAction::TogglePause, &mut console, &mut cycle),
            SsaoRequest::None
        );
        assert!(cycle.paused);
        assert_eq!(
            apply_debug_action(DebugAction::ToggleFog, &mut console, &mut cycle),
            SsaoRequest::None
        );
        assert!(!cycle.fog_enabled);
        assert!(console.ssao_enabled);
    }

    #[test]
    fn speed_and_scrub_actions_mutate_cycle() {
        let mut console = DebugConsole::default();
        let mut cycle = DayCycle {
            time_of_day: 0.2,
            speed: 1.0,
            ..DayCycle::default()
        };
        apply_debug_action(DebugAction::SpeedDouble, &mut console, &mut cycle);
        assert!((cycle.speed - 2.0).abs() < 1e-5);
        apply_debug_action(DebugAction::SpeedHalf, &mut console, &mut cycle);
        assert!((cycle.speed - 1.0).abs() < 1e-5);
        apply_debug_action(DebugAction::ScrubForward, &mut console, &mut cycle);
        assert!((cycle.time_of_day - 0.3).abs() < 1e-5);
    }

    #[test]
    fn day_length_adjust_clamps() {
        assert!((adjust_day_length(600.0, true) - 900.0).abs() < 1e-3);
        assert!((adjust_day_length(600.0, false) - 400.0).abs() < 1e-3);
        let mut secs = DAY_LENGTH_MAX_SECS;
        for _ in 0..5 {
            secs = adjust_day_length(secs, true);
        }
        assert_eq!(secs, DAY_LENGTH_MAX_SECS);
        secs = DAY_LENGTH_MIN_SECS;
        for _ in 0..5 {
            secs = adjust_day_length(secs, false);
        }
        assert_eq!(secs, DAY_LENGTH_MIN_SECS);
    }

    #[test]
    fn day_length_actions_use_clamp() {
        let mut console = DebugConsole::default();
        let mut cycle = DayCycle::default();
        apply_debug_action(DebugAction::DayLonger, &mut console, &mut cycle);
        assert!(cycle.day_length_secs > DEFAULT_DAY_LENGTH_SECS);
        apply_debug_action(DebugAction::DayShorter, &mut console, &mut cycle);
        assert!((cycle.day_length_secs - DEFAULT_DAY_LENGTH_SECS).abs() < 1e-2);
    }

    #[test]
    fn ssao_toggle_and_reset_request_camera_updates() {
        let mut console = DebugConsole::default();
        let mut cycle = DayCycle::default();
        assert_eq!(
            apply_debug_action(DebugAction::ToggleSsao, &mut console, &mut cycle),
            SsaoRequest::Disable
        );
        assert!(!console.ssao_enabled);
        assert_eq!(
            apply_debug_action(DebugAction::ToggleSsao, &mut console, &mut cycle),
            SsaoRequest::Enable
        );
        console.ssao_enabled = false;
        cycle.paused = true;
        cycle.time_of_day = 0.9;
        assert_eq!(
            apply_debug_action(DebugAction::Reset, &mut console, &mut cycle),
            SsaoRequest::Enable
        );
        assert!(console.ssao_enabled);
        assert_eq!(cycle.time_of_day, DEFAULT_TIME_OF_DAY);
        assert!(!cycle.paused);
    }

    #[test]
    fn status_line_includes_clock_and_flags() {
        let console = DebugConsole {
            open: true,
            ssao_enabled: false,
        };
        let cycle = DayCycle {
            time_of_day: 0.5,
            paused: true,
            fog_enabled: false,
            speed: 4.0,
            day_length_secs: 120.0,
            ..DayCycle::default()
        };
        let line = status_line(&cycle, &console);
        assert!(line.contains("12:00"));
        assert!(line.contains("paused"));
        assert!(line.contains("fog off"));
        assert!(line.contains("ssao off"));
        assert!(line.contains("×4.00"));
        assert!(line.contains("day 120s"));
    }

    #[test]
    fn panel_visibility_hides_hint_when_open() {
        let (panel, hint) = panel_and_hint_visibility(true);
        assert_eq!(panel, Visibility::Visible);
        assert_eq!(hint, Visibility::Hidden);
        let (panel, hint) = panel_and_hint_visibility(false);
        assert_eq!(panel, Visibility::Hidden);
        assert_eq!(hint, Visibility::Visible);
    }

    #[test]
    fn help_text_lists_core_binds() {
        let help = help_text();
        assert!(help.contains("F1"));
        assert!(help.contains("pause"));
        assert!(help.contains("SSAO"));
    }
}
