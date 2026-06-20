use crate::content::io::{count_delimited_tags, count_tags};

#[test]
fn bdd_tags_delimited_ignores_method_names() {
    let text = "let todo_app = 1; let methodTODO = 2; // TODO: real one";
    let tags = count_delimited_tags(text, &["TODO"]);
    assert_eq!(tags[0].1, 1);
}

#[test]
fn bdd_tags_non_overlapping() {
    let text = "TODOTODO";
    let tags = count_tags(text, &["TODO"]);
    assert_eq!(tags[0].1, 2);
}

#[test]
fn bdd_tags_delimited_in_real_code() {
    let code = "fn main() { // TODO: fix this\n  let TODO = 5;\n}";
    let tags = count_delimited_tags(code, &["TODO"]);
    assert_eq!(tags[0].1, 2);
}
