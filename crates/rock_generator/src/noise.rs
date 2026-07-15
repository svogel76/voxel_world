//! Seeded 3D value noise for rock shape sampling.
//!
//! Value noise assigns a pseudo-random value to each integer lattice point,
//! then smoothly interpolates between the eight corners of the containing cell.
//! Output is in `[0, 1]` and fully determined by `(x, y, z, seed)`.

/// Sample 3D value noise at `(x, y, z)` for the given `seed`.
pub fn value_noise_3d(x: f32, y: f32, z: f32, seed: u64) -> f32 {
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let z0 = z.floor() as i32;

    let fx = x - x0 as f32;
    let fy = y - y0 as f32;
    let fz = z - z0 as f32;

    let u = smoothstep(fx);
    let v = smoothstep(fy);
    let w = smoothstep(fz);

    let n000 = hash_lattice(x0, y0, z0, seed);
    let n100 = hash_lattice(x0 + 1, y0, z0, seed);
    let n010 = hash_lattice(x0, y0 + 1, z0, seed);
    let n110 = hash_lattice(x0 + 1, y0 + 1, z0, seed);
    let n001 = hash_lattice(x0, y0, z0 + 1, seed);
    let n101 = hash_lattice(x0 + 1, y0, z0 + 1, seed);
    let n011 = hash_lattice(x0, y0 + 1, z0 + 1, seed);
    let n111 = hash_lattice(x0 + 1, y0 + 1, z0 + 1, seed);

    let nx00 = lerp(n000, n100, u);
    let nx10 = lerp(n010, n110, u);
    let nx01 = lerp(n001, n101, u);
    let nx11 = lerp(n011, n111, u);

    let nxy0 = lerp(nx00, nx10, v);
    let nxy1 = lerp(nx01, nx11, v);

    lerp(nxy0, nxy1, w)
}

fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Map lattice coordinates + seed to a value in `[0, 1)`.
fn hash_lattice(x: i32, y: i32, z: i32, seed: u64) -> f32 {
    let mut n = seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(x as u64)
        .wrapping_mul(0xBF58_476D_1CE4_E5B9)
        .wrapping_add(y as u64)
        .wrapping_mul(0x94D0_49BB_1331_11EB)
        .wrapping_add(z as u64);
    // Finalizer inspired by SplitMix64 / xxhash-style avalanche.
    n = (n ^ (n >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    n = (n ^ (n >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    n ^= n >> 31;
    (n >> 11) as f32 / ((1u64 << 53) as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_and_position_is_deterministic() {
        let a = value_noise_3d(1.25, -0.5, 3.1, 42);
        let b = value_noise_3d(1.25, -0.5, 3.1, 42);
        assert_eq!(a, b);
    }

    #[test]
    fn output_is_in_unit_interval() {
        for i in 0..20 {
            let t = i as f32 * 0.37;
            let n = value_noise_3d(t, t * 1.3, -t * 0.7, 7);
            assert!((0.0..=1.0).contains(&n), "noise {n} out of range");
        }
    }

    #[test]
    fn different_seeds_change_the_field() {
        let a = value_noise_3d(2.5, 1.0, -1.5, 1);
        let b = value_noise_3d(2.5, 1.0, -1.5, 2);
        assert_ne!(a, b);
    }
}
