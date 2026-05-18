use anyhow::Error;

pub(super) fn missing_path_as_unrecognized_subcommand(err: &Error) -> Option<String> {
    for entry in err.chain() {
        let message = entry.to_string();
        let token = message
            .strip_prefix("Path not found: ")
            .or_else(|| message.strip_prefix("Input path does not exist: "));

        if let Some(token) = token {
            let token = token.trim();
            if looks_like_bare_subcommand_token(token) {
                return Some(token.to_string());
            }
        }
    }

    None
}

pub(super) fn looks_like_bare_subcommand_token(token: &str) -> bool {
    !token.is_empty()
        && !token.starts_with('-')
        && !token.contains('/')
        && !token.contains('\\')
        && !token.contains('.')
        && !token.contains(':')
}
