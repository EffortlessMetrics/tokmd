with open("crates/tokmd-context-git/tests/deep_w67.rs", "r") as f:
    content = f.read()

content = content.replace("sorted.sort_by(|a, b| b.1.cmp(&a.1));", "sorted.sort_by_key(|b| std::cmp::Reverse(b.1));")

with open("crates/tokmd-context-git/tests/deep_w67.rs", "w") as f:
    f.write(content)
