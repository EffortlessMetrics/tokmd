with open("crates/tokmd-analysis/tests/feature_gates_w71.rs", "r") as f:
    lines = f.readlines()

# Instead of changing to `let receipt`, let's just suppress the unused warning for those test cases when specific features are disabled, or fix the unused warnings.
# Let's inspect line 175 and below
for i in range(len(lines)):
    if "let _receipt" in lines[i]:
        lines[i] = lines[i].replace("let _receipt", "let receipt")
    if "let receipt" in lines[i]:
        # we can just allow unused variable if we want
        lines[i] = "    #[allow(unused_variables)]\n" + lines[i]

with open("crates/tokmd-analysis/tests/feature_gates_w71.rs", "w") as f:
    f.writelines(lines)
