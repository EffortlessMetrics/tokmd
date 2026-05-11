use std::collections::BTreeMap;

use tokmd_analysis_types::{NearDupCluster, NearDupPairRow};

/// Path-compressed union-find with union by rank.
struct DisjointSets {
    parent: Vec<usize>,
    rank: Vec<usize>,
}

impl DisjointSets {
    fn new(n: usize) -> Self {
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
        }
    }

    fn find(&mut self, x: usize) -> usize {
        if self.parent[x] != x {
            self.parent[x] = self.find(self.parent[x]);
        }
        self.parent[x]
    }

    fn union(&mut self, a: usize, b: usize) {
        let ra = self.find(a);
        let rb = self.find(b);
        if ra == rb {
            return;
        }
        match self.rank[ra].cmp(&self.rank[rb]) {
            std::cmp::Ordering::Less => self.parent[ra] = rb,
            std::cmp::Ordering::Greater => self.parent[rb] = ra,
            std::cmp::Ordering::Equal => {
                self.parent[rb] = ra;
                self.rank[ra] += 1;
            }
        }
    }
}

/// Build connected-component clusters from near-duplicate pairs.
pub(super) fn build_clusters(pairs: &[NearDupPairRow]) -> Vec<NearDupCluster> {
    let mut name_to_idx: BTreeMap<&str, usize> = BTreeMap::new();
    let mut names: Vec<&str> = Vec::new();
    for pair in pairs {
        for name in [pair.left.as_str(), pair.right.as_str()] {
            if !name_to_idx.contains_key(name) {
                let idx = names.len();
                name_to_idx.insert(name, idx);
                names.push(name);
            }
        }
    }

    let mut ds = DisjointSets::new(names.len());
    let mut connection_count: BTreeMap<usize, usize> = BTreeMap::new();

    for pair in pairs {
        let a = name_to_idx[pair.left.as_str()];
        let b = name_to_idx[pair.right.as_str()];
        ds.union(a, b);
        *connection_count.entry(a).or_insert(0) += 1;
        *connection_count.entry(b).or_insert(0) += 1;
    }

    let mut components: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for i in 0..names.len() {
        let root = ds.find(i);
        components.entry(root).or_default().push(i);
    }

    let mut comp_max_sim: BTreeMap<usize, f64> = BTreeMap::new();
    let mut comp_pair_count: BTreeMap<usize, usize> = BTreeMap::new();
    for pair in pairs {
        let a = name_to_idx[pair.left.as_str()];
        let root = ds.find(a);
        let entry = comp_max_sim.entry(root).or_insert(0.0);
        if pair.similarity > *entry {
            *entry = pair.similarity;
        }
        *comp_pair_count.entry(root).or_insert(0) += 1;
    }

    let mut clusters: Vec<NearDupCluster> = components
        .into_iter()
        .map(|(root, members)| {
            let mut file_list: Vec<String> =
                members.iter().map(|&i| names[i].to_string()).collect();
            file_list.sort();

            let representative = members
                .iter()
                .copied()
                .max_by(|&a, &b| {
                    let ca = connection_count.get(&a).copied().unwrap_or(0);
                    let cb = connection_count.get(&b).copied().unwrap_or(0);
                    ca.cmp(&cb).then_with(|| names[b].cmp(names[a]))
                })
                .map(|i| names[i].to_string())
                .unwrap_or_default();

            let max_similarity = comp_max_sim.get(&root).copied().unwrap_or(0.0);
            let pair_count = comp_pair_count.get(&root).copied().unwrap_or(0);

            NearDupCluster {
                files: file_list,
                max_similarity,
                representative,
                pair_count,
            }
        })
        .collect();

    clusters.sort_by(|a, b| {
        b.max_similarity
            .partial_cmp(&a.max_similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.representative.cmp(&b.representative))
    });

    clusters
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disjoint_sets_find_self() {
        let mut ds = DisjointSets::new(5);
        for i in 0..5 {
            assert_eq!(ds.find(i), i);
        }
    }

    #[test]
    fn disjoint_sets_union_and_find() {
        let mut ds = DisjointSets::new(5);
        ds.union(0, 1);
        ds.union(2, 3);
        assert_eq!(ds.find(0), ds.find(1));
        assert_eq!(ds.find(2), ds.find(3));
        assert_ne!(ds.find(0), ds.find(2));
        ds.union(1, 3);
        assert_eq!(ds.find(0), ds.find(3));
    }

    #[test]
    fn disjoint_sets_idempotent_union() {
        let mut ds = DisjointSets::new(3);
        ds.union(0, 1);
        ds.union(0, 1);
        assert_eq!(ds.find(0), ds.find(1));
    }

    #[test]
    fn build_clusters_empty() {
        let pairs: Vec<NearDupPairRow> = vec![];
        let clusters = build_clusters(&pairs);
        assert!(clusters.is_empty());
    }

    #[test]
    fn build_clusters_single_pair() {
        let pairs = vec![NearDupPairRow {
            left: "a.rs".to_string(),
            right: "b.rs".to_string(),
            similarity: 0.95,
            shared_fingerprints: 10,
            left_fingerprints: 20,
            right_fingerprints: 20,
        }];
        let clusters = build_clusters(&pairs);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].files, vec!["a.rs", "b.rs"]);
        assert!((clusters[0].max_similarity - 0.95).abs() < 1e-10);
        assert_eq!(clusters[0].pair_count, 1);
    }

    #[test]
    fn build_clusters_two_components() {
        let pairs = vec![
            NearDupPairRow {
                left: "a.rs".to_string(),
                right: "b.rs".to_string(),
                similarity: 0.90,
                shared_fingerprints: 10,
                left_fingerprints: 20,
                right_fingerprints: 20,
            },
            NearDupPairRow {
                left: "c.rs".to_string(),
                right: "d.rs".to_string(),
                similarity: 0.85,
                shared_fingerprints: 8,
                left_fingerprints: 20,
                right_fingerprints: 20,
            },
        ];
        let clusters = build_clusters(&pairs);
        assert_eq!(clusters.len(), 2);
        assert!((clusters[0].max_similarity - 0.90).abs() < 1e-10);
        assert!((clusters[1].max_similarity - 0.85).abs() < 1e-10);
    }

    #[test]
    fn build_clusters_merged_component() {
        let pairs = vec![
            NearDupPairRow {
                left: "a.rs".to_string(),
                right: "b.rs".to_string(),
                similarity: 0.90,
                shared_fingerprints: 10,
                left_fingerprints: 20,
                right_fingerprints: 20,
            },
            NearDupPairRow {
                left: "b.rs".to_string(),
                right: "c.rs".to_string(),
                similarity: 0.85,
                shared_fingerprints: 8,
                left_fingerprints: 20,
                right_fingerprints: 20,
            },
            NearDupPairRow {
                left: "a.rs".to_string(),
                right: "c.rs".to_string(),
                similarity: 0.80,
                shared_fingerprints: 7,
                left_fingerprints: 20,
                right_fingerprints: 20,
            },
        ];
        let clusters = build_clusters(&pairs);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].files, vec!["a.rs", "b.rs", "c.rs"]);
        assert!((clusters[0].max_similarity - 0.90).abs() < 1e-10);
        assert_eq!(clusters[0].pair_count, 3);
        assert_eq!(clusters[0].representative, "a.rs");
    }

    #[test]
    fn build_clusters_representative_most_connected() {
        let pairs = vec![
            NearDupPairRow {
                left: "a.rs".to_string(),
                right: "b.rs".to_string(),
                similarity: 0.90,
                shared_fingerprints: 10,
                left_fingerprints: 20,
                right_fingerprints: 20,
            },
            NearDupPairRow {
                left: "b.rs".to_string(),
                right: "c.rs".to_string(),
                similarity: 0.85,
                shared_fingerprints: 8,
                left_fingerprints: 20,
                right_fingerprints: 20,
            },
        ];
        let clusters = build_clusters(&pairs);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].representative, "b.rs");
    }

    #[test]
    fn clusters_complete_despite_truncation() {
        let pairs = vec![
            NearDupPairRow {
                left: "a.rs".to_string(),
                right: "b.rs".to_string(),
                similarity: 0.95,
                shared_fingerprints: 10,
                left_fingerprints: 20,
                right_fingerprints: 20,
            },
            NearDupPairRow {
                left: "c.rs".to_string(),
                right: "d.rs".to_string(),
                similarity: 0.90,
                shared_fingerprints: 9,
                left_fingerprints: 20,
                right_fingerprints: 20,
            },
            NearDupPairRow {
                left: "d.rs".to_string(),
                right: "e.rs".to_string(),
                similarity: 0.85,
                shared_fingerprints: 8,
                left_fingerprints: 20,
                right_fingerprints: 20,
            },
        ];

        let clusters = build_clusters(&pairs);
        assert_eq!(clusters.len(), 2);

        let large_cluster = clusters.iter().find(|c| c.files.len() == 3).unwrap();
        assert_eq!(large_cluster.pair_count, 2);
        assert_eq!(large_cluster.files, vec!["c.rs", "d.rs", "e.rs"]);
    }
}
