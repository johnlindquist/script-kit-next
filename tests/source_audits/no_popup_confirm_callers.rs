//! Regression test: all app confirmations should use the shared parent dialog path.
//! The old separate popup window (`open_confirm_window`) has been replaced by
//! in-window parent dialogs via `open_parent_confirm_dialog`.

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    fn collect_rs_files(root: &Path, files: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(root).expect("read_dir failed") {
            let entry = entry.expect("dir entry failed");
            let path = entry.path();

            if path.is_dir() {
                collect_rs_files(&path, files);
            } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                files.push(path);
            }
        }
    }

    #[test]
    fn repo_has_no_open_confirm_window_callers() {
        let mut files = Vec::new();
        collect_rs_files(Path::new("src"), &mut files);

        let offenders: Vec<String> = files
            .into_iter()
            .filter_map(|path| {
                let source = fs::read_to_string(&path).expect("read_to_string failed");
                source
                    .contains("open_confirm_window(")
                    .then(|| path.display().to_string())
            })
            .collect();

        assert!(
            offenders.is_empty(),
            "remaining open_confirm_window callers: {:?}",
            offenders
        );
    }

    #[test]
    fn repo_has_no_dispatch_confirm_key_callers() {
        let mut files = Vec::new();
        collect_rs_files(Path::new("src"), &mut files);

        let offenders: Vec<String> = files
            .into_iter()
            .filter_map(|path| {
                let source = fs::read_to_string(&path).expect("read_to_string failed");
                source
                    .contains("dispatch_confirm_key(")
                    .then(|| path.display().to_string())
            })
            .collect();

        assert!(
            offenders.is_empty(),
            "remaining dispatch_confirm_key callers: {:?}",
            offenders
        );
    }

    #[test]
    fn notes_delete_uses_entity_owned_confirm_helper() {
        let source = fs::read_to_string("src/notes/window/notes.rs")
            .expect("Failed to read src/notes/window/notes.rs");
        let normalized: String = source.split_whitespace().collect::<Vec<_>>().join(" ");

        assert!(
            normalized.contains("crate::confirm::open_parent_confirm_dialog_for_entity("),
            "Notes delete should use the entity-owned parent confirm helper"
        );
    }
}
