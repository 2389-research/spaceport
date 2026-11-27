// ABOUTME: Renders the privacy cover over contrasting terminal backgrounds on iOS.
// ABOUTME: Checks edge-to-edge opacity and captures portrait and unfolded layouts.
import SwiftUI
import XCTest
@testable import SpaceportMobile

@MainActor
final class SessionPrivacyCoverTests: XCTestCase {
    func testCoverConcealsEveryPixelInBothAppearancesAndLayouts() throws {
        for (name, size) in [("Phone", CGSize(width: 393, height: 852)),
                             ("Duo", CGSize(width: 920, height: 654))] {
            var variants = [Data]()
            for scheme in [ColorScheme.light, .dark] {
                func render(over background: Color) throws -> UIImage {
                    let scene = background.overlay { SessionPrivacyCover() }
                        .frame(width: size.width, height: size.height)
                        .environment(\.colorScheme, scheme)
                    return try XCTUnwrap(ImageRenderer(content: scene).uiImage)
                }
                let image = try render(over: .red)
                let pixels = try XCTUnwrap(image.pngData())
                XCTAssertEqual(image.size, size)
                XCTAssertEqual(pixels, try render(over: .green).pngData())
                XCTAssertTrue(try containsRocketFlame(image), "The warm rocket plume should be visible in both themes.")
                variants.append(pixels)
                let capture = XCTAttachment(image: image)
                capture.name = "\(name)-\(scheme == .dark ? "dark" : "light")"
                capture.lifetime = .keepAlways
                add(capture)
            }
            XCTAssertNotEqual(variants.first, variants.last)
        }
    }

    func testCoverAppearsAboveAlertsWithoutClaimingTheSessionWindow() throws {
        let session = try XCTUnwrap(UIApplication.shared.connectedScenes.compactMap { ($0 as? UIWindowScene)?.keyWindow }.first)
        let scene = try XCTUnwrap(session.windowScene)
        let anchor = SessionPrivacyWindow.Anchor()
        session.addSubview(anchor)
        defer { anchor.removeFromSuperview() }

        for style in [UIUserInterfaceStyle.light, .dark] {
            anchor.update(concealed: true, style: style)
            let shield = try XCTUnwrap(scene.windows.first { !$0.isHidden && $0.rootViewController is UIHostingController<SessionPrivacyCover> })
            XCTAssertGreaterThan(shield.windowLevel.rawValue, UIWindow.Level.alert.rawValue)
            XCTAssertEqual(shield.overrideUserInterfaceStyle, style)
            XCTAssertEqual(shield.bounds, session.bounds)
            XCTAssertTrue(session.isKeyWindow)
            if style == .dark { anchor.removeFromSuperview() }
            else { anchor.update(concealed: false, style: style) }
            XCTAssertTrue(shield.isHidden)
        }
    }

    private func containsRocketFlame(_ image: UIImage) throws -> Bool {
        let bitmap = try XCTUnwrap(image.cgImage)
        let context = try XCTUnwrap(CGContext(data: nil, width: bitmap.width, height: bitmap.height,
                                              bitsPerComponent: 8, bytesPerRow: bitmap.width * 4,
                                              space: CGColorSpaceCreateDeviceRGB(),
                                              bitmapInfo: CGImageAlphaInfo.noneSkipLast.rawValue))
        context.draw(bitmap, in: CGRect(origin: .zero, size: image.size))
        let bytes = try XCTUnwrap(context.data).assumingMemoryBound(to: UInt8.self)
        return stride(from: 0, to: bitmap.width * bitmap.height * 4, by: 4).contains { i in
            Int(bytes[i]) - Int(bytes[i + 1]) > 55 && Int(bytes[i + 1]) - Int(bytes[i + 2]) > 35
        }
    }
}
