## Target Evaluation
1. Fuzzers found that using `.unwrap_or(args)` allows invalid configuration shapes.
2. Specifically, if `lang` configuration is provided as a string instead of an object, it ignores the string and parses the root config, which masks an invalid type.
3. This was superseded by another PR, so we are producing a learning PR.
