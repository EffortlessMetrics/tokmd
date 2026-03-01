sed -i 's/let mut is_root = false;/let is_root = module_roots.iter().any(|r| r == first);/g' crates/tokmd-module-key/src/lib.rs
sed -i 's/for r in module_roots {//g' crates/tokmd-module-key/src/lib.rs
sed -i 's/    if r == first {//g' crates/tokmd-module-key/src/lib.rs
sed -i 's/        is_root = true;//g' crates/tokmd-module-key/src/lib.rs
sed -i 's/        break;//g' crates/tokmd-module-key/src/lib.rs
sed -i '/    }/,/    }/d' crates/tokmd-module-key/src/lib.rs
