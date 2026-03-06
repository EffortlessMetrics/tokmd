with open("crates/tokmd-git/tests/git_w74.rs", "r") as f:
    content = f.read()

new_content = content.replace("""    match collect_history(&repo.path, Some(2), None) {
        Ok(commits) => assert!(commits.len() <= 2, "max_commits should limit results"),
        Err(_) => {} // broken pipe on early close is known behaviour
    }""", """    if let Ok(commits) = collect_history(&repo.path, Some(2), None) {
        assert!(commits.len() <= 2, "max_commits should limit results");
    }""")

with open("crates/tokmd-git/tests/git_w74.rs", "w") as f:
    f.write(new_content)
