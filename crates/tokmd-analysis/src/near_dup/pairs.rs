//! Pair scoring for near-duplicate detection.

use rustc_hash::FxHashMap;

use tokmd_analysis_types::NearDupPairRow;
use tokmd_types::FileRow;

use super::fingerprint::MAX_POSTINGS;

pub(super) struct PairingResult {
    pub(super) pairs: Vec<NearDupPairRow>,
    pub(super) pairing_ms: u64,
}

/// Build thresholded near-duplicate pairs from partitioned file fingerprints.
pub(super) fn build_pairs(
    partition_fps: &[Vec<(usize, Vec<u64>)>],
    files: &[&FileRow],
    threshold: f64,
) -> PairingResult {
    let pair_start = std::time::Instant::now();
    let mut pairs = Vec::new();

    for file_fingerprints in partition_fps {
        if file_fingerprints.len() < 2 {
            continue;
        }

        let inverted = inverted_index(file_fingerprints);
        let pair_shared = shared_fingerprint_counts(&inverted);

        for ((a, b), shared) in pair_shared {
            let fp_a = file_fingerprints[a].1.len();
            let fp_b = file_fingerprints[b].1.len();
            let union = fp_a + fp_b - shared;
            if union == 0 {
                continue;
            }

            let similarity = shared as f64 / union as f64;
            if similarity >= threshold {
                let idx_a = file_fingerprints[a].0;
                let idx_b = file_fingerprints[b].0;
                pairs.push(NearDupPairRow {
                    left: files[idx_a].path.clone(),
                    right: files[idx_b].path.clone(),
                    similarity: round4(similarity),
                    shared_fingerprints: shared,
                    left_fingerprints: fp_a,
                    right_fingerprints: fp_b,
                });
            }
        }
    }

    sort_pairs(&mut pairs);

    PairingResult {
        pairs,
        pairing_ms: pair_start.elapsed().as_millis() as u64,
    }
}

fn inverted_index(file_fingerprints: &[(usize, Vec<u64>)]) -> FxHashMap<u64, Vec<usize>> {
    let mut inverted: FxHashMap<u64, Vec<usize>> = FxHashMap::default();
    for (local_idx, (_, fps)) in file_fingerprints.iter().enumerate() {
        for &fp in fps {
            inverted.entry(fp).or_default().push(local_idx);
        }
    }
    inverted
}

fn shared_fingerprint_counts(
    inverted: &FxHashMap<u64, Vec<usize>>,
) -> FxHashMap<(usize, usize), usize> {
    let mut pair_shared: FxHashMap<(usize, usize), usize> = FxHashMap::default();
    for posting_list in inverted.values() {
        if posting_list.len() > MAX_POSTINGS {
            continue;
        }
        for i in 0..posting_list.len() {
            for j in (i + 1)..posting_list.len() {
                let a = posting_list[i];
                let b = posting_list[j];
                if a == b {
                    continue;
                }
                let key = if a <= b { (a, b) } else { (b, a) };
                *pair_shared.entry(key).or_insert(0) += 1;
            }
        }
    }
    pair_shared
}

fn sort_pairs(pairs: &mut [NearDupPairRow]) {
    pairs.sort_by(|a, b| {
        b.similarity
            .partial_cmp(&a.similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.left.cmp(&b.left))
            .then_with(|| a.right.cmp(&b.right))
    });
}

fn round4(v: f64) -> f64 {
    (v * 10000.0).round() / 10000.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokmd_types::FileKind;

    fn row(path: &str) -> FileRow {
        FileRow {
            path: path.to_string(),
            module: "src".to_string(),
            lang: "Rust".to_string(),
            kind: FileKind::Parent,
            code: 10,
            comments: 0,
            blanks: 0,
            lines: 10,
            bytes: 100,
            tokens: 10,
        }
    }

    #[test]
    fn build_pairs_filters_by_threshold() {
        let files = [row("a.rs"), row("b.rs"), row("c.rs")];
        let file_refs = files.iter().collect::<Vec<_>>();
        let partition_fps = vec![vec![
            (0, vec![1, 2, 3, 4]),
            (1, vec![1, 2, 3, 5]),
            (2, vec![10, 11, 12, 13]),
        ]];

        let result = build_pairs(&partition_fps, &file_refs, 0.5);

        assert_eq!(result.pairs.len(), 1);
        assert_eq!(result.pairs[0].left, "a.rs");
        assert_eq!(result.pairs[0].right, "b.rs");
        assert_eq!(result.pairs[0].similarity, 0.6);
        assert_eq!(result.pairs[0].shared_fingerprints, 3);
    }

    #[test]
    fn pair_sort_deterministic_with_right_tiebreak() {
        let mut pairs = [
            NearDupPairRow {
                left: "a.rs".to_string(),
                right: "c.rs".to_string(),
                similarity: 0.90,
                shared_fingerprints: 10,
                left_fingerprints: 20,
                right_fingerprints: 20,
            },
            NearDupPairRow {
                left: "a.rs".to_string(),
                right: "b.rs".to_string(),
                similarity: 0.90,
                shared_fingerprints: 10,
                left_fingerprints: 20,
                right_fingerprints: 20,
            },
        ];

        sort_pairs(&mut pairs);

        assert_eq!(pairs[0].right, "b.rs");
        assert_eq!(pairs[1].right, "c.rs");
    }

    #[test]
    fn self_pair_guard_skips_same_index() {
        let mut inverted = FxHashMap::default();
        inverted.insert(1, vec![0, 0, 1]);

        let pair_shared = shared_fingerprint_counts(&inverted);

        assert_eq!(pair_shared.get(&(0, 1)), Some(&2));
    }
}
