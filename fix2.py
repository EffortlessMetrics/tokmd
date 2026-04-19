import re

content = open("crates/tokmd/src/error_hints.rs", "r").read()

search = r"""    if haystack.contains("unknown metric/finding key") {
        if !haystack.contains("use --explain list") {
            push_hint(
                &mut out,
                "Run `tokmd analyze --explain list` to see supported keys.",
            );
        }
    }"""

replace = r"""    if haystack.contains("unknown metric/finding key") && !haystack.contains("use --explain list") {
        push_hint(
            &mut out,
            "Run `tokmd analyze --explain list` to see supported keys.",
        );
    }"""

if search in content:
    content = content.replace(search, replace)
    with open("crates/tokmd/src/error_hints.rs", "w") as f:
        f.write(content)
    print("Replaced!")
else:
    print("Search string not found")
