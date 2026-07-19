//! Sky-Mini: readable day/night sky without a full skybox or weather system.
//!
//! - Inverted unlit dome follows the camera (fills the view behind terrain).
//! - ClearColor tracks the same zenith tint.
//! - Optional sun disc tracks the KeySun direction (visual only).

use bevy::{
    camera::visibility::Visibility,
    light::{NotShadowCaster, NotShadowReceiver},
    prelude::*,
    render::render_resource::Face,
};

use crate::day_night::{key_sun_color, lerp_color, solar_phase, DayCycle, LOOK_TARGET};
use crate::lighting::KeySun;
use crate::player::PlayerCamera;

const DOME_RADIUS: f32 = 450.0;
const SUN_DISTANCE: f32 = 420.0;
const SUN_RADIUS: f32 = 12.0;

#[derive(Component)]
pub(crate) struct SkyDome;

#[derive(Component)]
pub(crate) struct SkySun;

#[derive(Resource)]
pub(crate) struct SkyMaterials {
    dome: Handle<StandardMaterial>,
    sun: Handle<StandardMaterial>,
}

/// Zenith color: cool day blue → amber at dawn/dusk → near-black at night.
pub fn sky_zenith_color(day_factor: f32, horizon_glow: f32) -> Color {
    let day_sky = Color::srgb(0.35, 0.55, 0.92);
    let dusk_sky = Color::srgb(0.95, 0.45, 0.22);
    let night_sky = Color::srgb(0.02, 0.03, 0.06);
    let day_blend = lerp_color(dusk_sky, day_sky, day_factor);
    // Pull toward amber when the sun is near the horizon.
    let with_glow = lerp_color(day_blend, dusk_sky, horizon_glow * (1.0 - day_factor * 0.35));
    lerp_color(night_sky, with_glow, (day_factor * 0.85 + horizon_glow * 0.4).clamp(0.0, 1.0))
}

pub fn spawn_sky(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let dome_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.35, 0.55, 0.92),
        unlit: true,
        // Draw the inside of the sphere so we see the sky from within.
        cull_mode: Some(Face::Front),
        ..default()
    });
    let sun_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.95, 0.75),
        emissive: LinearRgba::rgb(12.0, 10.0, 4.0),
        unlit: true,
        ..default()
    });

    commands.insert_resource(SkyMaterials {
        dome: dome_mat.clone(),
        sun: sun_mat.clone(),
    });

    commands.spawn((
        Name::new("SkyDome"),
        SkyDome,
        Mesh3d(meshes.add(Sphere::new(1.0))),
        MeshMaterial3d(dome_mat),
        Transform::from_scale(Vec3::splat(DOME_RADIUS)),
        NotShadowCaster,
        NotShadowReceiver,
    ));

    commands.spawn((
        Name::new("SkySun"),
        SkySun,
        Mesh3d(meshes.add(Sphere::new(1.0))),
        MeshMaterial3d(sun_mat),
        Transform::from_scale(Vec3::splat(SUN_RADIUS)),
        NotShadowCaster,
        NotShadowReceiver,
        Visibility::Hidden,
    ));
}

/// Tint ClearColor + dome/sun materials from [`DayCycle`].
pub fn update_sky_colors(
    cycle: Res<DayCycle>,
    sky_mats: Res<SkyMaterials>,
    mut clear: ResMut<ClearColor>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let phase = solar_phase(cycle.time_of_day);
    let zenith = sky_zenith_color(phase.day_factor, phase.horizon_glow);
    clear.0 = zenith;

    if let Some(mut mat) = materials.get_mut(&sky_mats.dome) {
        mat.base_color = zenith;
    }

    let sun_color = key_sun_color(phase.day_factor, phase.horizon_glow);
    if let Some(mut mat) = materials.get_mut(&sky_mats.sun) {
        mat.base_color = sun_color;
        let s = sun_color.to_srgba();
        // Brighter emissive while the sun is up; dim near night.
        let glow = 4.0 + 10.0 * phase.day_factor;
        mat.emissive = LinearRgba::rgb(s.red * glow, s.green * glow, s.blue * glow);
    }
}

/// Keep the dome centered on the camera so the far plane always sees sky.
pub fn follow_camera_with_sky(
    camera_q: Query<&GlobalTransform, With<PlayerCamera>>,
    mut dome_q: Query<&mut Transform, (With<SkyDome>, Without<SkySun>)>,
) {
    let Ok(cam) = camera_q.single() else {
        return;
    };
    let cam_pos = cam.translation();
    if let Ok(mut dome) = dome_q.single_mut() {
        dome.translation = cam_pos;
    }
}

/// Place the visual sun along the KeySun direction from the camera.
pub fn update_sky_sun(
    cycle: Res<DayCycle>,
    camera_q: Query<&GlobalTransform, With<PlayerCamera>>,
    key_q: Query<&Transform, (With<KeySun>, Without<SkySun>, Without<SkyDome>)>,
    mut sun_q: Query<(&mut Transform, &mut Visibility), With<SkySun>>,
) {
    let Ok(cam) = camera_q.single() else {
        return;
    };
    let Ok(key) = key_q.single() else {
        return;
    };
    let Ok((mut sun_tf, mut visibility)) = sun_q.single_mut() else {
        return;
    };

    let phase = solar_phase(cycle.time_of_day);
    let sun_dir = (key.translation - LOOK_TARGET).normalize_or_zero();
    let above_horizon = key.translation.y > LOOK_TARGET.y && phase.day_factor > 0.02;

    *visibility = if above_horizon {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };

    if above_horizon && sun_dir != Vec3::ZERO {
        sun_tf.translation = cam.translation() + sun_dir * SUN_DISTANCE;
        sun_tf.scale = Vec3::splat(SUN_RADIUS);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::day_night::solar_phase;

    #[test]
    fn noon_sky_is_brighter_and_bluer_than_night() {
        let noon = solar_phase(0.5);
        let night = solar_phase(0.0);
        let noon_c = sky_zenith_color(noon.day_factor, noon.horizon_glow).to_srgba();
        let night_c = sky_zenith_color(night.day_factor, night.horizon_glow).to_srgba();
        assert!(noon_c.blue > night_c.blue);
        assert!(noon_c.red + noon_c.green + noon_c.blue > night_c.red + night_c.green + night_c.blue);
    }

    #[test]
    fn dusk_pulls_toward_warm_tones() {
        let dusk = solar_phase(0.75);
        let c = sky_zenith_color(dusk.day_factor, dusk.horizon_glow).to_srgba();
        // Horizon glow should keep red relatively high vs pure night.
        assert!(c.red > 0.15, "dusk red={}", c.red);
    }
}
