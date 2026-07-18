//! Seeded 2D value noise for moisture and placeholder terrain.
//!
//! Standalone implementation for `world_generator` — intentionally not shared
//! with `rock_generator` (crate isolation). Output is in `[0, 1]`.

/// Sample 2D value noise at `(x, z)` for the given `seed`.
pub fn value_noise_2d(x: f32, z: f32, seed: u64) -> f32 {
    let x0 = x.floor() as i32;
    let z0 = z.floor() as i32;

    let fx = x - x0 as f32;
    let fz = z - z0 as f32;

    let u = smoothstep(fx);
    let w = smoothstep(fz);

    let n00 = hash_lattice(x0, z0, seed);
    let n10 = hash_lattice(x0 + 1, z0, seed);
    let n01 = hash_lattice(x0, z0 + 1, seed);
    let n11 = hash_lattice(x0 + 1, z0 + 1, seed);

    let nx0 = lerp(n00, n10, u);
    let nx1 = lerp(n01, n11, u);
    lerp(nx0, nx1, w)
}

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Map lattice coordinates + seed to a value in `[0, 1)`.
fn hash_lattice(x: i32, z: i32, seed: u64) -> f32 {
    // Mixing constants differ from `rock_generator` on purpose (independent field).
    let mut n = seed
        .wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
        .wrapping_add(x as u64)
        .wrapping_mul(0x1656_67B1_9E37_79F9)
        .wrapping_add(z as u64);
    n = (n ^ (n >> 33)).wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    n = (n ^ (n >> 33)).wrapping_mul(0xC4CE_B9FE_1A85_EC53);
    n ^= n >> 33;
    (n >> 11) as f32 / ((1u64 << 53) as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_and_position_is_deterministic() {
        let a = value_noise_2d(3.25, -1.5, 9);
        let b = value_noise_2d(3.25, -1.5, 9);
        assert_eq!(a, b);
    }

    #[test]
    fn output_is_in_unit_interval() {
        for i in 0..20 {
            let t = i as f32 * 0.41;
            let n = value_noise_2d(t, -t * 1.2, 3);
            assert!((0.0..=1.0).contains(&n), "noise {n} out of range");
        }
    }
}
