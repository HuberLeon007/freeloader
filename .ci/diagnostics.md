## clippy
```
[1m[92m    Updating[0m crates.io index
[1m[92m    Checking[0m freeloader-protocol v0.1.0 (/home/runner/work/freeloader/freeloader/crates/protocol)
[1m[92m   Compiling[0m freeloader-desktop v0.1.0 (/home/runner/work/freeloader/freeloader/apps/desktop/src-tauri)
[1m[92m    Checking[0m tauri-plugin-opener v2.5.4
[1m[92m    Checking[0m tauri-plugin-single-instance v2.4.3
[1m[92m    Checking[0m freeloader-platform v0.1.0 (/home/runner/work/freeloader/freeloader/crates/platform)
[1m[92m    Checking[0m freeloader-download-core v0.1.0 (/home/runner/work/freeloader/freeloader/crates/download-core)
[1m[92m    Checking[0m freeloader-native-host v0.1.0 (/home/runner/work/freeloader/freeloader/crates/native-host)
crates/download-core/src/engine.rs:9:5: [1m[33mwarning[0m: unused import: `crate::repository::SqliteRepository`
crates/download-core/src/engine.rs:15:52: [1m[33mwarning[0m: unused import: `RecordPatch`
crates/download-core/src/engine.rs:16:48: [1m[33mwarning[0m: unused import: `TransferOutcome`
crates/download-core/src/engine.rs:17:13: [1m[33mwarning[0m: unused import: `DownloadStatus`
crates/download-core/src/engine.rs:64:5: [1m[33mwarning[0m: fields `settings`, `cancel_tokens`, and `progress_senders` are never read
crates/download-core/src/http_client.rs:18:5: [1m[33mwarning[0m: fields `connect_timeout`, `idle_timeout`, and `max_redirects` are never read
crates/download-core/src/clock_prod.rs:15:5: [1m[33mwarning[0m: you should consider adding a `Default` implementation for `SystemClock`
crates/download-core/src/filesystem.rs:14:5: [1m[33mwarning[0m: you should consider adding a `Default` implementation for `TokioFileSystem`
crates/download-core/src/repository.rs:69:12: [1m[33mwarning[0m: redundant closure: help: replace the closure with the method itself: `<&ErrorCode as Into<&'static str>>::into`
crates/download-core/src/repository.rs:73:12: [1m[33mwarning[0m: redundant closure: help: replace the closure with the method itself: `<&RestartNotice as Into<&'static str>>::into`
crates/download-core/src/clock_prod.rs:15:5: [1m[33mwarning[0m: missing documentation for an associated function
crates/download-core/src/engine.rs:25:5: [1m[33mwarning[0m: missing documentation for a struct field
crates/download-core/src/engine.rs:26:5: [1m[33mwarning[0m: missing documentation for a struct field
crates/download-core/src/engine.rs:27:5: [1m[33mwarning[0m: missing documentation for a struct field
crates/download-core/src/engine.rs:28:5: [1m[33mwarning[0m: missing documentation for a struct field
crates/download-core/src/engine.rs:29:5: [1m[33mwarning[0m: missing documentation for a struct field
crates/download-core/src/engine.rs:30:5: [1m[33mwarning[0m: missing documentation for a struct field
crates/download-core/src/engine.rs:31:5: [1m[33mwarning[0m: missing documentation for a struct field
crates/download-core/src/filesystem.rs:14:5: [1m[33mwarning[0m: missing documentation for an associated function
[1m[33mwarning[0m: `freeloader-download-core` (lib) generated 19 warnings (run `cargo clippy --fix --lib -p freeloader-download-core -- ` to apply 8 suggestions)
[1m[33mwarning[0m: `freeloader-download-core` (lib test) generated 19 warnings (19 duplicates)
apps/desktop/src-tauri/src/main.rs:210:14: [1m[91merror[0m: proc macro panicked
[1m[91merror[0m: could not compile `freeloader-desktop` (bin "freeloader-desktop" test) due to 1 previous error
[1m[33mwarning[0m: build failed, waiting for other jobs to finish...
[1m[91merror[0m: could not compile `freeloader-desktop` (bin "freeloader-desktop") due to 1 previous error
exit=101
```
## test
```
[1m[92m   Compiling[0m freeloader-protocol v0.1.0 (/home/runner/work/freeloader/freeloader/crates/protocol)
[1m[92m   Compiling[0m freeloader-desktop v0.1.0 (/home/runner/work/freeloader/freeloader/apps/desktop/src-tauri)
[1m[92m   Compiling[0m freeloader-platform v0.1.0 (/home/runner/work/freeloader/freeloader/crates/platform)
[1m[92m   Compiling[0m freeloader-download-core v0.1.0 (/home/runner/work/freeloader/freeloader/crates/download-core)
[1m[92m   Compiling[0m freeloader-native-host v0.1.0 (/home/runner/work/freeloader/freeloader/crates/native-host)
[1m[33mwarning[0m[1m: unused import: `crate::repository::SqliteRepository`[0m
 [1m[94m--> [0mcrates/download-core/src/engine.rs:9:5
  [1m[94m|[0m
[1m[94m9[0m [1m[94m|[0m use crate::repository::SqliteRepository;
  [1m[94m|[0m     [1m[33m^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^[0m
  [1m[94m|[0m
  [1m[94m= [0m[1mnote[0m: `#[warn(unused_imports)]` (part of `#[warn(unused)]`) on by default

[1m[33mwarning[0m[1m: unused import: `RecordPatch`[0m
  [1m[94m--> [0mcrates/download-core/src/engine.rs:15:52
   [1m[94m|[0m
[1m[94m15[0m [1m[94m|[0m use crate::seams::repository::{DownloadRepository, RecordPatch};
   [1m[94m|[0m                                                    [1m[33m^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: unused import: `TransferOutcome`[0m
  [1m[94m--> [0mcrates/download-core/src/engine.rs:16:48
   [1m[94m|[0m
[1m[94m16[0m [1m[94m|[0m use crate::seams::strategy::{DownloadStrategy, TransferOutcome};
   [1m[94m|[0m                                                [1m[33m^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: unused import: `DownloadStatus`[0m
  [1m[94m--> [0mcrates/download-core/src/engine.rs:17:13
   [1m[94m|[0m
[1m[94m17[0m [1m[94m|[0m use crate::{DownloadStatus, EngineError, Progress};
   [1m[94m|[0m             [1m[33m^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: fields `settings`, `cancel_tokens`, and `progress_senders` are never read[0m
  [1m[94m--> [0mcrates/download-core/src/engine.rs:64:5
   [1m[94m|[0m
[1m[94m62[0m [1m[94m|[0m pub struct DownloadEngine {
   [1m[94m|[0m            [1m[94m--------------[0m [1m[94mfields in this struct[0m
[1m[94m63[0m [1m[94m|[0m     deps: Arc<EngineDependencies>,
[1m[94m64[0m [1m[94m|[0m     settings: EngineSettings,
   [1m[94m|[0m     [1m[33m^^^^^^^^[0m
[1m[94m65[0m [1m[94m|[0m     /// Cancel tokens per download ID.
[1m[94m66[0m [1m[94m|[0m     cancel_tokens: Mutex<HashMap<Uuid, tokio_util::sync::CancellationToken>>,
   [1m[94m|[0m     [1m[33m^^^^^^^^^^^^^[0m
[1m[94m67[0m [1m[94m|[0m     /// Progress senders per download ID.
[1m[94m68[0m [1m[94m|[0m     progress_senders: Mutex<HashMap<Uuid, watch::Sender<Progress>>>,
   [1m[94m|[0m     [1m[33m^^^^^^^^^^^^^^^^[0m
   [1m[94m|[0m
   [1m[94m= [0m[1mnote[0m: `#[warn(dead_code)]` (part of `#[warn(unused)]`) on by default

[1m[33mwarning[0m[1m: fields `connect_timeout`, `idle_timeout`, and `max_redirects` are never read[0m
  [1m[94m--> [0mcrates/download-core/src/http_client.rs:18:5
   [1m[94m|[0m
[1m[94m16[0m [1m[94m|[0m pub struct ReqwestHttpClient {
   [1m[94m|[0m            [1m[94m-----------------[0m [1m[94mfields in this struct[0m
[1m[94m17[0m [1m[94m|[0m     inner: reqwest::Client,
[1m[94m18[0m [1m[94m|[0m     connect_timeout: Duration,
   [1m[94m|[0m     [1m[33m^^^^^^^^^^^^^^^[0m
[1m[94m19[0m [1m[94m|[0m     idle_timeout: Duration,
   [1m[94m|[0m     [1m[33m^^^^^^^^^^^^[0m
[1m[94m20[0m [1m[94m|[0m     max_redirects: u8,
   [1m[94m|[0m     [1m[33m^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: missing documentation for an associated function[0m
  [1m[94m--> [0mcrates/download-core/src/clock_prod.rs:15:5
   [1m[94m|[0m
[1m[94m15[0m [1m[94m|[0m     pub fn new() -> Self {
   [1m[94m|[0m     [1m[33m^^^^^^^^^^^^^^^^^^^^[0m
   [1m[94m|[0m
   [1m[94m= [0m[1mnote[0m: requested on the command line with `-W missing-docs`

[1m[33mwarning[0m[1m: missing documentation for a struct field[0m
  [1m[94m--> [0mcrates/download-core/src/engine.rs:25:5
   [1m[94m|[0m
[1m[94m25[0m [1m[94m|[0m     pub http: Arc<dyn HttpClient>,
   [1m[94m|[0m     [1m[33m^^^^^^^^^^^^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: missing documentation for a struct field[0m
  [1m[94m--> [0mcrates/download-core/src/engine.rs:26:5
   [1m[94m|[0m
[1m[94m26[0m [1m[94m|[0m     pub repository: Arc<dyn DownloadRepository>,
   [1m[94m|[0m     [1m[33m^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: missing documentation for a struct field[0m
  [1m[94m--> [0mcrates/download-core/src/engine.rs:27:5
   [1m[94m|[0m
[1m[94m27[0m [1m[94m|[0m     pub file_system: Arc<dyn FileSystem>,
   [1m[94m|[0m     [1m[33m^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: missing documentation for a struct field[0m
  [1m[94m--> [0mcrates/download-core/src/engine.rs:28:5
   [1m[94m|[0m
[1m[94m28[0m [1m[94m|[0m     pub clock: Arc<dyn Clock>,
   [1m[94m|[0m     [1m[33m^^^^^^^^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: missing documentation for a struct field[0m
  [1m[94m--> [0mcrates/download-core/src/engine.rs:29:5
   [1m[94m|[0m
[1m[94m29[0m [1m[94m|[0m     pub rate_limiter: Arc<dyn RateLimiter>,
   [1m[94m|[0m     [1m[33m^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: missing documentation for a struct field[0m
  [1m[94m--> [0mcrates/download-core/src/engine.rs:30:5
   [1m[94m|[0m
[1m[94m30[0m [1m[94m|[0m     pub strategy: Arc<dyn DownloadStrategy>,
   [1m[94m|[0m     [1m[33m^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: missing documentation for a struct field[0m
  [1m[94m--> [0mcrates/download-core/src/engine.rs:31:5
   [1m[94m|[0m
[1m[94m31[0m [1m[94m|[0m     pub checksums: Arc<dyn ChecksumVerifier>,
   [1m[94m|[0m     [1m[33m^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m[1m: missing documentation for an associated function[0m
  [1m[94m--> [0mcrates/download-core/src/filesystem.rs:14:5
   [1m[94m|[0m
[1m[94m14[0m [1m[94m|[0m     pub fn new() -> Self {
   [1m[94m|[0m     [1m[33m^^^^^^^^^^^^^^^^^^^^[0m

[1m[33mwarning[0m: `freeloader-download-core` (lib test) generated 15 warnings (15 duplicates)
[1m[33mwarning[0m: `freeloader-download-core` (lib) generated 15 warnings (run `cargo fix --lib -p freeloader-download-core` to apply 4 suggestions)
[1m[91merror[0m[1m: proc macro panicked[0m
   [1m[94m--> [0mapps/desktop/src-tauri/src/main.rs:210:14
    [1m[94m|[0m
[1m[94m210[0m [1m[94m|[0m         .run(tauri::generate_context!())
    [1m[94m|[0m              [1m[91m^^^^^^^^^^^^^^^^^^^^^^^^^^[0m
    [1m[94m|[0m
    [1m[94m= [0m[1mhelp[0m: message: failed to open icon /home/runner/work/freeloader/freeloader/apps/desktop/src-tauri/icons/32x32.png: No such file or directory (os error 2)

[1m[91merror[0m: could not compile `freeloader-desktop` (bin "freeloader-desktop" test) due to 1 previous error
exit=101
```
