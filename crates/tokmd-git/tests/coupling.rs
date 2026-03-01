//! Coupling analysis tests for tokmd-git.
//!
//! Verifies that `collect_history()` produces data suitable for computing
//! file coupling (co-change) matrices. Uses real temporary git repos.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use tokmd_git::{collect_history, git_available, repo_root};

// ============================================================================
// Helpers
// ============================================================================

fn git_in(dir: &Path) -> Command {
    let mut cmd = Command::new("git");
    cmd.env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .current_dir(dir);
    cmd
}

struct TempGitRepo {
    path: PathBuf,
}

impl Drop for TempGitRepo {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.path).ok();
    }
}

fn make_repo(suffix: &str) -> Option<TempGitRepo> {
    if !git_available() {
        return None;
    }
    let id = format!(
        "{}-{:?}-{}-coupling-{}",
        std::process::id(),
        std::thread::current().id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0),
        suffix,
    );
    let dir = std::env::temp_dir().join(format!("tokmd-git-coupling-{}", id));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).ok();
    }
    std::fs::create_dir_all(&dir).ok()?;

    let ok = |o: std::process::Output| o.status.success();
    if !ok(git_in(&dir).args(["init"]).output().ok()?) {
        std::fs::remove_dir_all(&dir).ok();
        return None;
    }
    git_in(&dir)
        .args(["config", "user.email", "coupling@test.com"])
        .output()
        .ok()?;
    git_in(&dir)
        .args(["config", "user.name", "Coupling Tester"])
        .output()
        .ok()?;

    std::fs::write(dir.join("seed.txt"), "seed").ok()?;
    git_in(&dir).args(["add", "."]).output().ok()?;
    if !ok(git_in(&dir).args(["commit", "-m", "seed"]).output().ok()?) {
        std::fs::remove_dir_all(&dir).ok();
        return None;
    }
    Some(TempGitRepo { path: dir })
}

fn commit_files(dir: &Path, files: &[(&str, &str)], message: &str) {
    for (name, content) in files {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, content).unwrap();
    }
    git_in(dir).args(["add", "."]).output().unwrap();
    git_in(dir)
        .args(["commit", "-m", message])
        .output()
        .unwrap();
}

/// Build a coupling matrix from collected commits.
/// Returns a BTreeMap of sorted file pairs → co-change count.
fn build_coupling_matrix(commits: &[tokmd_git::GitCommit]) -> BTreeMap<(String, String), usize> {
    let mut matrix: BTreeMap<(String, String), usize> = BTreeMap::new();
    for c in commits {
        let mut files: Vec<_> = c.files.clone();
        files.sort();
        files.dedup();
        for i in 0..files.len() {
            for j in (i + 1)..files.len() {
                *matrix
                    .entry((files[i].clone(), files[j].clone()))
                    .or_default() += 1;
            }
        }
    }
    matrix
}

// ============================================================================
// Tests
// ============================================================================

/// Files committed together N times produce coupling count of N.
#[test]
fn coupling_matrix_counts_co_changes_correctly() {
    let repo = make_repo("count").expect("repo");

    // Commit a.txt and b.txt together 4 times
    for i in 0..4 {
        commit_files(
            &repo.path,
            &[("a.txt", &format!("a-v{i}")), ("b.txt", &format!("b-v{i}"))],
            &format!("pair {i}"),
        );
    }

    let root = repo_root(&repo.path).unwrap();
    let commits = collect_history(&root, None, None).unwrap();
    let matrix = build_coupling_matrix(&commits);

    let pair = ("a.txt".to_string(), "b.txt".to_string());
    let count = matrix.get(&pair).copied().unwrap_or(0);
    assert_eq!(count, 4, "a.txt and b.txt co-changed 4 times, got {count}");
}

/// Empty repo (no commits) produces an empty coupling matrix.
#[test]
fn empty_repo_produces_empty_coupling() {
    if !git_available() {
        return;
    }

    let id = format!(
        "{}-{:?}-{}-coupling-empty",
        std::process::id(),
        std::thread::current().id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0),
    );
    let dir = std::env::temp_dir().join(format!("tokmd-git-coupling-{}", id));
    if dir.exists() {
        std::fs::remove_dir_all(&dir).ok();
    }
    std::fs::create_dir_all(&dir).unwrap();

    git_in(&dir).args(["init"]).output().unwrap();
    git_in(&dir)
        .args(["config", "user.email", "e@t.com"])
        .output()
        .unwrap();
    git_in(&dir)
        .args(["config", "user.name", "E"])
        .output()
        .unwrap();

    let result = collect_history(&dir, None, None);
    let commits = match result {
        Ok(c) => c,
        Err(_) => Vec::new(), // git log on empty repo may fail
    };

    let matrix = build_coupling_matrix(&commits);
    assert!(
        matrix.is_empty(),
        "Empty repo should produce empty coupling matrix"
    );

    std::fs::remove_dir_all(&dir).ok();
}

/// Single-file commits produce no coupling pairs.
#[test]
fn single_file_commits_produce_no_coupling() {
    let repo = make_repo("single-file").expect("repo");

    // Each commit touches exactly one file
    commit_files(&repo.path, &[("x.txt", "x1")], "add x");
    commit_files(&repo.path, &[("y.txt", "y1")], "add y");
    commit_files(&repo.path, &[("z.txt", "z1")], "add z");
    commit_files(&repo.path, &[("x.txt", "x2")], "edit x");
    commit_files(&repo.path, &[("y.txt", "y2")], "edit y");

    let root = repo_root(&repo.path).unwrap();
    let commits = collect_history(&root, None, None).unwrap();
    let matrix = build_coupling_matrix(&commits);

    // Filter out the seed commit (which only has seed.txt)
    // Single-file commits should contribute zero pairs
    let non_seed_pairs: BTreeMap<_, _> = matrix
        .into_iter()
        .filter(|((a, b), _)| a != "seed.txt" && b != "seed.txt")
        .collect();

    assert!(
        non_seed_pairs.is_empty(),
        "Single-file commits should produce no coupling, got: {:?}",
        non_seed_pairs
    );
}

/// Coupling matrix is symmetric: (A,B) == (B,A) when keys are sorted.
#[test]
fn coupling_matrix_is_symmetric() {
    let repo = make_repo("symmetric").expect("repo");

    commit_files(&repo.path, &[("p.txt", "p"), ("q.txt", "q")], "commit pq 1");
    commit_files(
        &repo.path,
        &[("q.txt", "q2"), ("p.txt", "p2")],
        "commit pq 2",
    );

    let root = repo_root(&repo.path).unwrap();
    let commits = collect_history(&root, None, None).unwrap();
    let matrix = build_coupling_matrix(&commits);

    // Since keys are always sorted (a < b), there should be no (q,p) entry
    let pq = matrix
        .get(&("p.txt".to_string(), "q.txt".to_string()))
        .copied()
        .unwrap_or(0);
    let qp = matrix
        .get(&("q.txt".to_string(), "p.txt".to_string()))
        .copied()
        .unwrap_or(0);
    assert_eq!(qp, 0, "Reverse pair should not exist in sorted matrix");
    assert_eq!(pq, 2, "Forward pair count should be 2");
}

/// Three files committed together form a triangle of pairs.
#[test]
fn coupling_triangle_three_files() {
    let repo = make_repo("triangle").expect("repo");

    // Commit three files together twice
    for i in 0..2 {
        commit_files(
            &repo.path,
            &[
                ("t1.txt", &format!("v{i}")),
                ("t2.txt", &format!("v{i}")),
                ("t3.txt", &format!("v{i}")),
            ],
            &format!("triple {i}"),
        );
    }

    let root = repo_root(&repo.path).unwrap();
    let commits = collect_history(&root, None, None).unwrap();
    let matrix = build_coupling_matrix(&commits);

    let pairs = [
        ("t1.txt".to_string(), "t2.txt".to_string()),
        ("t1.txt".to_string(), "t3.txt".to_string()),
        ("t2.txt".to_string(), "t3.txt".to_string()),
    ];
    for pair in &pairs {
        let count = matrix.get(pair).copied().unwrap_or(0);
        assert_eq!(
            count, 2,
            "Pair {:?} should have co-change count 2, got {count}",
            pair
        );
    }
}

/// Mixed coupling: some files coupled, others not.
#[test]
fn coupling_mixed_patterns() {
    let repo = make_repo("mixed").expect("repo");

    // a.txt and b.txt committed together 3 times
    for i in 0..3 {
        commit_files(
            &repo.path,
            &[("a.txt", &format!("a{i}")), ("b.txt", &format!("b{i}"))],
            &format!("ab-{i}"),
        );
    }
    // c.txt committed alone
    commit_files(&repo.path, &[("c.txt", "c1")], "solo-c");

    let root = repo_root(&repo.path).unwrap();
    let commits = collect_history(&root, None, None).unwrap();
    let matrix = build_coupling_matrix(&commits);

    let ab = matrix
        .get(&("a.txt".to_string(), "b.txt".to_string()))
        .copied()
        .unwrap_or(0);
    let ac = matrix
        .get(&("a.txt".to_string(), "c.txt".to_string()))
        .copied()
        .unwrap_or(0);
    let bc = matrix
        .get(&("b.txt".to_string(), "c.txt".to_string()))
        .copied()
        .unwrap_or(0);

    assert_eq!(ab, 3, "a+b coupled 3 times");
    assert_eq!(ac, 0, "a+c never coupled");
    assert_eq!(bc, 0, "b+c never coupled");
}
