sed -i '1s/^/#![cfg(feature = "git")]\n/' crates/tokmd/tests/handoff_integration.rs
sed -i '1s/^/#![cfg(feature = "git")]\n/' crates/tokmd/tests/handoff_w71.rs
