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

#[cfg(test)]
mod tests {
    use super::{
        generated_private_key_output_path, label, synthetic_private_key_bytes,
        write_generated_private_key,
    };

    #[test]
    fn synthetic_private_key_bytes_are_deterministic_for_stable_labels() {
        let first = synthetic_private_key_bytes(label::ENTROPY_PRIMARY);
        let second = synthetic_private_key_bytes(label::ENTROPY_PRIMARY);
        let different = synthetic_private_key_bytes(label::ENTROPY_ALTERNATE);

        assert_eq!(first, second);
        assert_ne!(first, different);
    }

    #[test]
    fn write_generated_private_key_uses_shared_fixture_path() {
        let dir = tempfile::tempdir().expect("tempdir");
        let output_path =
            write_generated_private_key(dir.path(), label::ENTROPY_PRIMARY).expect("write key");

        assert_eq!(output_path, generated_private_key_output_path(dir.path()));
        assert!(output_path.exists());
    }
}
