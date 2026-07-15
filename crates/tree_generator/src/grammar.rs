use std::collections::HashMap;

use rand::Rng;

/// One or more weighted replacements for an L-system symbol.
#[derive(Debug, Clone, PartialEq)]
pub struct ProductionRule {
    alternatives: Vec<(String, f32)>,
}

impl ProductionRule {
    pub fn deterministic(replacement: impl Into<String>) -> Self {
        Self {
            alternatives: vec![(replacement.into(), 1.0)],
        }
    }

    pub fn stochastic(alternatives: Vec<(impl Into<String>, f32)>) -> Self {
        Self {
            alternatives: alternatives
                .into_iter()
                .map(|(replacement, weight)| (replacement.into(), weight))
                .collect(),
        }
    }

    fn deterministic_replacement(&self) -> &str {
        self.alternatives
            .first()
            .map(|(replacement, _)| replacement.as_str())
            .unwrap_or("")
    }
}

/// L-system grammar with deterministic or weighted stochastic productions.
#[derive(Debug, Clone, PartialEq)]
pub struct LSystemGrammar {
    axiom: String,
    rules: HashMap<char, ProductionRule>,
}

impl LSystemGrammar {
    pub fn new(axiom: &str, rules: HashMap<char, String>) -> Self {
        Self::with_rules(
            axiom,
            rules
                .into_iter()
                .map(|(symbol, replacement)| {
                    (symbol, ProductionRule::deterministic(replacement))
                })
                .collect(),
        )
    }

    pub fn with_rules(axiom: &str, rules: HashMap<char, ProductionRule>) -> Self {
        Self {
            axiom: axiom.to_string(),
            rules,
        }
    }

    /// Expand the axiom for `depth` iterations. Depth 0 returns the axiom unchanged.
    /// Always uses the first alternative of each rule (deterministic).
    pub fn expand(&self, depth: u32) -> String {
        if depth == 0 {
            return self.axiom.clone();
        }

        let mut current = self.axiom.clone();
        for _ in 0..depth {
            current = expand_once_deterministic(&current, &self.rules);
        }
        current
    }

    /// Expand with weighted random rule selection. Requires an explicit RNG
    /// (e.g. `StdRng::seed_from_u64(seed)`) for reproducibility.
    pub fn expand_random(&self, depth: u32, rng: &mut impl Rng) -> String {
        if depth == 0 {
            return self.axiom.clone();
        }

        let mut current = self.axiom.clone();
        for _ in 0..depth {
            current = expand_once_random(&current, &self.rules, rng);
        }
        current
    }
}

fn expand_once_deterministic(current: &str, rules: &HashMap<char, ProductionRule>) -> String {
    let mut result = String::with_capacity(current.len());
    for ch in current.chars() {
        match rules.get(&ch) {
            Some(rule) => result.push_str(rule.deterministic_replacement()),
            None => result.push(ch),
        }
    }
    result
}

fn expand_once_random(
    current: &str,
    rules: &HashMap<char, ProductionRule>,
    rng: &mut impl Rng,
) -> String {
    let mut result = String::with_capacity(current.len());
    for ch in current.chars() {
        match rules.get(&ch) {
            Some(rule) => result.push_str(pick_alternative(rule, rng)),
            None => result.push(ch),
        }
    }
    result
}

fn pick_alternative<'a>(rule: &'a ProductionRule, rng: &mut impl Rng) -> &'a str {
    let positive: Vec<_> = rule
        .alternatives
        .iter()
        .filter(|(_, weight)| *weight > 0.0)
        .collect();

    if positive.is_empty() {
        return "";
    }

    let total: f32 = positive.iter().map(|(_, weight)| weight).sum();
    let mut roll = rng.gen_range(0.0..total);

    for (replacement, weight) in &positive {
        roll -= weight;
        if roll <= 0.0 {
            return replacement.as_str();
        }
    }

    positive
        .last()
        .map(|(replacement, _)| replacement.as_str())
        .unwrap_or("")
}

#[cfg(test)]
mod tests {
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    use super::*;

    fn rules(pairs: &[(&str, &str)]) -> HashMap<char, String> {
        pairs
            .iter()
            .map(|(symbol, replacement)| (symbol.chars().next().unwrap(), replacement.to_string()))
            .collect()
    }

    fn stochastic_rules(pairs: &[(&str, Vec<(&str, f32)>)]) -> HashMap<char, ProductionRule> {
        pairs
            .iter()
            .map(|(symbol, alternatives)| {
                (
                    symbol.chars().next().unwrap(),
                    ProductionRule::stochastic(
                        alternatives
                            .iter()
                            .map(|(replacement, weight)| (*replacement, *weight))
                            .collect(),
                    ),
                )
            })
            .collect()
    }

    #[test]
    fn depth_zero_returns_axiom_unchanged() {
        let grammar = LSystemGrammar::new("F", rules(&[("F", "FF")]));
        assert_eq!(grammar.expand(0), "F");
        assert_eq!(grammar.expand_random(0, &mut StdRng::seed_from_u64(1)), "F");
    }

    #[test]
    fn single_iteration_replaces_symbols() {
        let grammar = LSystemGrammar::new("F", rules(&[("F", "F[+F]F")]));
        assert_eq!(grammar.expand(1), "F[+F]F");
    }

    #[test]
    fn multiple_iterations_apply_rules_repeatedly() {
        let grammar = LSystemGrammar::new("A", rules(&[("A", "AB"), ("B", "A")]));
        assert_eq!(grammar.expand(1), "AB");
        assert_eq!(grammar.expand(2), "ABA");
        assert_eq!(grammar.expand(3), "ABAAB");
    }

    #[test]
    fn empty_rules_leave_symbols_unchanged() {
        let grammar = LSystemGrammar::new("ABC", HashMap::new());
        assert_eq!(grammar.expand(1), "ABC");
        assert_eq!(grammar.expand(5), "ABC");
    }

    #[test]
    fn partial_rules_only_replace_matching_symbols() {
        let grammar = LSystemGrammar::new("ABC", rules(&[("A", "AA")]));
        assert_eq!(grammar.expand(1), "AABC");
    }

    #[test]
    fn empty_axiom_stays_empty() {
        let grammar = LSystemGrammar::new("", rules(&[("F", "FF")]));
        assert_eq!(grammar.expand(0), "");
        assert_eq!(grammar.expand(3), "");
    }

    #[test]
    fn bracket_symbols_are_treated_like_any_other_character() {
        let grammar = LSystemGrammar::new("F[+F]", rules(&[("F", "FF")]));
        assert_eq!(grammar.expand(1), "FF[+FF]");
    }

    #[test]
    fn unbalanced_brackets_pass_through_without_error() {
        let grammar = LSystemGrammar::new("F[+F", rules(&[("F", "G")]));
        assert_eq!(grammar.expand(1), "G[+G");
    }

    #[test]
    fn expand_random_is_reproducible_for_same_seed() {
        let grammar = LSystemGrammar::with_rules(
            "F",
            stochastic_rules(&[("F", vec![("A", 1.0), ("B", 1.0)])]),
        );

        let first = grammar.expand_random(4, &mut StdRng::seed_from_u64(42));
        let second = grammar.expand_random(4, &mut StdRng::seed_from_u64(42));

        assert_eq!(first, second);
        assert_ne!(first, "F");
    }

    #[test]
    fn expand_random_differs_for_different_seeds() {
        let grammar = LSystemGrammar::with_rules(
            "F",
            stochastic_rules(&[("F", vec![("A", 1.0), ("B", 1.0)])]),
        );

        let first = grammar.expand_random(6, &mut StdRng::seed_from_u64(1));
        let second = grammar.expand_random(6, &mut StdRng::seed_from_u64(2));

        assert_ne!(first, second);
    }

    #[test]
    fn expand_random_matches_deterministic_for_single_alternative() {
        let grammar = LSystemGrammar::with_rules(
            "F",
            stochastic_rules(&[("F", vec![("F[+F]F", 1.0)])]),
        );

        assert_eq!(
            grammar.expand(2),
            grammar.expand_random(2, &mut StdRng::seed_from_u64(99))
        );
    }

    #[test]
    fn expand_random_respects_relative_weights() {
        let grammar = LSystemGrammar::with_rules(
            "F",
            stochastic_rules(&[("F", vec![("A", 3.0), ("B", 1.0)])]),
        );

        let output = grammar.expand_random(1, &mut StdRng::seed_from_u64(7));
        assert_eq!(output, "A");
    }
}
