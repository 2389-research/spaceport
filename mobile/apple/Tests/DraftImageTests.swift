// ABOUTME: Exercises screenshot preparation and image-only terminal targeting.
// ABOUTME: Checks real PNG decoding, thumbnail bounds, and rejection of invalid input.
import UIKit
import XCTest
@testable import SpaceportMobile

final class DraftImageTests: XCTestCase {
    func testImageOnlyDraftRetainsOriginalPaneAndNeverQueuesAsText() throws {
        let renderer = UIGraphicsImageRenderer(size: CGSize(width: 90, height: 120))
        let png = renderer.pngData { context in
            UIColor.systemPurple.setFill()
            context.fill(CGRect(x: 0, y: 0, width: 90, height: 120))
        }
        let image = try DraftImage(data: png)
        XCTAssertLessThanOrEqual(max(image.thumbnail.width, image.thumbnail.height), 192)
        XCTAssertNotNil(UIImage(data: image.png))
        var draft = TerminalDraft()
        let target = InputTarget(connection: 1, bootId: "boot", paneId: "one", popupId: nil)
        var next = target
        next.paneId = "two"
        draft.open(at: .init(target: target, workspaces: [], tabs: [], panes: []))
        draft.image = image
        draft.open(at: .init(target: next, workspaces: [], tabs: [], panes: []))
        XCTAssertTrue(draft.canSend(to: target))
        XCTAssertFalse(draft.canSend(to: next))
        XCTAssertFalse(draft.send(to: target) { _, _ in XCTFail("Dropped image from paste"); return true })
        XCTAssertNotNil(draft.image)
    }

    @MainActor
    func testComposerLoadsAFileBackedImageProvider() async throws {
        let url = FileManager.default.temporaryDirectory.appendingPathComponent("\(UUID()).png")
        defer { try? FileManager.default.removeItem(at: url) }
        let format = UIGraphicsImageRendererFormat()
        format.scale = 1
        let pixels = UIGraphicsImageRenderer(size: CGSize(width: 12, height: 9), format: format)
        try pixels.pngData { context in context.fill(CGRect(x: 0, y: 0, width: 12, height: 9)) }.write(to: url)
        let provider = try XCTUnwrap(NSItemProvider(contentsOf: url))
        let image = try await DraftImage.load(from: provider)
        XCTAssertEqual(image.thumbnail.width, 12)
        XCTAssertEqual(image.thumbnail.height, 9)
        XCTAssertNotNil(UIImage(data: image.png))
    }

    func testRejectsInvalidAndOversizedInput() {
        XCTAssertThrowsError(try DraftImage(data: Data([1, 2, 3])))
        XCTAssertThrowsError(try DraftImage(data: Data(repeating: 0, count: 16 * 1024 * 1024 + 1)))
    }
}
