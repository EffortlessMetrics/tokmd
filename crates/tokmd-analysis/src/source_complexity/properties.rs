#[cfg(test)]
mod tests {
    use crate::source_complexity::analyze_rust_function_complexity;
    use proptest::prelude::*;

    proptest! {
        /// Invariant: Reordering functions does not change total or max complexity.
        ///
        /// This ensures the analyzer's aggregation logic is commutative.
        #[test]
        fn property_function_order_independence(
            bodies in prop::collection::vec(
                // Generate safe, well-formed function bodies to avoid parser desync
                prop::sample::select(vec![
                    "if true { 1 } else { 2 }",
                    "match x { 1 => 1, _ => 2 }",
                    "while false { break; }",
                    "loop { break; }",
                    "for i in 0..1 { }",
                    "let x = y?",
                    "let a = b && c;",
                    "let a = b || c;",
                    "{}",
                ]),
                1..10
            )
        ) {
            // Build original code
            let mut original_code = String::new();
            for (i, body) in bodies.iter().enumerate() {
                original_code.push_str(&format!("fn f{i}() {{ {body} }}\n"));
            }

            // Build reversed code
            let mut reversed_code = String::new();
            for (i, body) in bodies.iter().rev().enumerate() {
                reversed_code.push_str(&format!("fn r{i}() {{ {body} }}\n"));
            }

            let original_analysis = analyze_rust_function_complexity(&original_code);
            let reversed_analysis = analyze_rust_function_complexity(&reversed_code);

            prop_assert_eq!(original_analysis.total_complexity, reversed_analysis.total_complexity, "Total complexity should be independent of function order");
            prop_assert_eq!(original_analysis.max_complexity, reversed_analysis.max_complexity, "Max complexity should be independent of function order");
            prop_assert_eq!(original_analysis.function_count, reversed_analysis.function_count, "Function count should be independent of function order");
        }

        /// Invariant: total_complexity is always >= max_complexity
        #[test]
        fn property_total_gte_max(
            code in "([a-zA-Z0-9 \\{\\}\n\t\\?&\\|=><]+)"
        ) {
            let analysis = analyze_rust_function_complexity(&code);
            prop_assert!(
                analysis.total_complexity >= analysis.max_complexity,
                "total_complexity ({}) must be >= max_complexity ({})",
                analysis.total_complexity,
                analysis.max_complexity
            );
        }
    }
}
