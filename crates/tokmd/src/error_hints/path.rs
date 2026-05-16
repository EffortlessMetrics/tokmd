use anyhow::Error;

pub(super) fn missing_path_as_unrecognized_subcommand(err: &Error) -> Option<String> {
    err.chain().find_map(|entry| {
        let message = entry.to_string();
        missing_path_token(&message)
            .and_then(|token| looks_like_bare_subcommand_token(token).then(|| token.to_string()))
    })
}

pub(super) fn extract_missing_path(err: &Error) -> Option<String> {
    err.chain().find_map(|entry| {
        let message = entry.to_string();
        missing_path_token(&message).map(ToString::to_string)
    })
}

pub(super) fn looks_like_bare_subcommand_token(token: &str) -> bool {
    !token.is_empty()
        && !token.starts_with('-')
        && !token.contains('/')
        && !token.contains('\\')
        && !token.contains('.')
        && !token.contains(':')
}

fn missing_path_token(message: &str) -> Option<&str> {
    message
        .strip_prefix("Path not found: ")
        .or_else(|| message.strip_prefix("Input path does not exist: "))
        .map(str::trim)
}
