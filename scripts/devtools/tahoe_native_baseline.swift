// tahoe_native_baseline.swift
//
// Measures REAL macOS 26 (Tahoe) AppKit control geometry so the Liquid Glass
// proof can compare our GPUI tokens against `measuredNative` baselines instead
// of guessed constants. Apple documents control metrics as system-owned ("don't
// hardcode sizes"), so the authoritative numbers come from instantiating the
// native controls and reading their cell geometry.
//
// Emits one JSON object on stdout: per-control-size search/text field heights
// and internal text insets (bezel -> text), plus NSGlassEffectView's default
// corner radius when the class resolves. No window/screenshot needed — pure
// AppKit cell geometry, so it runs headless under an accessory app.
//
// Usage:  tahoe_native_baseline   (no args)

import AppKit

let app = NSApplication.shared
app.setActivationPolicy(.accessory)

func rectJSON(_ r: NSRect) -> [String: Double] {
    ["x": Double(r.minX), "y": Double(r.minY), "w": Double(r.width), "h": Double(r.height)]
}

let sizes: [(NSControl.ControlSize, String)] = [
    (.mini, "mini"),
    (.small, "small"),
    (.regular, "regular"),
    (.large, "large"),
]

// --- NSSearchField geometry per control size ------------------------------
var searchFields: [[String: Any]] = []
for (size, label) in sizes {
    let field = NSSearchField(frame: NSRect(x: 0, y: 0, width: 320, height: 40))
    field.controlSize = size
    field.stringValue = "Search"
    let intrinsic = field.intrinsicContentSize
    let height = intrinsic.height > 0 ? intrinsic.height : field.frame.height
    field.frame = NSRect(x: 0, y: 0, width: 320, height: height)
    field.layoutSubtreeIfNeeded()
    let bounds = field.bounds
    var entry: [String: Any] = [
        "controlSize": label,
        "intrinsicHeightPt": Double(height),
        "bounds": rectJSON(bounds),
    ]
    if let cell = field.cell as? NSSearchFieldCell {
        let textRect = cell.searchTextRect(forBounds: bounds)
        let iconRect = cell.searchButtonRect(forBounds: bounds)
        let drawing = cell.drawingRect(forBounds: bounds)
        entry["searchTextRect"] = rectJSON(textRect)
        entry["searchButtonRect"] = rectJSON(iconRect)
        entry["drawingRect"] = rectJSON(drawing)
        // Left padding before the magnifier icon = the bezel inset to first content.
        entry["leadingBezelInsetPt"] = Double(iconRect.minX - bounds.minX)
        // Text vertical inset within the field.
        entry["textVerticalInsetPt"] = Double((bounds.height - textRect.height) / 2)
        entry["textTrailingInsetPt"] = Double(bounds.maxX - textRect.maxX)
    }
    searchFields.append(entry)
}

// --- NSTextField (plain) bezel->content inset per control size ------------
var textFields: [[String: Any]] = []
for (size, label) in sizes {
    let field = NSTextField(frame: NSRect(x: 0, y: 0, width: 320, height: 40))
    field.controlSize = size
    field.isBezeled = true
    field.bezelStyle = .roundedBezel
    field.stringValue = "Text"
    let intrinsic = field.intrinsicContentSize
    let height = intrinsic.height > 0 ? intrinsic.height : field.frame.height
    field.frame = NSRect(x: 0, y: 0, width: 320, height: height)
    field.layoutSubtreeIfNeeded()
    let bounds = field.bounds
    var entry: [String: Any] = [
        "controlSize": label,
        "intrinsicHeightPt": Double(height),
        "bounds": rectJSON(bounds),
    ]
    if let cell = field.cell {
        let drawing = cell.drawingRect(forBounds: bounds)
        let title = cell.titleRect(forBounds: bounds)
        entry["drawingRect"] = rectJSON(drawing)
        entry["titleRect"] = rectJSON(title)
        // Horizontal bezel->content inset = the input's internal left padding.
        entry["contentHorizontalInsetPt"] = Double(drawing.minX - bounds.minX)
        entry["contentVerticalInsetPt"] = Double((bounds.height - drawing.height) / 2)
    }
    textFields.append(entry)
}

// --- NSGlassEffectView default corner radius (Tahoe) ----------------------
var glass: [String: Any] = ["available": false]
if let glassClass = NSClassFromString("NSGlassEffectView") as? NSView.Type {
    let view = glassClass.init(frame: NSRect(x: 0, y: 0, width: 200, height: 120))
    glass["available"] = true
    if view.responds(to: NSSelectorFromString("cornerRadius")) {
        let radius = view.value(forKey: "cornerRadius") as? Double
        glass["defaultCornerRadiusPt"] = radius ?? NSNull()
    } else {
        glass["defaultCornerRadiusPt"] = NSNull()
        glass["note"] = "NSGlassEffectView has no readable cornerRadius default"
    }
}

let out: [String: Any] = [
    "ok": true,
    "platform": "macOS",
    "osVersion": ProcessInfo.processInfo.operatingSystemVersionString,
    "source": "native-appkit-baseline",
    "searchFields": searchFields,
    "textFields": textFields,
    "glassEffectView": glass,
]
let data = try! JSONSerialization.data(withJSONObject: out, options: [.sortedKeys, .prettyPrinted])
FileHandle.standardOutput.write(data)
FileHandle.standardOutput.write("\n".data(using: .utf8)!)
