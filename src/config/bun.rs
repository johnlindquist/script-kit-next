//! Bun is selected once per invocation, before a child can change directory or PATH.
//! No shell startup files, manager processes or project-local shims are consulted.

use std::collections::BTreeSet;
use std::env;
use std::io;
use std::path::{Path, PathBuf};

const NAME: &str = if cfg!(windows) { "bun.exe" } else { "bun" };

fn discovery_allowed() -> io::Result<()> {
    crate::runtime_policy::check(crate::runtime_policy::ExternalEffect::SystemDiscovery)
        .map_err(|error| io::Error::new(io::ErrorKind::PermissionDenied, error.to_string()))
}

/// Used by children that need the effective (possibly computed) config.ts value.
/// Never call this from load_config: the evaluator needs its own bootstrap path.
pub fn configured_bun() -> io::Result<PathBuf> {
    discovery_allowed()?;
    resolve_bun(super::load_config().bun_path.as_deref())
}

pub(crate) fn resolve_bun(configured: Option<&str>) -> io::Result<PathBuf> {
    discovery_allowed()?;
    let home = dirs::home_dir().filter(|path| path.is_absolute());
    let mut bins: Vec<PathBuf> = Vec::new();
    if let Some(root) = env::var_os("BUN_INSTALL").map(PathBuf::from) {
        bins.push(root.join("bin"));
    }
    if let Some(home) = &home {
        for relative in [
            ".bun/bin",
            "Library/pnpm",
            ".nvm/current/bin",
            ".volta/bin",
            ".local/bin",
            "bin",
        ] {
            bins.push(home.join(relative));
        }
    }
    bins.extend(["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin", "/bin"].map(PathBuf::from));
    let path: Vec<_> = env::var_os("PATH")
        .as_deref()
        .map(env::split_paths)
        .into_iter()
        .flatten()
        .collect();
    let mise = env::var_os("MISE_DATA_DIR")
        .map(PathBuf::from)
        .or_else(|| env::var_os("XDG_DATA_HOME").map(|dir| PathBuf::from(dir).join("mise")))
        .or_else(|| home.as_ref().map(|dir| dir.join(".local/share/mise")));
    let asdf = env::var_os("ASDF_DATA_DIR")
        .map(PathBuf::from)
        .or_else(|| home.as_ref().map(|dir| dir.join(".asdf")));
    select(
        configured.map(Path::new),
        &bins,
        &path,
        &[mise, asdf].into_iter().flatten().collect::<Vec<_>>(),
        env::current_dir().ok().as_deref(),
    )
}

fn shim_directory(dir: &Path) -> bool {
    let named = |p: &Path, name: &str| {
        p.file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|s| s.eq_ignore_ascii_case(name))
    };
    dir.ancestors().any(|part| {
        named(part, "shims")
            || (named(part, ".bin") && part.parent().is_some_and(|p| named(p, "node_modules")))
    })
}

fn executable(path: &Path) -> Option<PathBuf> {
    // A nested package can inherit its parent's node_modules/.bin. Directory aliases
    // and the link to the program must not hide a shim, even for an explicit pin.
    if !path.is_absolute() || !path.is_file() {
        return None;
    }
    let parent = path.parent()?;
    let resolved = path.canonicalize().ok()?;
    if shim_directory(parent)
        || shim_directory(&parent.canonicalize().ok()?)
        || shim_directory(resolved.parent()?)
    {
        return None;
    }
    // which on an absolute path checks effective executable access; no PATH search.
    which::which(&resolved).ok()
}

fn select(
    configured: Option<&Path>,
    bins: &[PathBuf],
    path: &[PathBuf],
    managers: &[PathBuf],
    cwd: Option<&Path>,
) -> io::Result<PathBuf> {
    if let Some(pin) = configured {
        return executable(pin).ok_or_else(|| {
            io::Error::other(
                "bun_path must name an absolute executable Bun binary, not a manager shim",
            )
        });
    }
    if let Some(binary) = bins.iter().find_map(|dir| executable(&dir.join(NAME))) {
        return Ok(binary);
    }
    // A launcher may inherit an absolute cwd or project bin in PATH as well as `.`.
    let cwd = cwd.and_then(|dir| dir.canonicalize().ok());
    let outside_cwd = |candidate: &Path| {
        cwd.as_ref().is_some_and(|dir| {
            candidate != dir && (dir.parent().is_none() || !candidate.starts_with(dir))
        })
    };
    for dir in path.iter().filter(|dir| dir.is_absolute()) {
        if outside_cwd(dir) && dir.canonicalize().ok().is_some_and(|dir| outside_cwd(&dir)) {
            if let Some(binary) = executable(&dir.join(NAME)).filter(|binary| outside_cwd(binary)) {
                return Ok(binary);
            }
        }
    }
    // Shims can change their target based on the project's .tool-versions or mise.toml.
    // Use the installed binary itself; aliases of one installation count only once.
    let mut installed = BTreeSet::new();
    for root in managers.iter().filter(|root| root.is_absolute()) {
        let installs = root.join("installs/bun");
        let versions = match std::fs::read_dir(&installs) {
            Ok(versions) => versions,
            Err(err) if err.kind() == io::ErrorKind::NotFound => continue,
            Err(err) => return Err(err),
        };
        let installs = installs.canonicalize()?;
        for version in versions {
            if let Some(binary) = executable(&version?.path().join("bin").join(NAME)) {
                if !binary.starts_with(&installs) {
                    return Err(io::Error::other(
                        "Linked Bun installation; set bun_path explicitly",
                    ));
                }
                installed.insert(binary);
            }
        }
    }
    let mut installed = installed.into_iter();
    match (installed.next(), installed.next()) {
        (Some(one), None) => Ok(one),
        (None, _) => Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Bun was not found; install Bun or set bun_path in config.ts",
        )),
        _ => Err(io::Error::other(
            "Multiple managed Bun binaries; set bun_path in config.ts to choose one",
        )),
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::{symlink, PermissionsExt};
    fn install(path: &Path, mode: u32) -> PathBuf {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "#!/bin/sh\nexit 0\n").unwrap();
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).unwrap();
        path.canonicalize().unwrap()
    }

    #[test]
    fn finder_launch_resolves_managed_installs_without_a_shell_or_cwd_shim() {
        let dir = tempfile::tempdir().unwrap();
        let cwd = dir.path().join("checkout");
        let mise = dir.path().join("mise");
        let asdf = dir.path().join("asdf");
        let managers = [mise.clone(), asdf.clone()];
        install(&cwd.join(NAME), 0o700);
        let shim = install(&mise.join("shims").join(NAME), 0o700);
        let path = [
            PathBuf::from(""),
            PathBuf::from("."),
            cwd.clone(),
            mise.join("shims"),
        ];
        let choose = |pin: Option<&Path>| select(pin, &[], &path, &managers, Some(&cwd));
        assert!(choose(None).is_err());
        assert!(choose(Some(&shim)).is_err());
        let first = install(&mise.join("installs/bun/1.2/bin").join(NAME), 0o700);
        symlink(
            mise.join("installs/bun/1.2"),
            mise.join("installs/bun/latest"),
        )
        .unwrap();
        assert_eq!(choose(None).unwrap(), first);
        let second = install(&asdf.join("installs/bun/1.3/bin").join(NAME), 0o700);
        assert!(choose(None).is_err());
        assert_eq!(choose(Some(&second)).unwrap(), second);
        std::fs::remove_file(first).unwrap();
        assert_eq!(choose(None).unwrap(), second);
    }

    #[test]
    fn a_nested_package_and_an_alias_to_a_shim_directory_cannot_pick_the_runtime() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let cwd = root.join("project/packages/example");
        let package_bin = root.join("project/node_modules/.bin");
        let shims = root.join("manager/shims");
        let alias = root.join("looks-direct");
        for dir in [&cwd, &package_bin, &shims] {
            std::fs::create_dir_all(dir).unwrap();
        }
        let direct = install(&root.join("installed/bin").join(NAME), 0o700);
        let wrapper = install(&root.join("manager/dispatch"), 0o700);
        symlink(&wrapper, package_bin.join(NAME)).unwrap();
        symlink(&wrapper, shims.join(NAME)).unwrap();
        symlink(&shims, &alias).unwrap();
        assert!(select(
            None,
            &[],
            &[package_bin.clone(), alias.clone()],
            &[],
            Some(&cwd)
        )
        .is_err());
        assert!(select(Some(&alias.join(NAME)), &[], &[], &[], Some(&cwd)).is_err());
        assert_eq!(
            select(
                None,
                &[],
                &[package_bin, alias, root.join("installed/bin")],
                &[],
                Some(&cwd)
            )
            .unwrap(),
            direct
        );
    }

    #[test]
    fn explicit_pin_is_authoritative_and_invalid_candidates_are_not_executed() {
        let dir = tempfile::tempdir().unwrap();
        let bin = dir.path().join("global/bin");
        let elsewhere = dir.path().join("empty cwd");
        std::fs::create_dir(&elsewhere).unwrap();
        let good = install(&bin.join(NAME), 0o700);
        let custom = install(&dir.path().join("my runtimes/custom-bun"), 0o700);
        let bad = install(&dir.path().join("not-executable"), 0o600);
        let choose = |pin: Option<&Path>| {
            select(pin, &[], std::slice::from_ref(&bin), &[], Some(&elsewhere))
        };
        assert_eq!(choose(None).unwrap(), good);
        assert_eq!(choose(Some(&custom)).unwrap(), custom);
        for invalid in [
            Path::new("bun"),
            Path::new(""),
            bad.as_path(),
            dir.path(),
            &dir.path().join("missing"),
        ] {
            assert!(choose(Some(invalid)).is_err());
        }
    }
}
