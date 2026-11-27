// ABOUTME: Checks local share publication and the native file size boundary.
// ABOUTME: Exercises the same file reader the system picker and share sheet use.
import XCTest
@testable import SpaceportMobile

final class DraftFileTests: XCTestCase {
    func testSharePreservesFileAndRejectsASecondPendingShare() throws {
        let directory = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
        try FileManager.default.createDirectory(at: directory, withIntermediateDirectories: true)
        defer { try? FileManager.default.removeItem(at: directory) }
        let url = directory.appendingPathComponent("agent's notes.txt")
        try Data([0, 23, 255]).write(to: url)
        let file = try DraftFile(url: url)
        try SharedDraftInbox.save(file, in: directory)
        XCTAssertThrowsError(try SharedDraftInbox.save(file, in: directory))
        let recovered = try SharedDraftInbox.read(XCTUnwrap(SharedDraftInbox.pending(in: directory)))
        XCTAssertEqual(recovered.id, file.id)
        XCTAssertEqual(recovered.name, file.name)
        XCTAssertEqual(recovered.data, file.data)
        let original = try XCTUnwrap(SharedDraftInbox.pending(in: directory))
        try SharedDraftInbox.remove(original)
        let replacement = try DraftFile(url: url)
        try SharedDraftInbox.save(replacement, in: directory)
        XCTAssertThrowsError(try SharedDraftInbox.remove(original))
        XCTAssertEqual(try SharedDraftInbox.read(XCTUnwrap(SharedDraftInbox.pending(in: directory))).id, replacement.id)
        try Data(count: DraftFile.limit + 1).write(to: url)
        XCTAssertThrowsError(try DraftFile(url: url))
    }
}
