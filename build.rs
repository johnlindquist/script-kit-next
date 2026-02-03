// Build script for script-kit-gpui
//
// This script tells Cargo to rebuild when key files change.
// SDK deployment to ~/.scriptkit is now handled at runtime by setup::ensure_kit_setup()
// rather than at build time, ensuring the SDK is always in sync with the running binary.

fn main() {
    // Expose the git commit hash as a compile-time env var (GIT_HASH).
    // Falls back to "unknown" if not inside a git repo or git is unavailable.
    let git_hash = std::process::Command::new("git")
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
        .unwrap_or_else(|| "unknown".to_string());
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    // Expose the build profile (debug/release) as a compile-time env var (BUILD_PROFILE).
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string());
    println!("cargo:rustc-env=BUILD_PROFILE={}", profile);

    // Rebuild when the git HEAD changes (e.g. new commit, branch switch).
    println!("cargo:rerun-if-changed=.git/HEAD");

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
