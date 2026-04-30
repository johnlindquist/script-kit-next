//! Source-level contract test for the Run 5 Pass #3
//! `tool-kit-config-writable-probe` user story.
//!
//! Pins the `getConfigFingerprint` stdin RPC that exposes the current
//! `~/.kit/config.ts` (path, len, modified_ms) fingerprint so
//! automation can verify a write landed on disk without invoking
//! `bun` or `stat`. The receipt is the same `(len, modified_ms)` pair
//! the config loader uses for its cache-hit check in
//! `try_load_cached_config`, so automation comparing two snapshots
//! can confirm "the next `bun` invocation will re-read the file".
//!
//! The contract pins four surfaces because any one of them drifting
//! would silently break automation: (a) the variant's serde shape,
//! (b) `request_id()` / `command_type()` wiring to the exact string
//! `"getConfigFingerprint"`, (c) all three dispatcher files include
//! the arm and emit `config_fingerprint_result` with the right fields,
//! (d) the `ConfigFingerprintReceipt` struct preserves the
//! serialization shape automation keys on (`path`, `len`, `modifiedMs`
//! via serde default rename — camelCase is NOT forced here since the
//! struct uses default snake_case except where overridden; see below).

const STDIN_COMMANDS: &str = include_str!("../src/stdin_commands/mod.rs");
const APP_RUN_SETUP: &str = include_str!("../src/main_entry/app_run_setup.rs");
const RUNTIME_STDIN: &str = include_str!("../src/main_entry/runtime_stdin.rs");
const RUNTIME_STDIN_MATCH_TAIL: &str =
    include_str!("../src/main_entry/runtime_stdin_match_tail.rs");
const CONFIG_LOADER: &str = include_str!("../src/config/loader.rs");
const CONFIG_MOD: &str = include_str!("../src/config/mod.rs");

#[test]
fn get_config_fingerprint_variant_is_defined_with_request_id_only() {
    // The RPC is pure read — no args beyond the correlation id. Adding
    // args (e.g. a path override or a force-rehash flag) must be a
    // separate story so the variant's protocol shape can't drift
    // silently.
    assert!(
        STDIN_COMMANDS.contains(
            "GetConfigFingerprint {\n        #[serde(default, rename = \"requestId\")]\n        request_id: Option<ExternalCommandRequestId>,\n    },"
        ),
        "src/stdin_commands/mod.rs must define `GetConfigFingerprint \
         {{ request_id }}` with ONLY the standard `requestId` field. \
         The fingerprint read is unconditional — no path override, no \
         force-rehash flag. Adding args here without updating this \
         test would let a refactor silently widen the RPC's surface."
    );
}

#[test]
fn get_config_fingerprint_is_wired_into_request_id_and_command_type() {
    assert!(
        STDIN_COMMANDS.contains("| Self::GetConfigFingerprint { request_id, .. }"),
        "src/stdin_commands/mod.rs `ExternalCommand::request_id()` must \
         include `| Self::GetConfigFingerprint {{ request_id, .. }}` so \
         structured-tracing correlation works. Without this, \
         request_id would log as `None` regardless of the incoming \
         value — automation harnesses could not correlate the receipt."
    );
    assert!(
        STDIN_COMMANDS.contains("Self::GetConfigFingerprint { .. } => \"getConfigFingerprint\","),
        "src/stdin_commands/mod.rs `ExternalCommand::command_type()` must \
         map `Self::GetConfigFingerprint {{ .. }}` to the exact literal \
         string `\"getConfigFingerprint\"`. Agentic-testing harnesses \
         key on this exact string — renaming invalidates any receipt \
         that inspects stdin command_type."
    );
}

#[test]
fn all_three_dispatchers_handle_get_config_fingerprint() {
    // Triple-embedded stdin dispatcher pattern: runtime_stdin.rs,
    // app_run_setup.rs, and the snippet mirror runtime_stdin_match_tail.rs
    // must all include the handler arm. A missing arm silently drops
    // the command from whichever code path the missing file owns.
    for (name, source) in [
        ("src/main_entry/app_run_setup.rs", APP_RUN_SETUP),
        ("src/main_entry/runtime_stdin.rs", RUNTIME_STDIN),
        (
            "src/main_entry/runtime_stdin_match_tail.rs",
            RUNTIME_STDIN_MATCH_TAIL,
        ),
    ] {
        assert!(
            source.contains("ExternalCommand::GetConfigFingerprint { ref request_id }"),
            "{} must contain an `ExternalCommand::GetConfigFingerprint \
             {{ ref request_id }}` arm. The triple-embedded stdin \
             dispatcher pattern means ALL three files must match — \
             otherwise the command is silently dropped from whichever \
             code path the missing file owns.",
            name
        );
        assert!(
            source.contains("crate::config::current_config_fingerprint_receipt()"),
            "{} must call `crate::config::current_config_fingerprint_receipt()` \
             — the single public accessor for the fingerprint. Calling \
             `fingerprint_config_file` or reopening the file directly \
             would bypass the receipt struct's stable shape and drift \
             the serialized JSON that automation keys on.",
            name
        );
        assert!(
            source.contains("event = \"config_fingerprint_result\""),
            "{} must emit `event = \"config_fingerprint_result\"` in both \
             Some and None arms. Automation harnesses key on this exact \
             event name — renaming breaks every receipt parser.",
            name
        );
        assert!(
            source.contains("command = \"getConfigFingerprint\""),
            "{} must emit `command = \"getConfigFingerprint\"` in the \
             tracing field set so the event is correlatable with the \
             command_type() literal. The two must stay in lockstep.",
            name
        );
        assert!(
            source.contains("error_code = \"config_file_missing\""),
            "{} must emit `error_code = \"config_file_missing\"` in the \
             None (file missing / metadata unreadable) branch. \
             Automation distinguishes \"file missing\" from \"command \
             did not land\" by this exact code. Dropping or renaming \
             it would collapse the two failure shapes.",
            name
        );
    }
}

#[test]
fn config_fingerprint_receipt_struct_preserves_wire_shape() {
    // The receipt is serialized directly into the `state` field of
    // the tracing event, so its struct shape IS the wire shape.
    // Automation consumers key on the exact field names.
    assert!(
        CONFIG_LOADER.contains("pub struct ConfigFingerprintReceipt {"),
        "src/config/loader.rs must expose `pub struct \
         ConfigFingerprintReceipt`. Narrowing visibility to pub(crate) \
         or moving it elsewhere would force the stdin dispatchers to \
         reach into private state."
    );
    assert!(
        CONFIG_LOADER.contains("pub path: String,"),
        "src/config/loader.rs `ConfigFingerprintReceipt` must carry a \
         `pub path: String` field. Automation uses this to confirm the \
         receipt reflects the expected `~/.kit/config.ts` file and \
         not some override."
    );
    assert!(
        CONFIG_LOADER.contains("pub len: u64,"),
        "src/config/loader.rs `ConfigFingerprintReceipt` must carry a \
         `pub len: u64` field. `len` is half of the cache-hit check \
         in `try_load_cached_config`; dropping it would break the \
         \"fingerprint changed\" acceptance clause."
    );
    assert!(
        CONFIG_LOADER.contains("pub modified_ms: u64,"),
        "src/config/loader.rs `ConfigFingerprintReceipt` must carry a \
         `pub modified_ms: u64` field. `modified_ms` is the other half \
         of the cache-hit check; dropping it would mean a write that \
         preserves length (e.g. in-place edit of a same-length line) \
         would silently appear unchanged."
    );
    assert!(
        CONFIG_LOADER.contains("pub fingerprint_hash: Option<String>,"),
        "src/config/loader.rs `ConfigFingerprintReceipt` must reserve \
         the `pub fingerprint_hash: Option<String>` slot (currently \
         always None — content hashing is a follow-up). Removing the \
         field now would force a protocol break when the content hash \
         does land; keeping it reserved lets a future pass populate it \
         without widening the wire shape."
    );
    assert!(
        CONFIG_LOADER
            .contains("#[serde(skip_serializing_if = \"Option::is_none\")]\n    pub fingerprint_hash: Option<String>,"),
        "src/config/loader.rs `ConfigFingerprintReceipt.fingerprint_hash` \
         must be `skip_serializing_if = \"Option::is_none\"` so \
         automation receives no `fingerprint_hash` key at all when \
         the content hash is not populated. Emitting `null` would \
         make parsers with strict key-presence checks false-positive \
         on \"hash is unset\"."
    );
    assert!(
        CONFIG_LOADER.contains(
            "pub fn current_config_fingerprint_receipt() -> Option<ConfigFingerprintReceipt>"
        ),
        "src/config/loader.rs must expose `pub fn \
         current_config_fingerprint_receipt() -> \
         Option<ConfigFingerprintReceipt>`. This is the single \
         entry point the stdin dispatchers call — routing through \
         any other helper would bypass the `None on stat failure` \
         invariant that the `config_file_missing` error branch \
         depends on."
    );
    assert!(
        CONFIG_MOD.contains("current_config_fingerprint_receipt")
            && CONFIG_MOD.contains("ConfigFingerprintReceipt"),
        "src/config/mod.rs must re-export `current_config_fingerprint_receipt` \
         AND `ConfigFingerprintReceipt` from `loader` at crate \
         visibility. Without both, `crate::config::current_config_fingerprint_receipt()` \
         from the dispatchers would fail to resolve."
    );
}
