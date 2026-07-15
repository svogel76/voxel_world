use std::collections::HashMap;

use rand::rngs::StdRng;
use rand::SeedableRng;

use glam::IVec3;

use crate::grammar::{LSystemGrammar, ProductionRule};
use crate::turtle::{interpret_with_rng, TurtleJitter, TurtleParams};
use crate::types::BlockType;
use crate::voxelize::{add_leaves, voxelize_with_shape, CrossSectionShape, LeafPlacement};

/// Full parameter set for [`generate`].
#[derive(Debug, Clone, PartialEq)]
pub struct TreeParams {
    pub axiom: String,
    pub rules: HashMap<char, ProductionRule>,
    pub depth: u32,
    pub turtle: TurtleParams,
    pub jitter: TurtleJitter,
    pub cross_section: CrossSectionShape,
    pub leaf_placement: LeafPlacement,
}

impl Default for TreeParams {
    fn default() -> Self {
        Self::generic_2d()
    }
}

impl TreeParams {
    /// Reference 2D biome preset (README Phase 5 / `visualize.rs` trees 1–3).
    pub fn generic_2d() -> Self {
        Self {
            axiom: "F".to_string(),
            rules: stochastic_2d_rules(),
            depth: 4,
            turtle: TurtleParams {
                step_length: 1.0,
                angle_degrees: 25.0,
                base_thickness: 2.0,
                taper_ratio: 0.72,
            },
            jitter: TurtleJitter::TREE_DEFAULT,
            cross_section: CrossSectionShape::Cube,
            leaf_placement: LeafPlacement::default(),
        }
    }

    /// Reference 3D biome preset (README Phase 5 / `visualize.rs` tree 4).
    pub fn generic_3d() -> Self {
        Self {
            axiom: "F".to_string(),
            rules: stochastic_3d_rules(),
            depth: 3,
            turtle: TurtleParams {
                step_length: 1.0,
                angle_degrees: 28.0,
                base_thickness: 1.8,
                taper_ratio: 0.7,
            },
            jitter: TurtleJitter::TREE_DEFAULT,
            cross_section: CrossSectionShape::Cube,
            leaf_placement: LeafPlacement::default(),
        }
    }
}

/// Generate a complete tree as voxel positions with block types.
///
/// Pipeline: `StdRng::seed_from_u64(seed)` → `expand_random` →
/// `interpret_with_rng` → `voxelize_with_shape` → `add_leaves`.
pub fn generate(seed: u64, params: &TreeParams) -> Vec<(IVec3, BlockType)> {
    let grammar = LSystemGrammar::with_rules(&params.axiom, params.rules.clone());
    let mut rng = StdRng::seed_from_u64(seed);

    let l_string = grammar.expand_random(params.depth, &mut rng);
    let segments = interpret_with_rng(
        &l_string,
        &params.turtle,
        &params.jitter,
        &mut rng,
    );
    let wood = voxelize_with_shape(&segments, params.cross_section);
    add_leaves(&wood, &segments, params.leaf_placement)
}

fn stochastic_2d_rules() -> HashMap<char, ProductionRule> {
    HashMap::from([(
        'F',
        ProductionRule::stochastic(vec![
            ("F[+F]F[-F]F", 3.0),
            ("F[+F][-F]F", 2.0),
            ("F[+F]F", 1.0),
        ]),
    )])
}

fn stochastic_3d_rules() -> HashMap<char, ProductionRule> {
    HashMap::from([(
        'F',
        ProductionRule::stochastic(vec![
            ("F[+F][&F][-F][^F]F", 2.0),
            ("F[+F][&F][-F]F", 2.0),
            ("F[+F][&F][^F]F", 1.0),
        ]),
    )])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn voxel_fingerprint(voxels: &[(IVec3, BlockType)]) -> u64 {
        let mut hash = voxels.len() as u64;
        for (position, block_type) in voxels {
            hash = hash
                .wrapping_mul(1_000_003)
                .wrapping_add(position.x as u64)
                .wrapping_mul(1_000_003)
                .wrapping_add(position.y as u64)
                .wrapping_mul(1_000_003)
                .wrapping_add(position.z as u64)
                .wrapping_mul(1_000_003)
                .wrapping_add(match block_type {
                    BlockType::Wood => 0,
                    BlockType::Leaf => 1,
                });
        }
        hash
    }

    fn preview_tree_one_params() -> TreeParams {
        TreeParams {
            turtle: TurtleParams {
                step_length: 1.0,
                angle_degrees: 22.0,
                base_thickness: 4.0,
                taper_ratio: 0.72,
            },
            cross_section: CrossSectionShape::Cube,
            depth: 4,
            ..TreeParams::generic_2d()
        }
    }

    #[test]
    fn generate_is_deterministic_for_same_seed_and_params() {
        let params = preview_tree_one_params();
        let first = generate(1, &params);
        let second = generate(1, &params);

        assert_eq!(first, second);
    }

    #[test]
    fn preview_tree_one_matches_legacy_visualize_fingerprint() {
        let params = preview_tree_one_params();
        let offset = IVec3::new(-27, 0, 0);
        let voxels = generate(1, &params)
            .into_iter()
            .map(|(position, block_type)| (position + offset, block_type))
            .collect::<Vec<_>>();

        assert_eq!(voxel_fingerprint(&voxels), 15758200525907297904);
    }

    #[test]
    fn generate_differs_for_different_seeds() {
        let params = TreeParams::generic_2d();

        assert_ne!(generate(1, &params), generate(2, &params));
    }

    #[test]
    fn generate_produces_wood_and_leaf_voxels_end_to_end() {
        let voxels = generate(42, &TreeParams::generic_2d());

        assert!(voxels.iter().any(|(_, block_type)| *block_type == BlockType::Wood));
        assert!(voxels.iter().any(|(_, block_type)| *block_type == BlockType::Leaf));
    }

    #[test]
    fn default_params_produce_non_empty_tree() {
        let voxels = generate(7, &TreeParams::default());

        assert!(!voxels.is_empty());
    }

    #[test]
    fn generic_3d_preset_produces_non_empty_tree() {
        let voxels = generate(4, &TreeParams::generic_3d());

        assert!(!voxels.is_empty());
        assert!(voxels.iter().any(|(_, block_type)| *block_type == BlockType::Leaf));
    }
}
