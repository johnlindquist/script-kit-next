const STDIN_COMMANDS: &str = include_str!("../src/stdin_commands/mod.rs");

#[test]
fn stdin_dispatch_uses_core_protocol_version_reader() {
    assert!(
        STDIN_COMMANDS.contains(
            "use crate::protocol::version::{read_wire_version, ProtocolVersion, ProtocolVersionError};"
        ),
        "stdin command parsing must share the core protocol version reader"
    );
    assert!(
        STDIN_COMMANDS.contains("fn parse_protocol_version(")
            && STDIN_COMMANDS.contains("read_wire_version(raw)"),
        "parse_protocol_version should delegate to read_wire_version"
    );
    assert!(
        !STDIN_COMMANDS.contains("pub const STDIN_PROTOCOL_VERSION: u16 = 1"),
        "stdin dispatch must not keep a v1-only protocol gate"
    );
}

#[test]
fn unsupported_version_stats_only_count_out_of_range_versions() {
    let unsupported_arm = STDIN_COMMANDS
        .split("let version = parse_protocol_version(&raw).inspect_err(|err| {")
        .nth(1)
        .expect("parse_stdin_command should inspect protocol version errors")
        .split("})?;")
        .next()
        .expect("protocol version inspect_err block should be bounded");
    assert!(
        unsupported_arm.contains("if let ProtocolVersionError::Unsupported { found } = err"),
        "unsupported-version counter should be guarded by the Unsupported error variant"
    );
    assert!(
        unsupported_arm.contains("stdin_unsupported_protocol_version_total"),
        "unsupported out-of-range versions should increment protocol stats"
    );
}
