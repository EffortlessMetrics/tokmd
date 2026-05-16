use super::edit_distance::levenshtein;

const KNOWN_SUBCOMMANDS: &[&str] = &[
    "lang",
    "module",
    "export",
    "analyze",
    "badge",
    "init",
    "completions",
    "run",
    "diff",
    "context",
    "check-ignore",
    "tools",
    "gate",
    "cockpit",
    "baseline",
    "handoff",
    "sensor",
];

pub(super) fn closest_known_subcommand(token: &str) -> Option<&'static str> {
    let (candidate, distance) = KNOWN_SUBCOMMANDS
        .iter()
        .map(|known| (*known, levenshtein(token, known)))
        .min_by_key(|(_, distance)| *distance)?;

    let threshold = std::cmp::max(2, candidate.len() / 3);
    (distance <= threshold && distance > 0).then_some(candidate)
}
