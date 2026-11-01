//! Explicit grants bind a main-plugin filename AND its exact source bytes. Capture once, hash,
//! parse and execute that snapshot, never reopen the source after the check. Trusted code (not a
//! security sandbox); automatic package installation, caller argv/env and interactive prompts are absent.
use crate::{mcp_kit_tools::ToolDefinition, schema_parser::Schema};
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{fs, io::Read, path::PathBuf};
pub(crate) fn name_ok(s: &str) -> bool {
    (1..=48).contains(&s.len())
        && s.as_bytes()[0].is_ascii_lowercase()
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}
fn grant(s: &str) -> Option<(&str, &str)> {
    let (name, rev) = s.strip_prefix("command:")?.split_once(':')?;
    (name_ok(name)
        && rev.len() == 64
        && rev
            .bytes()
            .all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f')))
    .then_some((name, rev))
}
struct Reviewed {
    src: String,
    schema: Schema,
    effects: String,
}
fn capture(name: &str, rev: &str) -> Result<Reviewed, &'static str> {
    if !name_ok(name) {
        return Err("invalid_command");
    }
    let dir = crate::setup::get_kit_path().join("plugins/main/scripts");
    let mut opts = fs::OpenOptions::new();
    opts.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        opts.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = opts
        .open(dir.join(format!("{name}.ts")))
        .map_err(|_| "command_unavailable")?;
    let st = file.metadata().map_err(|_| "command_unavailable")?;
    if !st.is_file() || st.len() > 262144 {
        return Err("invalid_command_source");
    }
    let mut bytes = Vec::new();
    file.take(262145)
        .read_to_end(&mut bytes)
        .map_err(|_| "command_unavailable")?;
    if bytes.len() > 262144 || hex::encode(Sha256::digest(&bytes)) != rev {
        return Err("revision_changed");
    }
    let src = String::from_utf8(bytes).map_err(|_| "invalid_command_source")?;
    let effects = src
        .lines()
        .find_map(|l| l.strip_prefix("// Agent effect: "))
        .filter(|s| !s.trim().is_empty() && s.len() <= 320)
        .ok_or("missing_effect_declaration")?
        .to_owned();
    let schema = crate::schema_parser::extract_schema(&src)
        .schema
        .filter(|s| !s.output.is_empty())
        .ok_or("missing_command_schema")?;
    Ok(Reviewed {
        src,
        schema,
        effects,
    })
}
fn closed(mut s: Value) -> Value {
    s["additionalProperties"] = json!(false);
    s
}
fn bun() -> Result<PathBuf, &'static str> {
    which::which("bun").map_err(|_| "bun_unavailable")
}
pub(crate) fn catalog(scopes: &[String]) -> (Vec<Value>, Vec<ToolDefinition>) {
    let mut status = vec![];
    let mut defs = vec![];
    for (name, rev) in scopes.iter().filter_map(|s| grant(s)) {
        match capture(name, rev) {
            Ok(r) => {
                let inp = closed(r.schema.to_json_schema_input());
                let out = closed(r.schema.to_json_schema_output());
                let ready = bun().is_ok();
                status.push(json!({"name":name,"revision":rev,"authorized":true,"ready":ready,"reason":if ready {None}else{Some("bun_unavailable")},"effect":r.effects,"inputSchema":inp,"outputSchema":out}));
                if ready {
                    defs.push(ToolDefinition{name:format!("commands/{name}"),description:r.effects,input_schema:json!({"type":"object","required":["revision","input"],"additionalProperties":false,"properties":{"revision":{"const":rev,"type":"string"},"input":inp}})});
                }
            }
            Err(e) => status.push(
                json!({"name":name,"revision":rev,"authorized":true,"ready":false,"reason":e}),
            ),
        }
    }
    (status, defs)
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Args {
    revision: String,
    input: Value,
}
pub(crate) async fn run(name: &str, args: Value, scopes: &[String]) -> Result<Value, &'static str> {
    let a: Args = serde_json::from_value(args).map_err(|_| "invalid_arguments")?;
    if !scopes.iter().any(|s| grant(s) == Some((name, &a.revision))) {
        return Err("scope_denied");
    }
    let r = capture(name, &a.revision)?;
    let inp = closed(r.schema.to_json_schema_input());
    let out = closed(r.schema.to_json_schema_output());
    let input = serde_json::to_vec(&a.input).map_err(|_| "invalid_input")?;
    if input.len() > 65536
        || !jsonschema::validator_for(&inp)
            .map_err(|_| "invalid_schema")?
            .is_valid(&a.input)
    {
        return Err("invalid_input");
    }
    let validator = jsonschema::validator_for(&out).map_err(|_| "invalid_schema")?;
    crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::Process)
        .map_err(|_| "process_forbidden")?;
    let bin = bun()?;
    let tmp = tempfile::tempdir().map_err(|_| "scratch_unavailable")?;
    let put = |p: &str, bytes: &[u8]| {
        fs::write(tmp.path().join(p), bytes).map_err(|_| "scratch_unavailable")
    };
    fs::create_dir_all(tmp.path().join("node_modules/@scriptkit/sdk"))
        .map_err(|_| "scratch_unavailable")?;
    put(
        "node_modules/@scriptkit/sdk/package.json",
        br#"{"type":"module","exports":"./index.js"}"#,
    )?;
    put(
        "node_modules/@scriptkit/sdk/index.js",
        br#"export const {input,output,defineSchema}=globalThis;"#,
    )?;
    put("command.ts", r.src.as_bytes())?;
    put("bunfig.toml", b"")?;
    put("run.ts", include_bytes!("runner.ts"))?;
    let bytes = super::process::output(
        &bin,
        &[
            "--no-env-file",
            "--no-install",
            "--config=./bunfig.toml",
            "run.ts",
        ],
        tmp.path(),
        &super::process::environment(&[
            "HOME",
            "PATH",
            "TMPDIR",
            "LANG",
            "SK_PATH",
            "SCRIPT_KIT_NONINTERACTIVE",
        ]),
        input,
        10,
        65536,
    )
    .await?;
    let result: Value =
        serde_json::from_slice(&bytes).map_err(|_| "invalid_output_delivery_unknown")?;
    if !validator.is_valid(&result) {
        return Err("invalid_output_delivery_unknown");
    }
    Ok(json!({"ok":true,"state":"completed","name":name,"revision":a.revision,"output":result}))
}
