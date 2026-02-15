use std::fs;
use std::path::{Path, PathBuf};

const COMPONENT_ROOT: &str = "src/components";
const FORBIDDEN_PATTERNS: [&str; 5] = ["unwrap(", "expect(", "panic!(", "todo!(", "unreachable!("];

#[derive(Debug)]
struct ForbiddenPatternMatch {
    path: PathBuf,
    line: usize,
    pattern: &'static str,
    source: String,
}

fn collect_rust_sources(dir: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_rust_sources(&path, files)?;
            continue;
        }

        let is_rust_source = path.extension().and_then(|ext| ext.to_str()) == Some("rs");
        if is_rust_source {
            files.push(path);
        }
    }

    Ok(())
}

fn is_test_source(path: &Path) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .map(|name| name == "tests.rs" || name.ends_with("_tests.rs"))
        .unwrap_or(false)
}

fn find_forbidden_patterns(path: &Path) -> std::io::Result<Vec<ForbiddenPatternMatch>> {
    let source = fs::read_to_string(path)?;
    let source = source.split("#[cfg(test)]").next().unwrap_or(&source);
    let mut matches = Vec::new();

    for (line_index, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }

        for pattern in FORBIDDEN_PATTERNS {
            if line.contains(pattern) {
                matches.push(ForbiddenPatternMatch {
                    path: path.to_path_buf(),
                    line: line_index + 1,
                    pattern,
                    source: line.trim().to_string(),
                });
            }
        }
    }

    Ok(matches)
}

#[test]
fn test_component_non_test_sources_do_not_panic_or_unwrap() {
    let mut sources = Vec::new();
    collect_rust_sources(Path::new(COMPONENT_ROOT), &mut sources)
        .expect("failed to enumerate component sources");
    sources.retain(|path| !is_test_source(path));
    sources.sort();

    let mut matches = Vec::new();
    for path in sources {
        matches.extend(
            find_forbidden_patterns(&path)
                .unwrap_or_else(|_| panic!("failed to read {}", path.display())),
        );
    }

    let summary = matches
        .iter()
        .map(|entry| {
            format!(
                "{}:{} pattern=`{}` source=`{}`",
                entry.path.display(),
                entry.line,
                entry.pattern,
                entry.source
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        matches.is_empty(),
        "Found panic-prone patterns in non-test component code:\n{}",
        summary
    );
}
