// tahoe_window_mask_probe.swift
//
// Measures the REAL rendered corner radius of native macOS 26 (Tahoe) windows
// so concern #1 ("our window corners aren't rounded enough") can be compared to
// an authoritative native baseline. Apple publishes NO window/panel corner-radius
// constant — the value is system/style-owned — so the only honest reference is to
// instantiate native windows and MEASURE their corner mask.
//
// Method (no private API): create a native NSWindow per style, show it opaque
// over the screen, capture it by window id with `screencapture -l<id> -o` (which
// yields a PNG whose rounded corners are transparent in the alpha channel), then
// scan the alpha mask at the top-left corner. For a rounded rect the first opaque
// pixel down column x=0 sits at y=radius (in backing pixels); divide by the
// backing scale for points.
//
// Emits one JSON object on stdout: per-style measured corner radius in pt.
//
// Usage:  tahoe_window_mask_probe

import AppKit
import Foundation

let app = NSApplication.shared
app.setActivationPolicy(.accessory)

func measureCornerRadiusPx(_ cgImage: CGImage) -> Int? {
    let width = cgImage.width
    let height = cgImage.height
    guard width > 4, height > 4 else { return nil }
    guard let data = cgImage.dataProvider?.data,
          let ptr = CFDataGetBytePtr(data) else { return nil }
    let bpr = cgImage.bytesPerRow
    let bpp = cgImage.bitsPerPixel / 8
    guard bpp >= 4 else { return nil }
    // Alpha byte index within a pixel. Most screencapture PNGs are RGBA or BGRA
    // with alpha last; the alpha channel is what carries the rounded mask.
    let alphaOffset = bpp - 1
    func alphaAt(_ x: Int, _ y: Int) -> UInt8 {
        ptr[y * bpr + x * bpp + alphaOffset]
    }
    // Scan column x=0 downward: first mostly-opaque pixel => corner radius (px).
    let threshold: UInt8 = 200
    for y in 0..<min(height, 200) {
        if alphaAt(0, y) >= threshold { return y }
    }
    // Fallback: scan row y=0 rightward.
    for x in 0..<min(width, 200) {
        if alphaAt(x, 0) >= threshold { return x }
    }
    return nil
}

func captureWindowCGImage(windowNumber: Int) -> CGImage? {
    let tmp = "/tmp/tahoe_mask_\(windowNumber).png"
    let proc = Process()
    proc.executableURL = URL(fileURLWithPath: "/usr/sbin/screencapture")
    // -l<id> capture by window id, -o omit shadow, -x no sound.
    proc.arguments = ["-l\(windowNumber)", "-o", "-x", tmp]
    do { try proc.run(); proc.waitUntilExit() } catch { return nil }
    guard proc.terminationStatus == 0,
          let img = NSImage(contentsOfFile: tmp),
          let cg = img.cgImage(forProposedRect: nil, context: nil, hints: nil)
    else { return nil }
    return cg
}

struct StyleSpec {
    let label: String
    let styleMask: NSWindow.StyleMask
    let useGlass: Bool
}

let specs: [StyleSpec] = [
    StyleSpec(label: "titledStandardWindow", styleMask: [.titled, .closable, .miniaturizable, .resizable], useGlass: false),
    StyleSpec(label: "titledFullSizeContent", styleMask: [.titled, .closable, .fullSizeContentView], useGlass: false),
    StyleSpec(label: "borderlessPanel", styleMask: [.borderless], useGlass: false),
    StyleSpec(label: "glassEffectWindow", styleMask: [.titled, .closable, .fullSizeContentView], useGlass: true),
]

var results: [[String: Any]] = []
let screen = NSScreen.main
let scale = Double(screen?.backingScaleFactor ?? 2.0)

for (i, spec) in specs.enumerated() {
    let rect = NSRect(x: 200 + i * 40, y: 200 + i * 40, width: 420, height: 300)
    let win = NSWindow(contentRect: rect, styleMask: spec.styleMask, backing: .buffered, defer: false)
    win.isOpaque = true
    win.backgroundColor = .white
    win.hasShadow = false
    let content = NSView(frame: NSRect(origin: .zero, size: rect.size))
    content.wantsLayer = true
    content.layer?.backgroundColor = NSColor.white.cgColor
    if spec.useGlass, let glassClass = NSClassFromString("NSGlassEffectView") as? NSView.Type {
        let glass = glassClass.init(frame: content.bounds)
        glass.autoresizingMask = [.width, .height]
        content.addSubview(glass)
    }
    win.contentView = content
    win.makeKeyAndOrderFront(nil)
    win.orderFrontRegardless()
    // Let the window server composite the rounded mask.
    RunLoop.main.run(until: Date().addingTimeInterval(0.45))

    var entry: [String: Any] = ["style": spec.label, "styleMaskRaw": spec.styleMask.rawValue]
    let radiusPx = captureWindowCGImage(windowNumber: win.windowNumber).flatMap(measureCornerRadiusPx)
    if let r = radiusPx {
        entry["cornerRadiusPx"] = r
        entry["cornerRadiusPt"] = Double(r) / scale
    } else {
        entry["cornerRadiusPx"] = NSNull()
        entry["note"] = "could not measure (capture failed or fully opaque corner)"
    }
    results.append(entry)
    win.orderOut(nil)
}

let out: [String: Any] = [
    "ok": true,
    "platform": "macOS",
    "osVersion": ProcessInfo.processInfo.operatingSystemVersionString,
    "source": "native-window-mask-screenshot",
    "backingScaleFactor": scale,
    "ourWindowRadiusTokenPt": 22.0,
    "styles": results,
]
let data = try! JSONSerialization.data(withJSONObject: out, options: [.sortedKeys, .prettyPrinted])
FileHandle.standardOutput.write(data)
FileHandle.standardOutput.write("\n".data(using: .utf8)!)
