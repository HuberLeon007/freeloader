## clippy
```
[1m[92m    Checking[0m freeloader-protocol v0.1.0 (/home/runner/work/freeloader/freeloader/crates/protocol)
[1m[92m   Compiling[0m freeloader-desktop v0.1.0 (/home/runner/work/freeloader/freeloader/apps/desktop/src-tauri)
[1m[92m    Checking[0m freeloader-platform v0.1.0 (/home/runner/work/freeloader/freeloader/crates/platform)
[1m[92m    Checking[0m freeloader-download-core v0.1.0 (/home/runner/work/freeloader/freeloader/crates/download-core)
[1m[92m    Checking[0m freeloader-native-host v0.1.0 (/home/runner/work/freeloader/freeloader/crates/native-host)
[1m[92m    Finished[0m `dev` profile [unoptimized + debuginfo] target(s) in 2.76s
exit=0
```
## test
```
[1m[92m   Compiling[0m freeloader-protocol v0.1.0 (/home/runner/work/freeloader/freeloader/crates/protocol)
[1m[92m   Compiling[0m freeloader-desktop v0.1.0 (/home/runner/work/freeloader/freeloader/apps/desktop/src-tauri)
[1m[92m   Compiling[0m freeloader-platform v0.1.0 (/home/runner/work/freeloader/freeloader/crates/platform)
[1m[92m   Compiling[0m freeloader-download-core v0.1.0 (/home/runner/work/freeloader/freeloader/crates/download-core)
[1m[92m   Compiling[0m freeloader-native-host v0.1.0 (/home/runner/work/freeloader/freeloader/crates/native-host)
[1m[92m    Finished[0m `test` profile [unoptimized + debuginfo] target(s) in 4.47s
[1m[92m     Running[0m unittests src/main.rs (target/debug/deps/freeloader_desktop-8040301cfaf0ab41)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

[1m[92m     Running[0m unittests src/lib.rs (target/debug/deps/freeloader_download_core-070d26390e52cd02)

running 10 tests
test containment::tests::resolve_unique_no_conflict ... ok
test containment::tests::containment_accepts_normal_path ... ok
test containment::tests::resolve_unique_avoids_collision ... ok
test containment::tests::containment_rejects_parent_traversal ... FAILED
test tests::sanitizes_path_traversal ... ok
test tests::sanitizes_empty_to_download ... ok
test tests::sanitizes_windows_device_names ... ok
test tests::state_machine_accepts_normal_flow ... ok
test tests::state_machine_rejects_invalid_transitions ... ok
test tests::open_database_is_idempotent ... ok

failures:

---- containment::tests::containment_rejects_parent_traversal stdout ----

thread 'containment::tests::containment_rejects_parent_traversal' (4336) panicked at crates/download-core/src/containment.rs:106:9:
assertion failed: result.is_err()
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    containment::tests::containment_rejects_parent_traversal

test result: FAILED. 9 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

[1m[91merror[0m: test failed, to rerun pass `-p freeloader-download-core --lib`
exit=101
```
