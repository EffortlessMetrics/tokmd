use std::path::PathBuf;
use proptest::prelude::*;
use tokmd_scan_args::{normalize_scan_input, scan_args};
use tokmd_settings::ScanOptions;
use tokmd_types::RedactMode;

proptest! {
    #[test]
    fn test_normalize_scan_input_deterministic(path in "[a-zA-Z0-9_./\\\\]{1,100}") {
        let p = PathBuf::from(path);
        let normalized = normalize_scan_input(&p);
        let normalized2 = normalize_scan_input(&p);
        assert_eq!(normalized, normalized2);
    }
}
