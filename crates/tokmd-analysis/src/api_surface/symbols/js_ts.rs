use super::{Symbol, has_doc_comment};

pub(super) fn extract_symbols(lines: &[&str]) -> Vec<Symbol> {
    let mut symbols = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("//") || trimmed.starts_with('*') || trimmed.starts_with("/*") {
            continue;
        }

        let is_public = is_export(trimmed);
        let is_internal = !is_public && is_internal(trimmed);

        if is_public || is_internal {
            symbols.push(Symbol {
                is_public,
                is_documented: has_doc_comment(lines, i),
            });
        }
    }

    symbols
}

fn is_export(trimmed: &str) -> bool {
    trimmed.starts_with("export function ")
        || trimmed.starts_with("export async function ")
        || trimmed.starts_with("export class ")
        || trimmed.starts_with("export const ")
        || trimmed.starts_with("export let ")
        || trimmed.starts_with("export default ")
        || trimmed.starts_with("export interface ")
        || trimmed.starts_with("export type ")
        || trimmed.starts_with("export enum ")
        || trimmed.starts_with("export abstract class ")
}

fn is_internal(trimmed: &str) -> bool {
    trimmed.starts_with("function ")
        || trimmed.starts_with("async function ")
        || trimmed.starts_with("class ")
        || trimmed.starts_with("const ")
        || trimmed.starts_with("let ")
        || trimmed.starts_with("interface ")
        || trimmed.starts_with("type ")
        || trimmed.starts_with("enum ")
}
