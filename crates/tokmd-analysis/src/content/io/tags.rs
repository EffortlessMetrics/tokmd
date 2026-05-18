//! Tag counting helpers for content analysis.

pub(super) fn count_tags(text: &str, tags: &[&str]) -> Vec<(String, usize)> {
    let upper = text.to_uppercase();
    tags.iter()
        .map(|tag| {
            let needle = tag.to_uppercase();
            let count = count_non_overlapping_matches(&upper, &needle);
            (tag.to_string(), count)
        })
        .collect()
}

pub(super) fn count_delimited_tags(text: &str, tags: &[&str]) -> Vec<(String, usize)> {
    let upper = text.to_uppercase();
    tags.iter()
        .map(|tag| {
            let needle = tag.to_uppercase();
            let count = count_delimited_matches(&upper, &needle);
            (tag.to_string(), count)
        })
        .collect()
}

fn count_non_overlapping_matches(haystack: &str, needle: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }

    haystack.matches(needle).count()
}

fn count_delimited_matches(haystack: &str, needle: &str) -> usize {
    if needle.is_empty() {
        return 0;
    }

    let mut count = 0;
    let mut start = 0;
    while let Some(offset) = haystack[start..].find(needle) {
        let idx = start + offset;
        let next = idx + needle.len();
        if is_delimited_match(haystack, idx, next) {
            count += 1;
        }
        start = next;
    }
    count
}

fn is_delimited_match(text: &str, start: usize, end: usize) -> bool {
    let prev_delimited = text[..start]
        .chars()
        .next_back()
        .is_none_or(|ch| !is_tag_continuation(ch));
    let next_delimited = text[end..]
        .chars()
        .next()
        .is_none_or(|ch| !is_tag_continuation(ch));
    prev_delimited && next_delimited
}

fn is_tag_continuation(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_delimited_tags_counts_standalone_tags() {
        let result = count_delimited_tags(
            "// TODO: one\n// todo(two)\n// TODO-list\n// FIXME/XXX\n",
            &["TODO", "FIXME", "XXX"],
        );

        assert_eq!(result[0], ("TODO".to_string(), 3));
        assert_eq!(result[1], ("FIXME".to_string(), 1));
        assert_eq!(result[2], ("XXX".to_string(), 1));
    }

    #[test]
    fn count_delimited_tags_ignores_identifier_like_substrings() {
        let result =
            count_delimited_tags("todo_app TODO1 methodTODO TODOS // TODO: real", &["TODO"]);

        assert_eq!(result[0], ("TODO".to_string(), 1));
    }

    #[test]
    fn count_tags_treats_empty_tag_as_zero_matches() {
        let result = count_tags("TODO FIXME", &["", "TODO"]);

        assert_eq!(result[0], (String::new(), 0));
        assert_eq!(result[1], ("TODO".to_string(), 1));
    }

    #[test]
    fn count_delimited_tags_treats_unicode_alphanumerics_as_identifier_boundaries() {
        let result = count_delimited_tags("éTODO TODOé αTODO TODOβ // TODO: real", &["TODO"]);

        assert_eq!(result[0], ("TODO".to_string(), 1));
    }
}
