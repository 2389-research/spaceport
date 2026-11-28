// ABOUTME: Prepares a pasted screenshot and thumbnail off the main thread.
// ABOUTME: Bounds input size and dimensions and strips image metadata before transfer.
import Foundation
import ImageIO
import UniformTypeIdentifiers

struct DraftImage: Identifiable {
    enum Failure: LocalizedError {
        case invalid, tooLarge
        var errorDescription: String? {
            switch self {
            case .invalid: return "Couldn’t read this image. Try copying the screenshot again."
            case .tooLarge: return "This image is too large. Crop it and try again."
            }
        }
    }

    private static let maxBytes = 16 * 1024 * 1024
    let id = UUID()
    let png: Data
    let thumbnail: CGImage

    init(data: Data) throws {
        guard data.count <= Self.maxBytes else { throw Failure.tooLarge }
        guard let source = CGImageSourceCreateWithData(data as CFData, nil),
              let properties = CGImageSourceCopyPropertiesAtIndex(source, 0, nil) as? [CFString: Any],
              let width = properties[kCGImagePropertyPixelWidth] as? Int,
              let height = properties[kCGImagePropertyPixelHeight] as? Int,
              width > 0, height > 0 else { throw Failure.invalid }
        guard width <= 24_000_000 / height else { throw Failure.tooLarge }
        func scaled(_ limit: Int) throws -> CGImage {
            let options: [CFString: Any] = [
                kCGImageSourceCreateThumbnailFromImageAlways: true,
                kCGImageSourceCreateThumbnailWithTransform: true,
                kCGImageSourceThumbnailMaxPixelSize: limit
            ]
            guard let image = CGImageSourceCreateThumbnailAtIndex(source, 0, options as CFDictionary) else {
                throw Failure.invalid
            }
            return image
        }
        let pixels = try scaled(min(4_096, max(width, height)))
        let encoded = NSMutableData()
        guard let output = CGImageDestinationCreateWithData(encoded, UTType.png.identifier as CFString, 1, nil) else {
            throw Failure.invalid
        }
        CGImageDestinationAddImage(output, pixels, nil)
        guard CGImageDestinationFinalize(output) else { throw Failure.invalid }
        guard encoded.length <= Self.maxBytes else { throw Failure.tooLarge }
        png = encoded as Data
        thumbnail = try scaled(min(192, max(width, height)))
    }

    @MainActor
    static func load(from provider: NSItemProvider) async throws -> DraftImage {
        guard let type = provider.registeredTypeIdentifiers.first(where: { UTType($0)?.conforms(to: .image) == true }) else {
            throw Failure.invalid
        }
        return try await withCheckedThrowingContinuation { continuation in
            provider.loadFileRepresentation(forTypeIdentifier: type) { url, _ in
                do {
                    guard let url else { throw Failure.invalid }
                    let file = try FileHandle(forReadingFrom: url)
                    defer { try? file.close() }
                    guard try file.seekToEnd() <= UInt64(Self.maxBytes) else { throw Failure.tooLarge }
                    try file.seek(toOffset: 0)
                    guard let bytes = try file.read(upToCount: Self.maxBytes + 1) else { throw Failure.invalid }
                    continuation.resume(returning: try DraftImage(data: bytes))
                } catch {
                    continuation.resume(throwing: error)
                }
            }
        }
    }
}
