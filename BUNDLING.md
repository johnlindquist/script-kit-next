# Cross-Platform App Bundling Guide

This document outlines the strategy and process for bundling Script Kit GPUI for distribution on macOS, Windows, and Linux.

## Table of Contents

1. [Recommended Approach](#recommended-approach)
2. [How Zed Handles Distribution](#how-zed-handles-distribution)
3. [Tooling Options](#tooling-options)
4. [macOS Bundling](#macos-bundling)
5. [Icon Generation](#icon-generation)
6. [Code Signing & Notarization](#code-signing--notarization)
7. [Platform-Specific Considerations](#platform-specific-considerations)
8. [Quick Start Guide](#quick-start-guide)
9. [CI/CD Integration](#cicd-integration)

---

## Recommended Approach

**Primary Tool: `cargo-bundle` (Zed's fork)**

We recommend using Zed's fork of cargo-bundle since:
1. We're already using GPUI (Zed's framework)
2. Zed has battle-tested their bundling process
3. It's specifically designed for Rust GUI apps
4. Direct integration with Cargo.toml metadata

**Install the Zed fork:**
```bash
cargo install cargo-bundle --git https://github.com/zed-industries/cargo-bundle.git --branch zed-deploy
```

---

## How Zed Handles Distribution

Zed's distribution process (from `script/bundle-mac`) provides an excellent reference:

### macOS Bundle Process

1. **Build the binary**
   ```bash
   cargo build --release --package zed --target aarch64-apple-darwin
   ```

2. **Create the .app bundle**
   ```bash
   cargo bundle --release --target aarch64-apple-darwin --select-workspace-root
   ```

3. **Add auxiliary binaries** (git, CLI tool) to `Contents/MacOS/`

4. **Code sign all binaries**
   ```bash
   /usr/bin/codesign --deep --force --timestamp --options runtime \
     --entitlements resources/zed.entitlements \
     --sign "$IDENTITY" "${app_path}"
   ```

5. **Create DMG with license agreement**
   ```bash
   hdiutil create -volname Zed -srcfolder "${dmg_source}" -ov -format UDZO "${dmg_path}"
   ```

6. **Notarize with Apple**
   ```bash
   notarytool submit --wait --key "$key" --key-id "$key_id" --issuer "$issuer" "${dmg_path}"
   stapler staple "${dmg_path}"
   ```

### Key Zed Cargo.toml Configuration

```toml
[package.metadata.bundle-stable]
icon = ["resources/app-icon@2x.png", "resources/app-icon.png"]
identifier = "dev.zed.Zed"
name = "Zed"
osx_minimum_system_version = "10.15.7"
osx_info_plist_exts = ["resources/info/*"]
osx_url_schemes = ["zed"]
```

---

## Tooling Options

### 1. cargo-bundle (Recommended)

**Pros:**
- Native Cargo integration
- Minimal dependencies
- Zed uses this successfully
- Supports macOS .app, Windows .msi, Linux .deb

**Cons:**
- Limited to these formats
- Requires separate tool for DMG creation

### 2. Tauri Bundler

**Pros:**
- Comprehensive format support (DMG, AppImage, Snap, MSI, NSIS)
- Built-in code signing support
- Active development

**Cons:**
- Designed for Tauri apps (may need adaptation)
- Additional complexity
- Web-focused architecture

### 3. Manual Bundling

**Pros:**
- Full control
- No dependencies

**Cons:**
- More work
- Easy to get wrong

### Recommendation

**Use cargo-bundle for the initial bundle, then custom scripts for:**
- DMG creation with license
- Notarization
- CI/CD automation

This matches Zed's approach.

---

## macOS Bundling

### Bundle Structure

A macOS .app bundle has this structure:

```
ScriptKit.app/
├── Contents/
│   ├── Info.plist          # App metadata
│   ├── MacOS/
│   │   ├── script-kit-gpui # Main binary
│   │   └── bun             # Bundled bun runtime (optional)
│   ├── Resources/
│   │   ├── AppIcon.icns    # App icon
│   │   └── sdk/            # Bundled SDK files
│   ├── Frameworks/         # Bundled frameworks (if any)
│   └── _CodeSignature/     # Code signature
```

### Cargo.toml Configuration

Add this to your `Cargo.toml`:

```toml
[package.metadata.bundle]
name = "Script Kit"
identifier = "com.scriptkit.app"
icon = ["assets/icon@2x.png", "assets/icon.png"]
version = "0.1.0"
copyright = "Copyright (c) 2024 Script Kit. All rights reserved."
category = "public.app-category.developer-tools"
short_description = "Automation made simple"
osx_minimum_system_version = "10.15"
osx_url_schemes = ["scriptkit"]

# Resources to bundle
resources = ["assets/*"]
```

### Build Command

```bash
# Build release binary
cargo build --release

# Create .app bundle
cargo bundle --release
```

The bundle will be created at `target/release/bundle/osx/Script Kit.app`

---

## Icon Generation

### Required Icon Sizes

#### macOS (.icns)

The ICNS format bundles multiple sizes. Required sizes:

| Size | Filename | Purpose |
|------|----------|---------|
| 16x16 | icon_16x16.png | Small icon |
| 16x16@2x | icon_16x16@2x.png | Small icon (Retina) |
| 32x32 | icon_32x32.png | Standard icon |
| 32x32@2x | icon_32x32@2x.png | Standard (Retina) |
| 128x128 | icon_128x128.png | Large icon |
| 128x128@2x | icon_128x128@2x.png | Large (Retina) |
| 256x256 | icon_256x256.png | Extra large |
| 256x256@2x | icon_256x256@2x.png | Extra large (Retina) |
| 512x512 | icon_512x512.png | Huge |
| 512x512@2x | icon_512x512@2x.png | Huge (Retina) = 1024x1024 |

#### Windows (.ico)

| Size | Purpose |
|------|---------|
| 16x16 | Small icon |
| 24x24 | Toolbar |
| 32x32 | Standard |
| 48x48 | Large |
| 64x64 | Extra large |
| 256x256 | Vista+ high-res |

#### Linux (PNG set)

| Size | Purpose |
|------|---------|
| 16x16 | Menu icons |
| 22x22 | Small icons |
| 24x24 | Panel icons |
| 32x32 | Desktop icons |
| 48x48 | Large icons |
| 64x64 | Dialog icons |
| 128x128 | High-res |
| 256x256 | Thumbnails |
| 512x512 | Large thumbnails |

### Icon Generation Workflow

#### Option 1: Using the `icns` crate (Rust)

cargo-bundle can automatically convert PNG to ICNS. Provide PNGs with `@2x` suffix for Retina:

```toml
[package.metadata.bundle]
icon = [
    "assets/icon.png",      # 512x512 base icon
    "assets/icon@2x.png"    # 1024x1024 Retina icon
]
```

#### Option 2: Manual Generation (Recommended for Quality)

From a 1024x1024 source SVG/PNG:

```bash
# Create iconset directory
mkdir MyIcon.iconset

# Generate all sizes (using sips on macOS or ImageMagick)
sips -z 16 16     icon-1024.png --out MyIcon.iconset/icon_16x16.png
sips -z 32 32     icon-1024.png --out MyIcon.iconset/icon_16x16@2x.png
sips -z 32 32     icon-1024.png --out MyIcon.iconset/icon_32x32.png
sips -z 64 64     icon-1024.png --out MyIcon.iconset/icon_32x32@2x.png
sips -z 128 128   icon-1024.png --out MyIcon.iconset/icon_128x128.png
sips -z 256 256   icon-1024.png --out MyIcon.iconset/icon_128x128@2x.png
sips -z 256 256   icon-1024.png --out MyIcon.iconset/icon_256x256.png
sips -z 512 512   icon-1024.png --out MyIcon.iconset/icon_256x256@2x.png
sips -z 512 512   icon-1024.png --out MyIcon.iconset/icon_512x512.png
sips -z 1024 1024 icon-1024.png --out MyIcon.iconset/icon_512x512@2x.png

# Convert to icns
iconutil -c icns MyIcon.iconset
```

#### Option 3: Using ImageMagick (Cross-Platform)

```bash
# Generate all PNG sizes
for size in 16 32 64 128 256 512 1024; do
    convert icon.svg -resize ${size}x${size} icon_${size}x${size}.png
done

# For Windows ICO
convert icon_16x16.png icon_32x32.png icon_48x48.png icon_256x256.png icon.ico
```

### Current Project Icon

The project has `assets/logo.svg` - a simple 32x32 SVG. For proper app distribution:

1. **Create a high-resolution source** (1024x1024 PNG or SVG)
2. **Generate all required sizes** using the scripts above
3. **Store in `assets/icons/`** directory:
   ```
   assets/
   ├── logo.svg              # Tray icon source
   └── icons/
       ├── icon.iconset/     # macOS iconset folder
       ├── icon.icns         # Generated macOS icon
       ├── icon.ico          # Windows icon
       └── icon-*.png        # Linux PNG set
   ```

---

## Code Signing & Notarization

### macOS Code Signing

#### Prerequisites

1. **Apple Developer Account** ($99/year for distribution)
2. **Developer ID Application** certificate (for distribution outside App Store)
3. **Provisioning profile** (optional, for specific entitlements)

#### Environment Variables

```bash
# For local signing
export APPLE_SIGNING_IDENTITY="Developer ID Application: Your Name (TEAM_ID)"

# For CI/CD
export APPLE_CERTIFICATE="<base64-encoded .p12>"
export APPLE_CERTIFICATE_PASSWORD="<password>"
export APPLE_ID="your@email.com"
export APPLE_PASSWORD="<app-specific-password>"
export APPLE_TEAM_ID="TEAM_ID"
```

#### Signing Process

```bash
# Find your signing identity
security find-identity -v -p codesigning

# Sign the app bundle
codesign --deep --force --timestamp --options runtime \
  --sign "Developer ID Application: Your Name (TEAM_ID)" \
  --entitlements entitlements.plist \
  "ScriptKit.app"

# Verify signature
codesign -vvv --deep --strict "ScriptKit.app"
spctl -a -vvv "ScriptKit.app"
```

#### Entitlements (entitlements.plist)

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
    <true/>
    <key>com.apple.security.cs.allow-jit</key>
    <true/>
    <key>com.apple.security.cs.disable-library-validation</key>
    <true/>
    <key>com.apple.security.automation.apple-events</key>
    <true/>
</dict>
</plist>
```

### Notarization

Apple requires notarization for apps distributed outside the App Store:

```bash
# Submit for notarization
xcrun notarytool submit "ScriptKit.dmg" \
  --apple-id "$APPLE_ID" \
  --password "$APPLE_PASSWORD" \
  --team-id "$APPLE_TEAM_ID" \
  --wait

# Staple the notarization ticket
xcrun stapler staple "ScriptKit.dmg"
```

### Ad-Hoc Signing (Development Only)

For development/testing without Apple certificates:

```bash
codesign --force --deep --sign - "ScriptKit.app"
```

**Note:** Ad-hoc signed apps will still show Gatekeeper warnings.

---

## Platform-Specific Considerations

### macOS

| Consideration | Details |
|--------------|---------|
| **Minimum OS Version** | Set to 10.15 (Catalina) for modern API support |
| **Universal Binary** | Consider building for both `aarch64-apple-darwin` and `x86_64-apple-darwin` |
| **Hardened Runtime** | Required for notarization |
| **App Sandbox** | Optional, but recommended for App Store |
| **Icon Extraction (NSWorkspace)** | Works on all macOS machines with proper entitlements |

### Windows (Future)

| Consideration | Details |
|--------------|---------|
| **Installer Format** | MSI (enterprise) or NSIS (general) |
| **Code Signing** | Authenticode certificate required |
| **UAC Manifest** | Include for proper elevation handling |
| **Visual C++ Runtime** | May need to bundle vcruntime |

### Linux (Future)

| Consideration | Details |
|--------------|---------|
| **Package Formats** | .deb (Debian/Ubuntu), .rpm (Fedora/RHEL), AppImage (universal) |
| **Dependencies** | List shared library dependencies |
| **Desktop Entry** | Include .desktop file for launcher integration |
| **Icon Locations** | `/usr/share/icons/hicolor/{size}/apps/` |

---

## Quick Start Guide

### Step 1: Prepare Icons

```bash
# From project root
mkdir -p assets/icons

# Convert SVG to high-res PNG (requires inkscape or similar)
inkscape assets/logo.svg --export-width=1024 --export-filename=assets/icons/icon-1024.png

# Generate iconset
mkdir assets/icons/icon.iconset
cd assets/icons
sips -z 16 16     icon-1024.png --out icon.iconset/icon_16x16.png
sips -z 32 32     icon-1024.png --out icon.iconset/icon_16x16@2x.png
sips -z 32 32     icon-1024.png --out icon.iconset/icon_32x32.png
sips -z 64 64     icon-1024.png --out icon.iconset/icon_32x32@2x.png
sips -z 128 128   icon-1024.png --out icon.iconset/icon_128x128.png
sips -z 256 256   icon-1024.png --out icon.iconset/icon_128x128@2x.png
sips -z 256 256   icon-1024.png --out icon.iconset/icon_256x256.png
sips -z 512 512   icon-1024.png --out icon.iconset/icon_256x256@2x.png
sips -z 512 512   icon-1024.png --out icon.iconset/icon_512x512.png
cp icon-1024.png  icon.iconset/icon_512x512@2x.png
iconutil -c icns icon.iconset
```

### Step 2: Update Cargo.toml

Add the bundle metadata (see [Cargo.toml Configuration](#cargotoml-configuration) above).

### Step 3: Install cargo-bundle

```bash
cargo install cargo-bundle --git https://github.com/zed-industries/cargo-bundle.git --branch zed-deploy
```

### Step 4: Build and Bundle

```bash
# Development build (unsigned)
cargo bundle

# Release build
cargo bundle --release

# For specific target
cargo bundle --release --target aarch64-apple-darwin
```

### Step 5: Create DMG (Optional)

```bash
# Create DMG from .app
hdiutil create -volname "Script Kit" \
  -srcfolder "target/release/bundle/osx/Script Kit.app" \
  -ov -format UDZO \
  "target/release/ScriptKit.dmg"
```

### Step 6: Sign and Notarize (Production)

```bash
# Sign
codesign --deep --force --timestamp --options runtime \
  --sign "$APPLE_SIGNING_IDENTITY" \
  --entitlements entitlements.plist \
  "target/release/bundle/osx/Script Kit.app"

# Create signed DMG
hdiutil create ... # as above

# Sign DMG
codesign --force --sign "$APPLE_SIGNING_IDENTITY" "ScriptKit.dmg"

# Notarize
xcrun notarytool submit "ScriptKit.dmg" --apple-id "$APPLE_ID" \
  --password "$APPLE_PASSWORD" --team-id "$APPLE_TEAM_ID" --wait

# Staple
xcrun stapler staple "ScriptKit.dmg"
```

---

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Build and Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build-macos:
    runs-on: macos-latest
    strategy:
      matrix:
        target:
          - aarch64-apple-darwin
          - x86_64-apple-darwin
    
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
        with:
          targets: ${{ matrix.target }}
      
      - name: Install cargo-bundle
        run: cargo install cargo-bundle --git https://github.com/zed-industries/cargo-bundle.git --branch zed-deploy
      
      - name: Build
        run: cargo build --release --target ${{ matrix.target }}
      
      - name: Bundle
        run: cargo bundle --release --target ${{ matrix.target }}
      
      - name: Import Certificate
        if: github.event_name != 'pull_request'
        env:
          APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
          APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
        run: |
          echo $APPLE_CERTIFICATE | base64 --decode > certificate.p12
          security create-keychain -p "" build.keychain
          security default-keychain -s build.keychain
          security unlock-keychain -p "" build.keychain
          security import certificate.p12 -k build.keychain -P "$APPLE_CERTIFICATE_PASSWORD" -T /usr/bin/codesign
          security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "" build.keychain
      
      - name: Sign App
        if: github.event_name != 'pull_request'
        run: |
          codesign --deep --force --timestamp --options runtime \
            --sign "${{ secrets.APPLE_SIGNING_IDENTITY }}" \
            --entitlements entitlements.plist \
            "target/${{ matrix.target }}/release/bundle/osx/Script Kit.app"
      
      - name: Create DMG
        run: |
          hdiutil create -volname "Script Kit" \
            -srcfolder "target/${{ matrix.target }}/release/bundle/osx/Script Kit.app" \
            -ov -format UDZO \
            "ScriptKit-${{ matrix.target }}.dmg"
      
      - name: Notarize
        if: github.event_name != 'pull_request'
        env:
          APPLE_ID: ${{ secrets.APPLE_ID }}
          APPLE_PASSWORD: ${{ secrets.APPLE_PASSWORD }}
          APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
        run: |
          xcrun notarytool submit "ScriptKit-${{ matrix.target }}.dmg" \
            --apple-id "$APPLE_ID" --password "$APPLE_PASSWORD" \
            --team-id "$APPLE_TEAM_ID" --wait
          xcrun stapler staple "ScriptKit-${{ matrix.target }}.dmg"
      
      - name: Upload Artifact
        uses: actions/upload-artifact@v4
        with:
          name: ScriptKit-${{ matrix.target }}
          path: ScriptKit-${{ matrix.target }}.dmg
```

---

## Current Icon Extraction Approach

The app currently uses `NSWorkspace` for icon extraction via Cocoa APIs:

```rust
// From src/app_launcher.rs (conceptual)
let workspace: id = msg_send![class!(NSWorkspace), sharedWorkspace];
let icon: id = msg_send![workspace, iconForFile: path];
```

### Will This Work on All macOS Machines?

**Yes**, with considerations:

1. **NSWorkspace is a public API** - Available since macOS 10.0
2. **No special entitlements required** for icon extraction
3. **File access permissions** may be needed for some paths
4. **Sandbox considerations** - If sandboxed, may need file access entitlements

### Recommendations

1. **Cache icons** to disk on first extraction (already done)
2. **Fallback icon** for permission errors
3. **Async loading** to prevent UI blocking
4. **Test with hardened runtime** to ensure it works when signed

---

## Summary

| Task | Tool | Notes |
|------|------|-------|
| Build binary | `cargo build --release` | Standard Rust build |
| Create .app bundle | `cargo-bundle` (Zed fork) | Integrates with Cargo.toml |
| Generate icons | `iconutil` / `sips` | Create from 1024x1024 source |
| Code sign | `codesign` | Requires Apple Developer cert |
| Create DMG | `hdiutil` | Optional but professional |
| Notarize | `notarytool` | Required for distribution |
| CI/CD | GitHub Actions | See example workflow |

---

## References

- [Zed's bundle-mac script](https://github.com/zed-industries/zed/blob/main/script/bundle-mac)
- [cargo-bundle (Zed fork)](https://github.com/zed-industries/cargo-bundle)
- [Apple Code Signing Guide](https://developer.apple.com/documentation/security/code_signing_services)
- [Notarization Documentation](https://developer.apple.com/documentation/security/notarizing_macos_software_before_distribution)
- [Tauri Bundler Documentation](https://v2.tauri.app/distribute/)
