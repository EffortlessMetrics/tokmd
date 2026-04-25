use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use uselesskey::{Factory, RsaFactoryExt, RsaSpec, Seed};

const SEED_NAMESPACE: &str = "tokmd/uselesskey/v1";

pub const GENERATED_PRIVATE_KEY_RELATIVE_PATH: &str = "fixtures/generated/private-key.pk8";

pub mod label {
    pub const ENTROPY_PRIMARY: &str = "entropy-private-key-primary";
    pub const ENTROPY_ALTERNATE: &str = "entropy-private-key-alternate";
    pub const ENTROPY_REPORT: &str = "entropy-private-key-report";
    pub const SECURITY_SUSPECT: &str = "security-private-key-suspect";
}

pub fn factory() -> &'static Factory {
    static FACTORY: OnceLock<Factory> = OnceLock::new();

    FACTORY.get_or_init(|| {
        let seed =
            Seed::from_env_value(SEED_NAMESPACE).expect("seed namespace should remain valid");
        Factory::deterministic(seed)
    })
}

pub fn rsa_private_key_pkcs8_der(label: &str) -> Vec<u8> {
    factory()
        .rsa(label, RsaSpec::rs256())
        .private_key_pkcs8_der()
        .to_vec()
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
    fs::write(&output_path, rsa_private_key_pkcs8_der(label))?;
    Ok(output_path)
}

#[cfg(test)]
mod tests {
    use super::{factory, generated_private_key_output_path, label, write_generated_private_key};
    use uselesskey::RsaFactoryExt;

    #[test]
    fn factory_is_deterministic_for_stable_labels() {
        let first = factory().rsa(label::ENTROPY_PRIMARY, uselesskey::RsaSpec::rs256());
        let second = factory().rsa(label::ENTROPY_PRIMARY, uselesskey::RsaSpec::rs256());
        let different = factory().rsa(label::ENTROPY_ALTERNATE, uselesskey::RsaSpec::rs256());

        assert_eq!(
            first.private_key_pkcs8_der(),
            second.private_key_pkcs8_der()
        );
        assert_ne!(
            first.private_key_pkcs8_der(),
            different.private_key_pkcs8_der()
        );
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
