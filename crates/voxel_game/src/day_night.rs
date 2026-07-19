//! Configurable day/night cycle driving Phase-2 key/fill lights and mood.

use std::f32::consts::TAU;

use bevy::{
    light::FogVolume,
    prelude::*,
};

use crate::lighting::{CoolFill, KeySun, SceneFogVolume};

/// Real-time length of one full day/night cycle (10 minutes).
pub const DEFAULT_DAY_LENGTH_SECS: f32 = 600.0;
/// Late morning — close to the static Phase-2 noon mood at first frame.
pub const DEFAULT_TIME_OF_DAY: f32 = 0.35;

pub(crate) const LOOK_TARGET: Vec3 = Vec3::new(0.0, 4.0, 0.0);
const ORBIT_RADIUS: f32 = 45.0;
const NOON_KEY_ILLUMINANCE: f32 = 80_000.0;
const NOON_FILL_ILLUMINANCE: f32 = 5_500.0;
const NOON_AMBIENT: f32 = 55.0;
const NIGHT_AMBIENT: f32 = 10.0;
const NOON_FOG_DENSITY: f32 = 0.08;

/// Progress of the sun through a day. Independent of `Time<Virtual>` so pausing
/// the cycle does not freeze physics or player movement.
#[derive(Resource, Debug, Clone)]
pub struct DayCycle {
    /// `0.0` midnight, `0.25` sunrise, `0.5` noon, `0.75` sunset.
    pub time_of_day: f32,
    /// Real seconds for one full cycle.
    pub day_length_secs: f32,
    pub paused: bool,
    /// Multiplier on day advance only (`1.0` = realtime vs `day_length_secs`).
    pub speed: f32,
    /// When false, fog density is forced to `0` (debug toggle).
    pub fog_enabled: bool,
    /// Baseline fog density restored when fog is re-enabled.
    pub fog_density: f32,
}

impl Default for DayCycle {
    fn default() -> Self {
        Self {
            time_of_day: DEFAULT_TIME_OF_DAY,
            day_length_secs: DEFAULT_DAY_LENGTH_SECS,
            paused: false,
            speed: 1.0,
            fog_enabled: true,
            fog_density: NOON_FOG_DENSITY,
        }
    }
}

impl DayCycle {
    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn scrub(&mut self, delta: f32) {
        self.time_of_day = (self.time_of_day + delta).rem_euclid(1.0);
    }

    pub fn multiply_speed(&mut self, factor: f32) {
        self.speed = (self.speed * factor).clamp(0.125, 32.0);
    }

    /// Advance `time_of_day` by `delta_secs` of real time (no-op when paused).
    pub fn advance(&mut self, delta_secs: f32) {
        if self.paused || self.day_length_secs <= 0.0 || delta_secs <= 0.0 {
            return;
        }
        let step = delta_secs * self.speed / self.day_length_secs;
        self.time_of_day = (self.time_of_day + step).rem_euclid(1.0);
    }
}

pub struct DayNightPlugin;

impl Plugin for DayNightPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DayCycle>().add_systems(
            Update,
            (advance_day_cycle, apply_day_night).chain(),
        );
    }
}

fn advance_day_cycle(time: Res<Time>, mut cycle: ResMut<DayCycle>) {
    cycle.advance(time.delta_secs());
}

fn apply_day_night(
    cycle: Res<DayCycle>,
    mut ambient: ResMut<GlobalAmbientLight>,
    mut key_q: Query<(&mut Transform, &mut DirectionalLight), With<KeySun>>,
    mut fill_q: Query<&mut DirectionalLight, (With<CoolFill>, Without<KeySun>)>,
    mut fog_q: Query<&mut FogVolume, With<SceneFogVolume>>,
) {
    let phase = solar_phase(cycle.time_of_day);
    let day = phase.day_factor;
    let dawn_dusk = phase.horizon_glow;

    if let Ok((mut transform, mut light)) = key_q.single_mut() {
        *transform = sun_transform(cycle.time_of_day);
        light.illuminance = key_illuminance(day);
        light.color = key_sun_color(day, dawn_dusk);
        light.shadow_maps_enabled = shadows_enabled(day);
    }

    if let Ok(mut fill) = fill_q.single_mut() {
        fill.illuminance = fill_illuminance(day);
        fill.color = Color::srgb(
            0.35 + 0.05 * day,
            0.42 + 0.06 * day,
            0.55 + 0.05 * day,
        );
    }

    ambient.brightness = ambient_brightness(day);
    ambient.color = Color::srgb(
        0.14 + 0.08 * day,
        0.18 + 0.08 * day,
        0.28 + 0.04 * day,
    );

    // ClearColor / sky dome tint: `sky::update_sky_colors`.

    if let Ok(mut fog) = fog_q.single_mut() {
        fog.density_factor = fog_density_factor(&cycle, dawn_dusk);
        fog.fog_color = Color::srgb(
            0.75 + 0.17 * day,
            0.80 + 0.14 * day,
            0.90 + 0.07 * day,
        );
    }
}

fn key_illuminance(day_factor: f32) -> f32 {
    NOON_KEY_ILLUMINANCE * day_factor.powf(1.35)
}

fn fill_illuminance(day_factor: f32) -> f32 {
    // Keep a readable night fill so silhouettes do not disappear.
    NOON_FILL_ILLUMINANCE * (0.25 + 0.75 * day_factor)
}

fn ambient_brightness(day_factor: f32) -> f32 {
    NIGHT_AMBIENT + (NOON_AMBIENT - NIGHT_AMBIENT) * day_factor
}

fn shadows_enabled(day_factor: f32) -> bool {
    day_factor > 0.05
}

fn fog_density_factor(cycle: &DayCycle, dawn_dusk: f32) -> f32 {
    if cycle.fog_enabled {
        // Slightly thicker shafts at dawn/dusk.
        cycle.fog_density * (0.85 + 0.35 * dawn_dusk)
    } else {
        0.0
    }
}

pub(crate) struct SolarPhase {
    pub day_factor: f32,
    pub horizon_glow: f32,
}

pub(crate) fn solar_phase(time_of_day: f32) -> SolarPhase {
    // -1 at midnight, 0 at sunrise/sunset, +1 at noon.
    let elev = -(time_of_day * TAU).cos();
    let day_factor = ((elev + 0.12) / 1.12).clamp(0.0, 1.0);
    // Peaks when elevation crosses the horizon.
    let horizon_glow = (1.0 - elev.abs()).clamp(0.0, 1.0).powf(1.5);
    SolarPhase {
        day_factor,
        horizon_glow,
    }
}

fn sun_transform(time_of_day: f32) -> Transform {
    let angle = time_of_day * TAU;
    let pos = LOOK_TARGET
        + Vec3::new(
            angle.sin() * ORBIT_RADIUS,
            -angle.cos() * ORBIT_RADIUS,
            angle.cos() * ORBIT_RADIUS * 0.35,
        );
    Transform::from_translation(pos).looking_at(LOOK_TARGET, Vec3::Y)
}

pub(crate) fn key_sun_color(day: f32, dawn_dusk: f32) -> Color {
    // Warm noon, amber near horizon, cool when night takes over.
    let warm = Color::srgb(1.0, 0.96, 0.88);
    let amber = Color::srgb(1.0, 0.55, 0.28);
    let cool = Color::srgb(0.55, 0.62, 0.85);
    let day_color = lerp_color(amber, warm, day);
    lerp_color(cool, day_color, (day + dawn_dusk * 0.5).clamp(0.0, 1.0))
}

pub(crate) fn lerp_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let a = a.to_srgba();
    let b = b.to_srgba();
    Color::srgba(
        a.red + (b.red - a.red) * t,
        a.green + (b.green - a.green) * t,
        a.blue + (b.blue - a.blue) * t,
        1.0,
    )
}

/// Human-readable clock label for the debug overlay (`HH:MM` style).
pub fn clock_label(time_of_day: f32) -> String {
    let minutes = (time_of_day.rem_euclid(1.0) * 24.0 * 60.0).round() as u32 % (24 * 60);
    let h = minutes / 60;
    let m = minutes % 60;
    format!("{h:02}:{m:02}")
}

/// Rough solar elevation in degrees for the debug readout (−90…+90).
pub fn elevation_degrees(time_of_day: f32) -> f32 {
    let elev = -(time_of_day * TAU).cos();
    elev.asin().to_degrees().clamp(-90.0, 90.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noon_is_brightest() {
        let noon = solar_phase(0.5).day_factor;
        let midnight = solar_phase(0.0).day_factor;
        let sunrise = solar_phase(0.25).day_factor;
        assert!(noon > 0.95, "noon day_factor={noon}");
        assert!(midnight < 0.05, "midnight day_factor={midnight}");
        assert!(sunrise < 0.2, "sunrise day_factor={sunrise}");
    }

    #[test]
    fn horizon_glow_peaks_near_sunrise_and_sunset() {
        let sunrise = solar_phase(0.25).horizon_glow;
        let sunset = solar_phase(0.75).horizon_glow;
        let noon = solar_phase(0.5).horizon_glow;
        let midnight = solar_phase(0.0).horizon_glow;
        assert!(sunrise > noon);
        assert!(sunset > noon);
        assert!(sunrise > midnight);
    }

    #[test]
    fn scrub_wraps_unit_interval() {
        let mut cycle = DayCycle::default();
        cycle.time_of_day = 0.95;
        cycle.scrub(0.1);
        assert!((cycle.time_of_day - 0.05).abs() < 1e-5);
    }

    #[test]
    fn multiply_speed_is_clamped() {
        let mut cycle = DayCycle::default();
        cycle.speed = 1.0;
        cycle.multiply_speed(0.5);
        assert!((cycle.speed - 0.5).abs() < 1e-5);
        for _ in 0..20 {
            cycle.multiply_speed(0.5);
        }
        assert!((cycle.speed - 0.125).abs() < 1e-5);
        for _ in 0..20 {
            cycle.multiply_speed(2.0);
        }
        assert!((cycle.speed - 32.0).abs() < 1e-5);
    }

    #[test]
    fn reset_restores_defaults() {
        let mut cycle = DayCycle::default();
        cycle.time_of_day = 0.9;
        cycle.speed = 8.0;
        cycle.paused = true;
        cycle.fog_enabled = false;
        cycle.day_length_secs = 60.0;
        cycle.reset();
        assert_eq!(cycle.time_of_day, DEFAULT_TIME_OF_DAY);
        assert_eq!(cycle.day_length_secs, DEFAULT_DAY_LENGTH_SECS);
        assert!(!cycle.paused);
        assert!((cycle.speed - 1.0).abs() < 1e-5);
        assert!(cycle.fog_enabled);
    }

    #[test]
    fn advance_respects_pause_and_speed() {
        let mut running = DayCycle {
            time_of_day: 0.0,
            day_length_secs: 100.0,
            speed: 1.0,
            paused: false,
            ..DayCycle::default()
        };
        running.advance(10.0);
        assert!((running.time_of_day - 0.1).abs() < 1e-5);

        let mut paused = running.clone();
        paused.paused = true;
        paused.advance(50.0);
        assert!((paused.time_of_day - 0.1).abs() < 1e-5);

        let mut fast = DayCycle {
            time_of_day: 0.0,
            day_length_secs: 100.0,
            speed: 2.0,
            paused: false,
            ..DayCycle::default()
        };
        fast.advance(10.0);
        assert!((fast.time_of_day - 0.2).abs() < 1e-5);
    }

    #[test]
    fn advance_wraps_past_midnight() {
        let mut cycle = DayCycle {
            time_of_day: 0.95,
            day_length_secs: 100.0,
            speed: 1.0,
            paused: false,
            ..DayCycle::default()
        };
        cycle.advance(10.0); // +0.1 → wraps to 0.05
        assert!((cycle.time_of_day - 0.05).abs() < 1e-5);
    }

    #[test]
    fn clock_label_formats_key_times() {
        assert_eq!(clock_label(0.5), "12:00");
        assert_eq!(clock_label(0.0), "00:00");
        assert_eq!(clock_label(0.25), "06:00");
        assert_eq!(clock_label(0.75), "18:00");
    }

    #[test]
    fn elevation_degrees_match_solar_extremes() {
        let noon = elevation_degrees(0.5);
        let midnight = elevation_degrees(0.0);
        assert!(noon > 80.0, "noon elev={noon}");
        assert!(midnight < -80.0, "midnight elev={midnight}");
    }

    #[test]
    fn sun_is_high_at_noon() {
        let t = sun_transform(0.5);
        assert!(t.translation.y > LOOK_TARGET.y + 20.0);
        let night = sun_transform(0.0);
        assert!(night.translation.y < LOOK_TARGET.y);
    }

    #[test]
    fn lighting_scales_with_day_factor() {
        let noon = solar_phase(0.5).day_factor;
        let midnight = solar_phase(0.0).day_factor;
        assert!(key_illuminance(noon) > key_illuminance(midnight) * 10.0);
        assert!(fill_illuminance(noon) > fill_illuminance(midnight));
        assert!(ambient_brightness(noon) > ambient_brightness(midnight));
        assert!(shadows_enabled(noon));
        assert!(!shadows_enabled(midnight));
    }

    #[test]
    fn fog_density_zero_when_disabled() {
        let mut cycle = DayCycle::default();
        let dawn = solar_phase(0.25).horizon_glow;
        assert!(fog_density_factor(&cycle, dawn) > 0.0);
        cycle.fog_enabled = false;
        assert_eq!(fog_density_factor(&cycle, dawn), 0.0);
    }

    #[test]
    fn fog_thicker_at_dawn_than_noon() {
        let cycle = DayCycle::default();
        let dawn = fog_density_factor(&cycle, solar_phase(0.25).horizon_glow);
        let noon = fog_density_factor(&cycle, solar_phase(0.5).horizon_glow);
        assert!(dawn > noon);
    }
}
