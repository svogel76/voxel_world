use std::collections::HashMap;

/// Deterministic L-system grammar: one axiom and at most one production per symbol.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LSystemGrammar {
    axiom: String,
    rules: HashMap<char, String>,
}

impl LSystemGrammar {
    pub fn new(axiom: &str, rules: HashMap<char, String>) -> Self {
        Self {
            axiom: axiom.to_string(),
            rules,
        }
    }

    /// Expand the axiom for `depth` iterations. Depth 0 returns the axiom unchanged.
    /// Symbols without a production rule are copied verbatim.
    pub fn expand(&self, depth: u32) -> String {
        if depth == 0 {
            return self.axiom.clone();
        }

        let mut current = self.axiom.clone();
        for _ in 0..depth {
            current = expand_once(&current, &self.rules);
        }
        current
    }
}

fn expand_once(current: &str, rules: &HashMap<char, String>) -> String {
    let mut result = String::with_capacity(current.len());
    for ch in current.chars() {
        match rules.get(&ch) {
            Some(replacement) => result.push_str(replacement),
            None => result.push(ch),
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules(pairs: &[(&str, &str)]) -> HashMap<char, String> {
        pairs
            .iter()
            .map(|(symbol, replacement)| (symbol.chars().next().unwrap(), replacement.to_string()))
            .collect()
    }

    #[test]
    fn depth_zero_returns_axiom_unchanged() {
        let grammar = LSystemGrammar::new("F", rules(&[("F", "FF")]));
        assert_eq!(grammar.expand(0), "F");
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
}
