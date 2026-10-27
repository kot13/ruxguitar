import AppKit

let emoji = "🎸"
let sizes: [(name: String, px: Int)] = [
    ("icon_16x16.png", 16),
    ("icon_16x16@2x.png", 32),
    ("icon_32x32.png", 32),
    ("icon_32x32@2x.png", 64),
    ("icon_128x128.png", 128),
    ("icon_128x128@2x.png", 256),
    ("icon_256x256.png", 256),
    ("icon_256x256@2x.png", 512),
    ("icon_512x512.png", 512),
    ("icon_512x512@2x.png", 1024),
]

guard CommandLine.arguments.count == 2 else {
    fputs("usage: render-macos-icon.swift <iconset-dir>\n", stderr)
    exit(1)
}

let iconset = URL(fileURLWithPath: CommandLine.arguments[1], isDirectory: true)
try FileManager.default.createDirectory(at: iconset, withIntermediateDirectories: true)

func drawIcon(pixels: Int) -> NSImage {
    let image = NSImage(size: NSSize(width: pixels, height: pixels))
    image.lockFocus()

    let rect = NSRect(x: 0, y: 0, width: pixels, height: pixels)
    let path = NSBezierPath(roundedRect: rect.insetBy(dx: CGFloat(pixels) * 0.06, dy: CGFloat(pixels) * 0.06),
                            xRadius: CGFloat(pixels) * 0.22,
                            yRadius: CGFloat(pixels) * 0.22)
    NSColor.white.setFill()
    path.fill()

    let fontSize = CGFloat(pixels) * 0.52
    let font = NSFont(name: "Apple Color Emoji", size: fontSize)
        ?? NSFont.systemFont(ofSize: fontSize)
    let attrs: [NSAttributedString.Key: Any] = [.font: font]
    let str = NSAttributedString(string: emoji, attributes: attrs)
    let strSize = str.size()
    let textRect = NSRect(
        x: (CGFloat(pixels) - strSize.width) / 2,
        y: (CGFloat(pixels) - strSize.height) / 2 - CGFloat(pixels) * 0.02,
        width: strSize.width,
        height: strSize.height
    )
    str.draw(in: textRect)

    image.unlockFocus()
    return image
}

for entry in sizes {
    let image = drawIcon(pixels: entry.px)
    guard let tiff = image.tiffRepresentation,
          let rep = NSBitmapImageRep(data: tiff),
          let png = rep.representation(using: .png, properties: [:]) else {
        fputs("failed to encode \(entry.name)\n", stderr)
        exit(1)
    }
    let out = iconset.appendingPathComponent(entry.name)
    try png.write(to: out)
}
