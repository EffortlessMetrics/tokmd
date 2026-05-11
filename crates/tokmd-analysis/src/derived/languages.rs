use std::collections::BTreeMap;

use tokmd_analysis_types::{LangPurityReport, LangPurityRow, PolyglotReport};
use tokmd_scan::{round_f64, safe_ratio};
use tokmd_types::FileRow;

pub(super) fn build_lang_purity_report(rows: &[&FileRow]) -> LangPurityReport {
    let mut by_module: BTreeMap<&str, BTreeMap<&str, usize>> = BTreeMap::new();

    for row in rows {
        let entry = if let Some(existing) = by_module.get_mut(row.module.as_str()) {
            existing
        } else {
            by_module.insert(row.module.as_str(), BTreeMap::new());
            by_module.get_mut(row.module.as_str()).unwrap()
        };

        if let Some(val) = entry.get_mut(row.lang.as_str()) {
            *val += row.lines;
        } else {
            entry.insert(row.lang.as_str(), row.lines);
        }
    }

    let mut out = Vec::new();
    for (module, langs) in by_module {
        let mut total = 0usize;
        let mut dominant_lang: Option<&str> = None;
        let mut dominant_lines = 0usize;
        for (&lang, lines) in &langs {
            total += *lines;
            if *lines > dominant_lines
                || (*lines == dominant_lines && dominant_lang.is_some_and(|d| lang < d))
            {
                dominant_lines = *lines;
                dominant_lang = Some(lang);
            }
        }
        let pct = if total == 0 {
            0.0
        } else {
            safe_ratio(dominant_lines, total)
        };
        out.push(LangPurityRow {
            module: module.to_string(),
            lang_count: langs.len(),
            dominant_lang: dominant_lang.unwrap_or_default().to_string(),
            dominant_lines,
            dominant_pct: pct,
        });
    }

    out.sort_by(|a, b| a.module.cmp(&b.module));
    LangPurityReport { rows: out }
}

pub(super) fn build_polyglot_report(rows: &[&FileRow]) -> PolyglotReport {
    let mut by_lang: BTreeMap<&str, usize> = BTreeMap::new();
    let mut total = 0usize;

    for row in rows {
        if let Some(val) = by_lang.get_mut(row.lang.as_str()) {
            *val += row.code;
        } else {
            by_lang.insert(row.lang.as_str(), row.code);
        }
        total += row.code;
    }

    let mut entropy = 0.0;
    let mut dominant_lang: Option<&str> = None;
    let mut dominant_lines = 0usize;

    for (&lang, lines) in &by_lang {
        if *lines > dominant_lines
            || (*lines == dominant_lines && dominant_lang.is_some_and(|d| lang < d))
        {
            dominant_lines = *lines;
            dominant_lang = Some(lang);
        }
        if total > 0 && *lines > 0 {
            let p = *lines as f64 / total as f64;
            entropy -= p * p.log2();
        }
    }

    let dominant_pct = if total == 0 {
        0.0
    } else {
        safe_ratio(dominant_lines, total)
    };

    PolyglotReport {
        lang_count: by_lang.len(),
        entropy: round_f64(entropy, 4),
        dominant_lang: dominant_lang.unwrap_or_default().to_string(),
        dominant_lines,
        dominant_pct,
    }
}
