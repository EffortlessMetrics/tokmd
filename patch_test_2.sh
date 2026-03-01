sed -i 's/let f = subdir.join("data.bin");/let f = dir.path().join("sub\\\\data.bin");/g' crates/tokmd-analysis-entropy/tests/bdd.rs
