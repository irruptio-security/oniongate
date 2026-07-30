#!/usr/bin/env swift

import CoreGraphics
import Foundation
import ImageIO
import UniformTypeIdentifiers

enum IconError: Error, CustomStringConvertible {
    case cannotLoad(String)
    case cannotCreateContext
    case cannotEncode(String)

    var description: String {
        switch self {
        case .cannotLoad(let path):
            return "Could not load image at \(path)"
        case .cannotCreateContext:
            return "Could not create an RGBA drawing context"
        case .cannotEncode(let path):
            return "Could not write PNG at \(path)"
        }
    }
}

let fileManager = FileManager.default
let root = URL(fileURLWithPath: fileManager.currentDirectoryPath, isDirectory: true)
let logoURL = root.appendingPathComponent("public/logo.png")
let docsLogoURL = root.appendingPathComponent("docs/public/logo.png")
let macTrayURL = root.appendingPathComponent("src-tauri/icons/tray-macos.png")
let colorTrayURL = root.appendingPathComponent("src-tauri/icons/tray-color.png")

func loadImage(_ url: URL) throws -> CGImage {
    guard
        let source = CGImageSourceCreateWithURL(url as CFURL, nil),
        let image = CGImageSourceCreateImageAtIndex(source, 0, nil)
    else {
        throw IconError.cannotLoad(url.path)
    }
    return image
}

func writePNG(_ image: CGImage, to url: URL) throws {
    guard let destination = CGImageDestinationCreateWithURL(
        url as CFURL,
        UTType.png.identifier as CFString,
        1,
        nil
    ) else {
        throw IconError.cannotEncode(url.path)
    }
    CGImageDestinationAddImage(destination, image, nil)
    guard CGImageDestinationFinalize(destination) else {
        throw IconError.cannotEncode(url.path)
    }
}

func rgbaContext(
    width: Int,
    height: Int,
    data: UnsafeMutableRawPointer? = nil
) throws -> CGContext {
    guard let context = CGContext(
        data: data,
        width: width,
        height: height,
        bitsPerComponent: 8,
        bytesPerRow: width * 4,
        space: CGColorSpaceCreateDeviceRGB(),
        bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue
            | CGBitmapInfo.byteOrder32Big.rawValue
    ) else {
        throw IconError.cannotCreateContext
    }
    context.setAllowsAntialiasing(true)
    context.setShouldAntialias(true)
    return context
}

/// Remove only neutral dark pixels connected to an image edge. This clears the
/// old opaque black corner matte without erasing dark purple details inside the
/// logo. Transparent edge pixels remain traversable, so the operation is
/// idempotent.
func clearConnectedCornerMatte(_ source: CGImage) throws -> CGImage {
    let width = source.width
    let height = source.height
    var pixels = [UInt8](repeating: 0, count: width * height * 4)

    try pixels.withUnsafeMutableBytes { bytes in
        let context = try rgbaContext(
            width: width,
            height: height,
            data: bytes.baseAddress
        )
        context.draw(
            source,
            in: CGRect(x: 0, y: 0, width: width, height: height)
        )
    }

    var visited = [Bool](repeating: false, count: width * height)
    var queue = [Int]()
    queue.reserveCapacity(width * 4 + height * 4)

    func enqueue(_ x: Int, _ y: Int) {
        guard x >= 0, x < width, y >= 0, y < height else { return }
        let index = y * width + x
        guard !visited[index] else { return }
        visited[index] = true
        queue.append(index)
    }

    for x in 0..<width {
        enqueue(x, 0)
        enqueue(x, height - 1)
    }
    for y in 0..<height {
        enqueue(0, y)
        enqueue(width - 1, y)
    }

    var cursor = 0
    while cursor < queue.count {
        let pixelIndex = queue[cursor]
        cursor += 1
        let byteIndex = pixelIndex * 4
        let red = Int(pixels[byteIndex])
        let green = Int(pixels[byteIndex + 1])
        let blue = Int(pixels[byteIndex + 2])
        let alpha = Int(pixels[byteIndex + 3])
        let maximum = max(red, green, blue)
        let minimum = min(red, green, blue)
        let isTransparent = alpha == 0
        let isNeutralDarkMatte = maximum <= 70 && maximum - minimum <= 20

        guard isTransparent || isNeutralDarkMatte else { continue }

        if !isTransparent {
            pixels[byteIndex] = 0
            pixels[byteIndex + 1] = 0
            pixels[byteIndex + 2] = 0
            pixels[byteIndex + 3] = 0
        }

        let x = pixelIndex % width
        let y = pixelIndex / width
        enqueue(x - 1, y)
        enqueue(x + 1, y)
        enqueue(x, y - 1)
        enqueue(x, y + 1)
    }

    return try pixels.withUnsafeMutableBytes { bytes in
        let context = try rgbaContext(
            width: width,
            height: height,
            data: bytes.baseAddress
        )
        guard let image = context.makeImage() else {
            throw IconError.cannotCreateContext
        }
        return image
    }
}

func onionBulbPath(size: CGFloat) -> CGPath {
    let scale = size / 64
    let path = CGMutablePath()
    path.move(to: CGPoint(x: 32 * scale, y: 58 * scale))
    path.addCurve(
        to: CGPoint(x: 11 * scale, y: 36 * scale),
        control1: CGPoint(x: 20 * scale, y: 56 * scale),
        control2: CGPoint(x: 11 * scale, y: 48 * scale)
    )
    path.addCurve(
        to: CGPoint(x: 27 * scale, y: 15 * scale),
        control1: CGPoint(x: 11 * scale, y: 27 * scale),
        control2: CGPoint(x: 21 * scale, y: 22 * scale)
    )
    path.addCurve(
        to: CGPoint(x: 32 * scale, y: 8 * scale),
        control1: CGPoint(x: 29 * scale, y: 12 * scale),
        control2: CGPoint(x: 31 * scale, y: 10 * scale)
    )
    path.addCurve(
        to: CGPoint(x: 37 * scale, y: 15 * scale),
        control1: CGPoint(x: 33 * scale, y: 10 * scale),
        control2: CGPoint(x: 35 * scale, y: 12 * scale)
    )
    path.addCurve(
        to: CGPoint(x: 53 * scale, y: 36 * scale),
        control1: CGPoint(x: 43 * scale, y: 22 * scale),
        control2: CGPoint(x: 53 * scale, y: 27 * scale)
    )
    path.addCurve(
        to: CGPoint(x: 32 * scale, y: 58 * scale),
        control1: CGPoint(x: 53 * scale, y: 48 * scale),
        control2: CGPoint(x: 44 * scale, y: 56 * scale)
    )
    path.closeSubpath()
    return path
}

func leafPaths(size: CGFloat) -> [CGPath] {
    let scale = size / 64
    let center = CGMutablePath()
    center.move(to: CGPoint(x: 32 * scale, y: 18 * scale))
    center.addCurve(
        to: CGPoint(x: 32 * scale, y: 2 * scale),
        control1: CGPoint(x: 26 * scale, y: 12 * scale),
        control2: CGPoint(x: 28 * scale, y: 6 * scale)
    )
    center.addCurve(
        to: CGPoint(x: 32 * scale, y: 18 * scale),
        control1: CGPoint(x: 36 * scale, y: 6 * scale),
        control2: CGPoint(x: 38 * scale, y: 12 * scale)
    )
    center.closeSubpath()

    let left = CGMutablePath()
    left.move(to: CGPoint(x: 30 * scale, y: 19 * scale))
    left.addCurve(
        to: CGPoint(x: 15 * scale, y: 7 * scale),
        control1: CGPoint(x: 23 * scale, y: 18 * scale),
        control2: CGPoint(x: 18 * scale, y: 12 * scale)
    )
    left.addCurve(
        to: CGPoint(x: 30 * scale, y: 19 * scale),
        control1: CGPoint(x: 23 * scale, y: 8 * scale),
        control2: CGPoint(x: 29 * scale, y: 12 * scale)
    )
    left.closeSubpath()

    let right = CGMutablePath()
    right.move(to: CGPoint(x: 34 * scale, y: 19 * scale))
    right.addCurve(
        to: CGPoint(x: 49 * scale, y: 7 * scale),
        control1: CGPoint(x: 41 * scale, y: 18 * scale),
        control2: CGPoint(x: 46 * scale, y: 12 * scale)
    )
    right.addCurve(
        to: CGPoint(x: 34 * scale, y: 19 * scale),
        control1: CGPoint(x: 41 * scale, y: 8 * scale),
        control2: CGPoint(x: 35 * scale, y: 12 * scale)
    )
    right.closeSubpath()

    return [center, left, right]
}

func makeTrayIcon(size: Int, template: Bool) throws -> CGImage {
    let context = try rgbaContext(width: size, height: size)
    context.clear(CGRect(x: 0, y: 0, width: size, height: size))
    context.translateBy(x: 0, y: CGFloat(size))
    context.scaleBy(x: 1, y: -1)

    let bulb = onionBulbPath(size: CGFloat(size))
    let leaves = leafPaths(size: CGFloat(size))
    let black = CGColor(gray: 0, alpha: 1)
    let purple = CGColor(
        red: 102 / 255,
        green: 45 / 255,
        blue: 145 / 255,
        alpha: 1
    )
    let green = CGColor(
        red: 35 / 255,
        green: 181 / 255,
        blue: 101 / 255,
        alpha: 1
    )

    context.setFillColor(template ? black : purple)
    context.addPath(bulb)
    context.fillPath()
    context.setFillColor(template ? black : green)
    for leaf in leaves {
        context.addPath(leaf)
        context.fillPath()
    }

    if !template {
        let scale = CGFloat(size) / 64
        context.setStrokeColor(CGColor(gray: 1, alpha: 0.85))
        context.setLineWidth(max(1, 1.5 * scale))
        context.setLineCap(.round)
        for offset in [-8.0, 0.0, 8.0] {
            let line = CGMutablePath()
            line.move(to: CGPoint(x: (32 + offset * 0.2) * scale, y: 52 * scale))
            line.addCurve(
                to: CGPoint(x: (32 + offset) * scale, y: 20 * scale),
                control1: CGPoint(x: (24 + offset) * scale, y: 42 * scale),
                control2: CGPoint(x: (25 + offset) * scale, y: 28 * scale)
            )
            context.addPath(line)
            context.strokePath()
        }
    }

    guard let image = context.makeImage() else {
        throw IconError.cannotCreateContext
    }
    return image
}

do {
    let source = try loadImage(logoURL)
    let cleaned = try clearConnectedCornerMatte(source)
    try writePNG(cleaned, to: logoURL)
    try writePNG(cleaned, to: docsLogoURL)
    try writePNG(makeTrayIcon(size: 32, template: true), to: macTrayURL)
    try writePNG(makeTrayIcon(size: 32, template: false), to: colorTrayURL)
    print("Updated \(logoURL.path)")
    print("Updated \(docsLogoURL.path)")
    print("Wrote \(macTrayURL.path)")
    print("Wrote \(colorTrayURL.path)")
} catch {
    fputs("Icon generation failed: \(error)\n", stderr)
    exit(1)
}
