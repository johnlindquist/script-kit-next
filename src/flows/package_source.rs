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

/// Parse one `<name>.<engine>.md` package flow file into a descriptor.
fn parse_package_flow(path: &Path, origin: &str, bin_dir: Option<&Path>) -> Option<FlowDescriptor> {
    let file_name = path.file_name()?.to_str()?;
    if !file_name.ends_with(".md") || file_name.starts_with('.') {
        return None;
    }
    let stem = file_name.strip_suffix(".md")?;
    // `<name>.<engine>` — engine is the last dot segment when present.
    let (name, engine) = match stem.rsplit_once('.') {
        Some((name, engine)) if !engine.is_empty() && !name.is_empty() => {
            (name.to_string(), engine.to_string())
        }
        _ => (stem.to_string(), "codex".to_string()),
    };

    let source = std::fs::read_to_string(path).ok()?;
    let description = frontmatter_value(&source, "description");
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
        is_workflow: false,
        // Package flows are agent identities; the desk's Enter means
        // "converse", implemented via `--_interactive`.
        interactive: true,
        mtime_ms,
        origin: Some(origin.to_string()),
        wrapper_command,
    })
}

/// Extract a simple single-line frontmatter value (`key: value`), stripping
/// surrounding quotes. Package flows keep `description:` on one line.
fn frontmatter_value(source: &str, key: &str) -> Option<String> {
    let prefix = format!("{key}:");
    for line in source.lines().take(60) {
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

#[cfg(test)]
mod tests {
    use super::*;

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
        assert!(gmail.interactive);
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
