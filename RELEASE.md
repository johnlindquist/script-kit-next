# Release Process

This document describes the versioning, building, and release process for Script Kit GPUI.

## Overview

| Trigger | Workflow | Signing | Output |
|---------|----------|---------|--------|
| Push to `main` | CI | Ad-hoc (dev) | Downloadable artifact (14 days) |
| Push tag `v*` | Release | Developer ID + Notarized | GitHub Release |

## Versioning

We use [Semantic Versioning](https://semver.org/):

```
v{MAJOR}.{MINOR}.{PATCH}[-{PRERELEASE}]
```

| Component | When to Increment |
|-----------|-------------------|
| **MAJOR** | Breaking changes to scripts/SDK API |
| **MINOR** | New features, backwards compatible |
| **PATCH** | Bug fixes, backwards compatible |
| **PRERELEASE** | Alpha/beta releases (e.g., `v1.0.0-beta.1`) |

### Examples

- `v1.0.0` - First stable release
- `v1.1.0` - Added new prompt type
- `v1.1.1` - Fixed bug in editor prompt
- `v2.0.0` - Changed script protocol (breaking)
- `v2.0.0-alpha.1` - Pre-release for testing

## Development Builds (CI)

Every push to `main` that passes all checks produces a downloadable artifact.

### What Happens

1. CI runs: `check`, `clippy`, `test`, `fmt`
2. If all pass, `build-artifact` job runs
3. Builds release binary and `.app` bundle
4. Signs with ad-hoc signature (for local testing only)
5. Uploads as GitHub Actions artifact

### Downloading Dev Builds

1. Go to [Actions tab](../../actions)
2. Click on the workflow run for your commit
3. Scroll to **Artifacts** section
4. Download `script-kit-macos-{commit-sha}`

### Limitations of Dev Builds

- Ad-hoc signed (shows security warning on first launch)
- Not notarized (Gatekeeper may block)
- To run: Right-click → Open, or `xattr -cr "Script Kit.app"`
- Artifacts expire after 14 days

## Production Releases

Tagged releases are fully signed and notarized for distribution.

### Prerequisites

Before your first release, ensure these GitHub secrets are configured:

| Secret | Description | How to Get |
|--------|-------------|------------|
| `APPLE_CERTIFICATE_BASE64` | Base64-encoded .p12 certificate | Export from Keychain, run `base64 -i cert.p12` |
| `APPLE_CERTIFICATE_PASSWORD` | Password for .p12 file | Set when exporting from Keychain |
| `APPLE_ID` | Your Apple ID email | Your Apple Developer account email |
| `APPLE_APP_PASSWORD` | App-specific password | [appleid.apple.com](https://appleid.apple.com) → App-Specific Passwords |
| `APPLE_TEAM_ID` | 10-character Team ID | [developer.apple.com/account](https://developer.apple.com/account) → Membership |

### Local Pre-Tag Check

```bash
make ship-check
```

### Creating a Release

```bash
# 1. Ensure you're on main with latest changes
git checkout main
git pull origin main

# 2. Update version in Cargo.toml (if needed)
# version = "1.0.0"

# 3. Commit version bump
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to 1.0.0"
git push origin main

# 4. Run the full ship gate before tagging
make ship-check

# 5. Create and push tag
git tag v1.0.0
git push origin v1.0.0
```

### What Happens During Release

1. **Build** - Compiles release binary, creates `.app` bundle
2. **Keychain Setup** - Creates temporary keychain, imports certificate
3. **Code Signing** - Signs with Developer ID Application certificate
   - Signs nested components (dylibs, frameworks)
   - Signs main executable with entitlements
   - Signs app bundle with hardened runtime
4. **Notarization** - Submits to Apple's notary service
   - Waits for Apple's response (typically 2-10 minutes)
   - Fails the build if notarization is rejected
5. **Stapling** - Attaches notarization ticket to app
6. **Release** - Creates GitHub Release with signed `.zip`

### Release Types

The workflow automatically detects pre-releases:

| Tag | Release Type |
|-----|--------------|
| `v1.0.0` | Full release |
| `v1.0.0-alpha.1` | Pre-release (marked in GitHub) |
| `v1.0.0-beta.2` | Pre-release |
| `v1.0.0-rc.1` | Pre-release |

Pre-releases are marked with a banner in GitHub and don't show as "Latest".

## Troubleshooting

### Notarization Failed

Check the workflow logs for the specific error. Common issues:

| Error | Solution |
|-------|----------|
| "Invalid credentials" | Verify `APPLE_ID` and `APPLE_APP_PASSWORD` secrets |
| "No signing identity found" | Check `APPLE_CERTIFICATE_BASE64` is correct |
| "The signature is invalid" | Ensure entitlements.plist is present and valid |
| "Hardened runtime not enabled" | Check codesign uses `--options runtime` |

To debug locally:
```bash
# Check your signing identity
security find-identity -v -p codesigning

# Test notarization credentials
xcrun notarytool history --apple-id "your@email.com" --password "app-specific-password" --team-id "TEAM_ID"
```

### Code Signing Failed

```bash
# Verify certificate is valid
security find-identity -v -p codesigning | grep "Developer ID Application"

# Check certificate expiration
security find-certificate -c "Developer ID Application" -p | openssl x509 -noout -enddate
```

### App Won't Open After Download

For development builds:
```bash
# Remove quarantine attribute
xattr -cr "/Applications/Script Kit.app"
```

For release builds, notarization should handle this. If not:
```bash
# Check if notarization ticket is stapled
xcrun stapler validate "/Applications/Script Kit.app"
```

## Manual Release (Emergency)

If CI is broken, you can build and sign locally:

```bash
# Validate the exact ship path first
make ship-check

# Build/bundle explicitly for the app target
cargo build --release --bin script-kit-gpui
cargo bundle --release --bin script-kit-gpui
bash scripts/verify-macos-bundle.sh

# Sign (replace IDENTITY with your certificate name)
IDENTITY="Developer ID Application: Your Name (TEAM_ID)"
APP="target/release/bundle/osx/Script Kit.app"

codesign --force --options runtime --entitlements entitlements.plist --sign "$IDENTITY" "$APP"

# Create zip
cd target/release/bundle/osx
ditto -c -k --keepParent "Script Kit.app" "Script-Kit-macos.zip"

# Notarize
xcrun notarytool submit "Script-Kit-macos.zip" \
  --apple-id "your@email.com" \
  --password "app-specific-password" \
  --team-id "TEAM_ID" \
  --wait

# Staple
xcrun stapler staple "Script Kit.app"

# Re-zip for distribution
rm Script-Kit-macos.zip
ditto -c -k --keepParent "Script Kit.app" "Script-Kit-macos.zip"
```

Then create the release manually on GitHub and upload the zip.

## Certificate Renewal

Apple Developer ID certificates expire after 5 years. To renew:

1. Create new certificate at [developer.apple.com](https://developer.apple.com/account/resources/certificates)
2. Export as .p12 from Keychain Access
3. Update `APPLE_CERTIFICATE_BASE64` secret:
   ```bash
   base64 -i NewCertificate.p12 | pbcopy
   ```
4. Update `APPLE_CERTIFICATE_PASSWORD` if changed

## Quick Reference

```bash
# Check current version
grep '^version' Cargo.toml

# List recent tags
git tag -l --sort=-v:refname | head -5

# Delete a tag (if needed)
git tag -d v1.0.0
git push origin :refs/tags/v1.0.0

# View release workflow status
gh run list --workflow=release.yml

# Download latest release artifact
gh release download --pattern "*.zip"
```
