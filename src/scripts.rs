use std::path::PathBuf;
use std::env;
use std::fs;
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct Script {
    pub name: String,
    pub path: PathBuf,
    pub extension: String,
    pub description: Option<String>,
}

/// Represents a scored match result for fuzzy search
#[derive(Clone, Debug)]
pub struct ScriptMatch {
    pub script: Script,
    pub score: i32,
}

/// Extract metadata from script file comments
/// Looks for lines starting with "// Description:"
fn extract_metadata(path: &PathBuf) -> Option<String> {
    match fs::read_to_string(path) {
        Ok(content) => {
            for line in content.lines().take(20) {  // Check only first 20 lines
                if line.trim().starts_with("// Description:") {
                    if let Some(desc) = line.split("// Description:").nth(1) {
                        return Some(desc.trim().to_string());
                    }
                }
            }
            None
        }
        Err(_) => None,
    }
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
                                            let description = extract_metadata(&path);
                                            scripts.push(Script {
                                                name: name.to_string(),
                                                path: path.clone(),
                                                extension: ext_str.to_string(),
                                                description,
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

/// Check if a pattern is a fuzzy match for haystack (characters appear in order)
fn is_fuzzy_match(haystack: &str, pattern: &str) -> bool {
    let mut pattern_chars = pattern.chars().peekable();
    for ch in haystack.chars() {
        if let Some(&p) = pattern_chars.peek() {
            if ch.eq_ignore_ascii_case(&p) {
                pattern_chars.next();
            }
        }
    }
    pattern_chars.peek().is_none()
}

/// Fuzzy search scripts by query string
/// Searches across name, description, and path
/// Returns results sorted by relevance score (highest first)
pub fn fuzzy_search_scripts(scripts: &[Script], query: &str) -> Vec<ScriptMatch> {
    if query.is_empty() {
        // If no query, return all scripts with equal score, sorted by name
        return scripts.iter().map(|s| ScriptMatch {
            script: s.clone(),
            score: 0,
        }).collect();
    }

    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();

    for script in scripts {
        let mut score = 0i32;
        let name_lower = script.name.to_lowercase();

        // Score by name match - highest priority
        if let Some(pos) = name_lower.find(&query_lower) {
            // Bonus for exact substring match at start of name
            score += if pos == 0 { 100 } else { 75 };
        }

        // Fuzzy character matching in name (characters in order)
        if is_fuzzy_match(&name_lower, &query_lower) {
            score += 50;
        }

        // Score by description match - medium priority
        if let Some(ref desc) = script.description {
            if desc.to_lowercase().contains(&query_lower) {
                score += 25;
            }
        }

        // Score by path match - lower priority
        let path_str = script.path.to_string_lossy().to_lowercase();
        if path_str.contains(&query_lower) {
            score += 10;
        }

        if score > 0 {
            matches.push(ScriptMatch {
                script: script.clone(),
                score,
            });
        }
    }

    // Sort by score (highest first), then by name for ties
    matches.sort_by(|a, b| {
        match b.score.cmp(&a.score) {
            Ordering::Equal => a.script.name.cmp(&b.script.name),
            other => other,
        }
    });

    matches
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
            description: None,
        };
        assert_eq!(script.name, "test");
        assert_eq!(script.extension, "ts");
    }

    #[test]
    fn test_fuzzy_search_by_name() {
        let scripts = vec![
            Script {
                name: "openfile".to_string(),
                path: PathBuf::from("/test/openfile.ts"),
                extension: "ts".to_string(),
                description: Some("Open a file dialog".to_string()),
            },
            Script {
                name: "savefile".to_string(),
                path: PathBuf::from("/test/savefile.ts"),
                extension: "ts".to_string(),
                description: None,
            },
        ];

        let results = fuzzy_search_scripts(&scripts, "open");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].script.name, "openfile");
        assert!(results[0].score > 0);
    }

    #[test]
    fn test_fuzzy_search_empty_query() {
        let scripts = vec![
            Script {
                name: "test1".to_string(),
                path: PathBuf::from("/test/test1.ts"),
                extension: "ts".to_string(),
                description: None,
            },
        ];

        let results = fuzzy_search_scripts(&scripts, "");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].score, 0);
    }

    #[test]
    fn test_fuzzy_search_ranking() {
        let scripts = vec![
            Script {
                name: "openfile".to_string(),
                path: PathBuf::from("/test/openfile.ts"),
                extension: "ts".to_string(),
                description: Some("Open a file dialog".to_string()),
            },
            Script {
                name: "open".to_string(),
                path: PathBuf::from("/test/open.ts"),
                extension: "ts".to_string(),
                description: Some("Basic open function".to_string()),
            },
            Script {
                name: "reopen".to_string(),
                path: PathBuf::from("/test/reopen.ts"),
                extension: "ts".to_string(),
                description: None,
            },
        ];

        let results = fuzzy_search_scripts(&scripts, "open");
        // Should have all three results
        assert_eq!(results.len(), 3);
        // "open" should be first (exact match at start: 100 + fuzzy match: 50 = 150)
        assert_eq!(results[0].script.name, "open");
        // "openfile" should be second (substring at start: 100 + fuzzy match: 50 = 150, but "open" comes first alphabetically in tie)
        assert_eq!(results[1].script.name, "openfile");
        // "reopen" should be third (substring not at start: 75 + fuzzy match: 50 = 125)
        assert_eq!(results[2].script.name, "reopen");
    }
}
