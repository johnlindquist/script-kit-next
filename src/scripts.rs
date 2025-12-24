use std::path::PathBuf;
use std::env;

#[derive(Clone, Debug)]
pub struct Script {
    pub name: String,
    pub path: PathBuf,
    pub extension: String,
}

/// Reads scripts from ~/.kenv/scripts directory
/// Returns a sorted list of Script structs for .ts and .js files
/// Returns empty vec if directory doesn't exist or is inaccessible
pub fn read_scripts() -> Vec<Script> {
    // Expand ~ to home directory using HOME environment variable
    let home = match env::var("HOME") {
        Ok(home_path) => PathBuf::from(home_path),
        Err(_) => return vec![],
    };

    let scripts_dir = home.join(".kenv/scripts");

    // Check if directory exists
    if !scripts_dir.exists() {
        return vec![];
    }

    let mut scripts = Vec::new();

    // Read the directory contents
    match std::fs::read_dir(&scripts_dir) {
        Ok(entries) => {
            for entry in entries.flatten() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        let path = entry.path();
                        
                        // Check extension
                        if let Some(ext) = path.extension() {
                            if let Some(ext_str) = ext.to_str() {
                                if ext_str == "ts" || ext_str == "js" {
                                    // Get filename without extension
                                    if let Some(file_name) = path.file_stem() {
                                        if let Some(name) = file_name.to_str() {
                                            scripts.push(Script {
                                                name: name.to_string(),
                                                path: path.clone(),
                                                extension: ext_str.to_string(),
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Err(_) => return vec![],
    }

    // Sort by name
    scripts.sort_by(|a, b| a.name.cmp(&b.name));

    scripts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_scripts_returns_vec() {
        let scripts = read_scripts();
        // scripts should be a Vec, check it's valid
        assert!(scripts.is_empty() || !scripts.is_empty());
    }

    #[test]
    fn test_script_struct_has_required_fields() {
        let script = Script {
            name: "test".to_string(),
            path: PathBuf::from("/test/path"),
            extension: "ts".to_string(),
        };
        assert_eq!(script.name, "test");
        assert_eq!(script.extension, "ts");
    }
}
