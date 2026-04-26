#![allow(dead_code)]

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

const FIXTURE_NAMESPACE: &[u8] = b"tokmd/high-entropy-test-fixture/v1";
const SYNTHETIC_PRIVATE_KEY_LEN: usize = 2048;

pub const GENERATED_PRIVATE_KEY_RELATIVE_PATH: &str = "fixtures/generated/private-key.pk8";

pub mod label {
    pub const ENTROPY_PRIMARY: &str = "entropy-private-key-primary";
    pub const ENTROPY_ALTERNATE: &str = "entropy-private-key-alternate";
    pub const ENTROPY_REPORT: &str = "entropy-private-key-report";
    pub const SECURITY_SUSPECT: &str = "security-private-key-suspect";
}

pub fn synthetic_private_key_bytes(label: &str) -> Vec<u8> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(FIXTURE_NAMESPACE);
    hasher.update(label.as_bytes());

    let mut bytes = vec![0; SYNTHETIC_PRIVATE_KEY_LEN];
    hasher.finalize_xof().fill(&mut bytes);
    bytes
}

pub fn generated_private_key_output_path(root: &Path) -> PathBuf {
    root.join("fixtures")
        .join("generated")
        .join("private-key.pk8")
}

pub fn write_generated_private_key(root: &Path, label: &str) -> io::Result<PathBuf> {
    let output_path = generated_private_key_output_path(root);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&output_path, synthetic_private_key_bytes(label))?;
    Ok(output_path)
}
