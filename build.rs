// Build script for script-kit-gpui
//
// This script tells Cargo to rebuild when key files change.
// SDK deployment to ~/.scriptkit is now handled at runtime by setup::ensure_kit_setup()
// rather than at build time, ensuring the SDK is always in sync with the running binary.

use std::path::PathBuf;
use std::process::Command;

fn read_git_hash() -> Option<String> {
    Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() {
                String::from_utf8(out.stdout)
                    .ok()
                    .map(|s| s.trim().to_string())
            } else {
                None
            }
        })
}

fn resolve_git_dir() -> Option<PathBuf> {
    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() {
                String::from_utf8(out.stdout)
                    .ok()
                    .map(|s| PathBuf::from(s.trim()))
            } else {
                None
            }
        })
}

fn emit_git_rerun_triggers() {
    if let Some(git_dir) = resolve_git_dir() {
        let head_path = git_dir.join("HEAD");
        println!("cargo:rerun-if-changed={}", head_path.display());
        println!(
            "cargo:rerun-if-changed={}",
            git_dir.join("packed-refs").display()
        );

        if let Ok(head_contents) = std::fs::read_to_string(&head_path) {
            if let Some(reference_path) = head_contents.strip_prefix("ref:").map(str::trim) {
                println!(
                    "cargo:rerun-if-changed={}",
                    git_dir.join(reference_path).display()
                );
            }
        }
    } else {
        // Fallback for environments where git is unavailable.
        println!("cargo:rerun-if-changed=.git/HEAD");
        println!("cargo:rerun-if-changed=.git/packed-refs");
    }
}

fn main() {
    // Expose the git commit hash as a compile-time env var (GIT_HASH).
    // Falls back to CI-provided SHA or "unknown" if git is unavailable.
    let git_hash = read_git_hash()
        .or_else(|| {
            std::env::var("GITHUB_SHA")
                .ok()
                .map(|sha| sha.chars().take(7).collect())
        })
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=GIT_HASH={git_hash}");

    // Expose the build profile (debug/release) as a compile-time env var (BUILD_PROFILE).
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=BUILD_PROFILE={profile}");

    // Track VCS state changes for commit hash propagation in regular repos and worktrees.
    emit_git_rerun_triggers();
    println!("cargo:rerun-if-env-changed=GITHUB_SHA");

    // Trigger rebuild when SDK source changes (it's embedded via include_str!)
    println!("cargo:rerun-if-changed=scripts/kit-sdk.ts");

    // Trigger rebuild when kit-init files change (embedded and shipped to ~/.scriptkit/)
    println!("cargo:rerun-if-changed=kit-init/config-template.ts");
    println!("cargo:rerun-if-changed=kit-init/theme.example.json");
    println!("cargo:rerun-if-changed=kit-init/GUIDE.md");

    // Trigger rebuild when bundled fonts change (embedded via include_bytes!)
    println!("cargo:rerun-if-changed=assets/fonts/JetBrainsMono-Regular.ttf");
    println!("cargo:rerun-if-changed=assets/fonts/JetBrainsMono-Bold.ttf");
    println!("cargo:rerun-if-changed=assets/fonts/JetBrainsMono-Italic.ttf");
    println!("cargo:rerun-if-changed=assets/fonts/JetBrainsMono-BoldItalic.ttf");
    println!("cargo:rerun-if-changed=assets/fonts/JetBrainsMono-Medium.ttf");
    println!("cargo:rerun-if-changed=assets/fonts/JetBrainsMono-SemiBold.ttf");
}
