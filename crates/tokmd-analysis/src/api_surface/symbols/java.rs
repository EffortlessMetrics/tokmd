use super::{Symbol, has_doc_comment};

pub(super) fn extract_symbols(lines: &[&str]) -> Vec<Symbol> {
    let mut symbols = Vec::new();

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();

        if trimmed.starts_with("//") || trimmed.starts_with('*') || trimmed.starts_with("/*") {
            continue;
        }

        let is_public = is_public(trimmed);
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

fn is_public(trimmed: &str) -> bool {
    trimmed.starts_with("public class ")
        || trimmed.starts_with("public interface ")
        || trimmed.starts_with("public enum ")
        || trimmed.starts_with("public static ")
        || trimmed.starts_with("public abstract class ")
        || trimmed.starts_with("public final class ")
        || trimmed.starts_with("public record ")
        || trimmed.starts_with("public sealed ")
        // public return-type method(
        || (trimmed.starts_with("public ")
            && (trimmed.contains('(') || trimmed.contains(" class ") || trimmed.contains(" interface ")))
}

fn is_internal(trimmed: &str) -> bool {
    // private/protected/package-private items
    trimmed.starts_with("private ")
        || trimmed.starts_with("protected ")
        || trimmed.starts_with("class ")
        || trimmed.starts_with("interface ")
        || trimmed.starts_with("enum ")
        || trimmed.starts_with("abstract class ")
        || trimmed.starts_with("final class ")
        || trimmed.starts_with("static ")
        || trimmed.starts_with("record ")
}
