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

// Shared layout manager for default line-height measurement.
let sharedLayoutManager = NSLayoutManager()

// Capture the measurable typographic metrics of a resolved NSFont. The weight
// trait is AppKit's normalized weight (regular ~= 0.0, medium ~= 0.23,
// semibold ~= 0.3, bold ~= 0.4); we emit it alongside the point size and the
// default line height so the guideline engine can classify rendered text
// against the real system font instead of a guessed constant.
func fontTraits(_ font: NSFont) -> [String: Any] {
    let traits = font.fontDescriptor.object(forKey: .traits) as? [NSFontDescriptor.TraitKey: Any]
    let weight = traits?[.weight] as? NSNumber
    let width = traits?[.width] as? NSNumber
    return [
        "fontName": font.fontName,
        "familyName": font.familyName ?? "",
        "pointSizePt": Double(font.pointSize),
        "weightTrait": weight?.doubleValue as Any? ?? NSNull(),
        "widthTrait": width?.doubleValue as Any? ?? NSNull(),
        "ascenderPt": Double(font.ascender),
        "descenderPt": Double(font.descender),
        "leadingPt": Double(font.leading),
        "capHeightPt": Double(font.capHeight),
        "xHeightPt": Double(font.xHeight),
        "defaultLineHeightPt": Double(sharedLayoutManager.defaultLineHeight(for: font)),
    ]
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
    if let font = field.font {
        entry["font"] = fontTraits(font)
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
    if let font = field.font {
        entry["font"] = fontTraits(font)
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

// --- System font metrics (Tahoe) ------------------------------------------
// The standard system font sizes and the resolved fonts for the regular
// control content + body text style. These are the measuredNative anchors for
// the typography guideline metrics: ordinary macOS control/body text is 13pt
// Regular with a 16pt default line height; the launcher's hero search input
// runs larger by design, which the engine reports as a soft divergence.
let regularControlSize = NSFont.systemFontSize(for: .regular)
let fontMetrics: [String: Any] = [
    "systemFontSizePt": Double(NSFont.systemFontSize),
    "smallSystemFontSizePt": Double(NSFont.smallSystemFontSize),
    "labelFontSizePt": Double(NSFont.labelFontSize),
    "controlSystemFontSizeByControlSizePt": [
        "mini": Double(NSFont.systemFontSize(for: .mini)),
        "small": Double(NSFont.systemFontSize(for: .small)),
        "regular": Double(regularControlSize),
        "large": Double(NSFont.systemFontSize(for: .large)),
    ],
    "systemRegular": fontTraits(NSFont.systemFont(ofSize: NSFont.systemFontSize, weight: .regular)),
    "systemBody": fontTraits(NSFont.preferredFont(forTextStyle: .body, options: [:])),
    "systemBodyEmphasizedApprox": fontTraits(NSFont.systemFont(ofSize: NSFont.systemFontSize, weight: .semibold)),
    "smallSystemRegular": fontTraits(NSFont.systemFont(ofSize: NSFont.smallSystemFontSize, weight: .regular)),
    "labelFont": fontTraits(NSFont.systemFont(ofSize: NSFont.labelFontSize, weight: .regular)),
    "regularControlContentFont": fontTraits(NSFont.controlContentFont(ofSize: regularControlSize)),
    "menuFontDefault": fontTraits(NSFont.menuFont(ofSize: 0)),
]

let out: [String: Any] = [
    "ok": true,
    "platform": "macOS",
    "osVersion": ProcessInfo.processInfo.operatingSystemVersionString,
    "source": "native-appkit-baseline",
    "searchFields": searchFields,
    "textFields": textFields,
    "fontMetrics": fontMetrics,
    "glassEffectView": glass,
]
let data = try! JSONSerialization.data(withJSONObject: out, options: [.sortedKeys, .prettyPrinted])
FileHandle.standardOutput.write(data)
FileHandle.standardOutput.write("\n".data(using: .utf8)!)
