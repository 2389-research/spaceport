// ABOUTME: Renders successive rocket frames with the native SwiftUI text renderer.
// ABOUTME: Checks that movement changes the artwork without resizing the waiting screen.
import SwiftUI
import XCTest
@testable import SpaceportMobile

final class ConnectionFlightTests: XCTestCase {
    #if os(iOS)
    @MainActor
    func testPhoneShowsRocketDuringHostPreflight() throws {
        let profile = try HostProfile.make(hostname: "flight.invalid", port: "22", username: "pilot",
                                           session: "default", login: .password)
        let connection = MobileConnection(profile: profile)
        connection.firstOpen { _ in }
        XCTAssertEqual(connection.phase, .connecting)
        let session = SessionScreen(presentation: .init(connection: connection), hostDetails: {}, home: {}, disconnect: {})
        for scheme in [ColorScheme.light, .dark] {
            let screen = session.frame(width: 320, height: 480)
                .environment(\.scenePhase, .active).environment(\.colorScheme, scheme)
            let image = try XCTUnwrap(ImageRenderer(content: screen).uiImage)
            let attachment = XCTAttachment(image: image)
            attachment.name = "Connecting – \(scheme)"
            attachment.lifetime = .keepAlways
            add(attachment)

            // The orange exhaust distinguishes the rocket from text, a spinner or an empty screen.
            let context = try XCTUnwrap(CGContext(data: nil, width: 320, height: 480, bitsPerComponent: 8,
                                                  bytesPerRow: 320 * 4, space: CGColorSpaceCreateDeviceRGB(),
                                                  bitmapInfo: CGImageAlphaInfo.premultipliedLast.rawValue))
            context.draw(try XCTUnwrap(image.cgImage), in: CGRect(x: 0, y: 0, width: 320, height: 480))
            let bytes = try XCTUnwrap(context.data).assumingMemoryBound(to: UInt8.self)
            XCTAssertTrue(stride(from: 0, to: 320 * 480 * 4, by: 4).contains { pixel in
                bytes[pixel] > 200 && bytes[pixel + 1] > 65 && bytes[pixel + 1] < 195 && bytes[pixel + 2] < 80
            })
        }
    }
    #endif

    @MainActor
    func testRocketMovesWithinTheSameCompactFrameInBothAppearances() throws {
        for scheme in [ColorScheme.light, .dark] {
            func render(_ step: Int) throws -> CGImage {
                let artwork = ConnectionFlight.Frame(step: step)
                    .environment(\.colorScheme, scheme)
                return try XCTUnwrap(ImageRenderer(content: artwork).cgImage)
            }
            let first = try render(0)
            let next = try render(1)
            XCTAssertLessThanOrEqual(first.width, 240)
            XCTAssertLessThanOrEqual(first.height, 110)
            XCTAssertEqual(first.width, next.width)
            XCTAssertEqual(first.height, next.height)
            func pixels(_ image: CGImage) throws -> Data {
                try XCTUnwrap(image.dataProvider?.data) as Data
            }
            XCTAssertNotEqual(try pixels(first), try pixels(next))
            let paused = ConnectionFlight().environment(\.scenePhase, .inactive).environment(\.colorScheme, scheme)
            let still = try XCTUnwrap(ImageRenderer(content: paused).cgImage)
            XCTAssertEqual(try pixels(first), try pixels(still))
        }
    }
}
