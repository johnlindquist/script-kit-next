use std::{fs, path::Path};

fn rust_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

#[test]
fn file_search_and_actions_do_not_stage_context_parts_directly() {
    for dir in ["src/file_search", "src/actions"] {
        let mut files = Vec::new();
        rust_files(&Path::new(env!("CARGO_MANIFEST_DIR")).join(dir), &mut files);
        for path in files {
            let src = fs::read_to_string(&path).unwrap();
            assert!(
                !src.contains("thread.add_context_part"),
                "{} must route Agent Chat context through AgentChatEntryRequest/staging helpers",
                path.display()
            );
        }
    }
}
