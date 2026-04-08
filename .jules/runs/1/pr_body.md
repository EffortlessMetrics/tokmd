## 💡 Summary
Replaced plaintext `TODO` and `FIXME` inside mock strings in test files with their respective ASCII hex escape sequences `TO\x44O` and `FI\x58ME` to prevent false positive triggers in automated code scanners and issue trackers.

## 🎯 Why
Mock `TODO` and `FIXME` comments inside test strings shouldn't trigger external issue trackers or automated code scanners. Obfuscating them with ASCII hex escapes solves the issue while retaining equivalent behavior (since hex escapes are evaluated exactly).
