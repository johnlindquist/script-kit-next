//! App-side discovery of the installed flows package (`@johnlindquist/flows`).
//!
//! `md roster --json` only sees flows reachable from a cwd; the user's real
//! agent corpus lives in a globally installed bun package whose flows are
//! exposed as bun-linked `flow-*` wrapper commands. This scanner surfaces that
//! corpus in the desk with true provenance (package name) and the wrapper
//! command a conversation should launch.
//!
//! Precedence mirrors the package's own `flow.ts` dispatcher: tracked
//! `flows/` wins over local `.flows/` for the same flow name.

use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::model::{FlowDescriptor, FlowSource};

const PACKAGE_NAME: &str = "@johnlindquist/flows";
const CACHE_TTL: Duration = Duration::from_secs(10);

/// Directory of the installed flows package, or `None` when not installed.
/// `SCRIPT_KIT_FLOWS_PACKAGE_DIR` overrides for tests/probes.
pub fn flows_package_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("SCRIPT_KIT_FLOWS_PACKAGE_DIR") {
        if dir.is_empty() {
            return None;
        }
        let path = PathBuf::from(dir);
        return path.is_dir().then_some(path);
    }
    let home = std::env::var("HOME").ok()?;
    let path = PathBuf::from(home)
        .join(".bun/install/global/node_modules")
        .join(PACKAGE_NAME);
    path.is_dir().then_some(path)
}

/// Directory holding the bun-linked wrapper binaries (`flow-*`).
fn bun_bin_dir() -> Option<PathBuf> {
    if let Ok(dir) = std::env::var("SCRIPT_KIT_FLOWS_BIN_DIR") {
        if dir.is_empty() {
            return None;
        }
        return Some(PathBuf::from(dir));
    }
    let home = std::env::var("HOME").ok()?;
    Some(PathBuf::from(home).join(".bun/bin"))
}

/// Cached package scan. Refreshes at most every `CACHE_TTL`; flow definitions
/// change rarely and the desk re-renders often.
pub fn package_flows() -> Vec<FlowDescriptor> {
    static CACHE: Mutex<Option<(Instant, Vec<FlowDescriptor>)>> = Mutex::new(None);
    let mut cache = CACHE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some((at, flows)) = cache.as_ref() {
        if at.elapsed() < CACHE_TTL {
            return flows.clone();
        }
    }
    let flows = scan_package_flows();
    *cache = Some((Instant::now(), flows.clone()));
    flows
}

/// Uncached scan of the flows package. Tracked `flows/` wins over `.flows/`.
pub fn scan_package_flows() -> Vec<FlowDescriptor> {
    let Some(package_dir) = flows_package_dir() else {
        return Vec::new();
    };
    let bin_dir = bun_bin_dir();
    let mut by_name: std::collections::BTreeMap<String, FlowDescriptor> =
        std::collections::BTreeMap::new();

    // Scan `.flows/` first so `flows/` inserts overwrite it (tracked wins).
    for (dir, origin) in [
        (
            package_dir.join(".flows"),
            format!("{PACKAGE_NAME} (local)"),
        ),
        (package_dir.join("flows"), PACKAGE_NAME.to_string()),
    ] {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(descriptor) = parse_package_flow(&path, &origin, bin_dir.as_deref()) else {
                continue;
            };
            by_name.insert(descriptor.name.clone(), descriptor);
        }
    }

    by_name.into_values().collect()
}

/// Parse one `<name>[.i].<engine>.md` package flow file into a descriptor.
///
/// Metadata is derived from what the file actually declares (filename engine
/// segment / `.i.` marker, frontmatter keys) mirroring mdflow's own ladder —
/// never invented. The old scanner marked EVERY package flow
/// `interactive: true` with a `codex` default engine, which disagreed with
/// mdflow's resolution and broke the desk's interactive-flow terminal
/// fallback (2026-07-11 audit).
fn parse_package_flow(path: &Path, origin: &str, bin_dir: Option<&Path>) -> Option<FlowDescriptor> {
    let file_name = path.file_name()?.to_str()?;
    if !file_name.ends_with(".md") || file_name.starts_with('.') {
        return None;
    }
    let stem = file_name.strip_suffix(".md")?;
    // `<name>[.i][.<engine>]` — engine is the last dot segment when present;
    // a trailing/penultimate `i` segment is the interactive marker.
    let (mut name, engine_from_filename) = match stem.rsplit_once('.') {
        Some((name, segment)) if segment == "i" && !name.is_empty() => (format!("{name}.i"), None),
        Some((name, engine)) if !engine.is_empty() && !name.is_empty() => {
            (name.to_string(), Some(engine.to_string()))
        }
        _ => (stem.to_string(), None),
    };
    let mut interactive = false;
    if let Some(stripped) = name.strip_suffix(".i") {
        name = stripped.to_string();
        interactive = true;
    }

    let source = std::fs::read_to_string(path).ok()?;
    let block = frontmatter_block(&source).unwrap_or("");
    let description = frontmatter_value(block, "description");
    interactive =
        interactive || frontmatter_flag(block, "_interactive") || frontmatter_flag(block, "_i");
    // Engine ladder mirrors mdflow: filename segment > frontmatter > the
    // CLI's own default (`pi`).
    let engine = engine_from_filename
        .or_else(|| frontmatter_value(block, "engine"))
        .or_else(|| frontmatter_value(block, "tool"))
        .unwrap_or_else(|| "pi".to_string());
    let is_workflow = block
        .lines()
        .any(|line| line.trim_start().starts_with("_steps:"));
    let mtime_ms = std::fs::metadata(path)
        .and_then(|meta| meta.modified())
        .ok()
        .and_then(|modified| modified.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(0);

    let wrapper_command = bin_dir
        .map(|dir| dir.join(&name))
        .filter(|bin| bin.exists())
        .map(|_| name.clone());

    Some(FlowDescriptor {
        id: format!("package:{name}"),
        path: path.to_string_lossy().to_string(),
        source: FlowSource::Package,
        name,
        description,
        engine,
        engine_source: None,
        inputs: Vec::new(),
        is_workflow,
        interactive,
        mtime_ms,
        origin: Some(origin.to_string()),
        wrapper_command,
    })
}

/// The YAML frontmatter block between the first `---` pair, skipping an
/// optional shebang line. `None` when the file has no frontmatter.
fn frontmatter_block(source: &str) -> Option<&str> {
    let body = if source.starts_with("#!") {
        source.split_once('\n').map(|(_, rest)| rest).unwrap_or("")
    } else {
        source
    };
    let rest = body.trim_start_matches('\u{feff}').strip_prefix("---")?;
    let end = rest.find("\n---")?;
    Some(&rest[..end])
}

/// Extract a simple single-line frontmatter value (`key: value`) from the
/// frontmatter block, stripping surrounding quotes. Multi-line scalars are
/// honestly reported as absent rather than mis-parsed.
fn frontmatter_value(block: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}:");
    for line in block.lines() {
        let Some(rest) = line.strip_prefix(&prefix) else {
            continue;
        };
        let value = rest.trim().trim_matches('"').trim_matches('\'').trim();
        if value.is_empty() {
            return None;
        }
        return Some(value.to_string());
    }
    None
}

fn frontmatter_flag(block: &str, key: &str) -> bool {
    frontmatter_value(block, key).is_some_and(|value| value.eq_ignore_ascii_case("true"))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `SCRIPT_KIT_FLOWS_PACKAGE_DIR`/`_BIN_DIR` are process-global: tests
    /// that set them must run serially or they race each other's scans.
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn write_flow(dir: &Path, file: &str, description: &str) {
        std::fs::create_dir_all(dir).unwrap();
        std::fs::write(
            dir.join(file),
            format!("#!/usr/bin/env md\n---\ndescription: \"{description}\"\nengine: codex\n---\nbody\n"),
        )
        .unwrap();
    }

    #[test]
    fn scans_package_flows_with_provenance_and_wrapper() {
        let _env = ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let root = std::env::temp_dir().join(format!("sk-flows-pkg-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let pkg = root.join("pkg");
        let bin = root.join("bin");
        write_flow(&pkg.join("flows"), "flow-gmail.codex.md", "Gmail via gog");
        write_flow(
            &pkg.join(".flows"),
            "flow-gmail.codex.md",
            "local override loses",
        );
        write_flow(&pkg.join(".flows"), "flow-scratch.codex.md", "local only");
        std::fs::create_dir_all(&bin).unwrap();
        std::fs::write(bin.join("flow-gmail"), "#!/usr/bin/env bun\n").unwrap();

        // Direct scan against explicit dirs via env override.
        std::env::set_var("SCRIPT_KIT_FLOWS_PACKAGE_DIR", &pkg);
        std::env::set_var("SCRIPT_KIT_FLOWS_BIN_DIR", &bin);
        let flows = scan_package_flows();
        std::env::remove_var("SCRIPT_KIT_FLOWS_PACKAGE_DIR");
        std::env::remove_var("SCRIPT_KIT_FLOWS_BIN_DIR");

        let gmail = flows
            .iter()
            .find(|f| f.name == "flow-gmail")
            .expect("gmail");
        // Tracked flows/ wins over .flows/ for the same name.
        assert_eq!(gmail.description.as_deref(), Some("Gmail via gog"));
        assert_eq!(gmail.engine, "codex");
        assert_eq!(gmail.origin.as_deref(), Some("@johnlindquist/flows"));
        assert_eq!(gmail.wrapper_command.as_deref(), Some("flow-gmail"));
        assert!(
            !gmail.interactive,
            "interactivity is declared, never invented — no `.i.` marker and no `_interactive:` key"
        );
        assert_eq!(gmail.friendly_name(), "Gmail");

        let scratch = flows
            .iter()
            .find(|f| f.name == "flow-scratch")
            .expect("scratch");
        assert_eq!(
            scratch.origin.as_deref(),
            Some("@johnlindquist/flows (local)")
        );
        // No bun bin for it → no wrapper; conversations fall back to `md <path>`.
        assert_eq!(scratch.wrapper_command, None);

        let _ = std::fs::remove_dir_all(&root);
    }

    /// Metadata comes from the file, mirroring mdflow's ladder: `.i.`
    /// marker / `_interactive:` for interactivity, filename > frontmatter >
    /// `pi` for engine, `_steps:` for workflows (2026-07-11 audit: the
    /// scanner used to invent `interactive: true` + `codex` for everything).
    #[test]
    fn package_metadata_is_declared_not_invented() {
        let _env = ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let root = std::env::temp_dir().join(format!("sk-flows-meta-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let pkg = root.join("pkg");
        let flows_dir = pkg.join("flows");
        std::fs::create_dir_all(&flows_dir).unwrap();
        std::fs::write(
            flows_dir.join("flow-tui.i.claude.md"),
            "---\ndescription: needs a terminal\n---\nbody\n",
        )
        .unwrap();
        std::fs::write(
            flows_dir.join("flow-marked.md"),
            "---\ndescription: marked interactive\n_interactive: true\nengine: droid\n---\nbody\n",
        )
        .unwrap();
        std::fs::write(
            flows_dir.join("flow-plain.md"),
            "#!/usr/bin/env md\n---\ndescription: defaults\n---\nbody with description: not-this\n",
        )
        .unwrap();
        std::fs::write(
            flows_dir.join("flow-dag.codex.md"),
            "---\ndescription: pipeline\n_steps:\n  - id: a\n---\nbody\n",
        )
        .unwrap();

        std::env::set_var("SCRIPT_KIT_FLOWS_PACKAGE_DIR", &pkg);
        std::env::set_var("SCRIPT_KIT_FLOWS_BIN_DIR", root.join("nobin"));
        let flows = scan_package_flows();
        std::env::remove_var("SCRIPT_KIT_FLOWS_PACKAGE_DIR");
        std::env::remove_var("SCRIPT_KIT_FLOWS_BIN_DIR");

        let tui = flows.iter().find(|f| f.name == "flow-tui").expect("tui");
        assert!(tui.interactive, "`.i.` filename marker");
        assert_eq!(tui.engine, "claude");

        let marked = flows
            .iter()
            .find(|f| f.name == "flow-marked")
            .expect("marked");
        assert!(marked.interactive, "`_interactive: true` frontmatter");
        assert_eq!(
            marked.engine, "droid",
            "frontmatter engine when no filename segment"
        );

        let plain = flows
            .iter()
            .find(|f| f.name == "flow-plain")
            .expect("plain");
        assert!(!plain.interactive);
        assert_eq!(plain.engine, "pi", "mdflow's own default engine");
        assert_eq!(
            plain.description.as_deref(),
            Some("defaults"),
            "description reads the frontmatter block only, never the body"
        );

        let dag = flows.iter().find(|f| f.name == "flow-dag").expect("dag");
        assert!(dag.is_workflow, "`_steps:` marks a workflow");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn friendly_names_read_like_agent_identities() {
        let mut flow = FlowDescriptor {
            id: "package:flow-npm".into(),
            path: String::new(),
            source: FlowSource::Package,
            name: "flow-npm".into(),
            description: None,
            engine: "codex".into(),
            engine_source: None,
            inputs: Vec::new(),
            is_workflow: false,
            interactive: true,
            mtime_ms: 0,
            origin: None,
            wrapper_command: None,
        };
        assert_eq!(flow.friendly_name(), "NPM");
        flow.name = "flow-browser-automate".into();
        assert_eq!(flow.friendly_name(), "Browser Automate");
        flow.name = "fast-success".into();
        assert_eq!(flow.friendly_name(), "Fast Success");
    }
}
