// ABOUTME: Verifies that agent identities select the desktop artwork in the installed bundle.
// ABOUTME: Checks raster color, vector template rendering and the unrecognized-agent fallback.
import SwiftUI
import XCTest
@testable import SpaceportMobile

final class AgentBadgeTests: XCTestCase {
    @MainActor
    func testKnownIdentitiesLoadTheirMarksFromTheBundle() throws {
        for identity in ["codex", "claude", "gemini", "omo"] {
            let expected = try XCTUnwrap(UIImage(named: "agent-" + identity))
            let mark = try XCTUnwrap(AgentBadge.artwork(for: identity))
            XCTAssertEqual(mark.pngData(), expected.pngData(), identity)
            XCTAssertEqual(mark.renderingMode, identity == "omo" ? .alwaysOriginal : .alwaysTemplate)
        }
    }

    @MainActor
    func testUnknownMissingAndPathLikeIdentitiesUseGenericArtwork() throws {
        let generic = try render(identity: "generic")
        for identity in [nil, "", "Reviewer", "Claude Code", "../codex", "codex.svg", "AppIcon"] {
            XCTAssertEqual(try render(identity: identity), generic)
        }
    }

    @MainActor
    func testDifferentCustomAgentsKeepDistinctInitialsInTheRail() throws {
        XCTAssertNotEqual(try render(identity: nil, name: "Reviewer"), try render(identity: nil, name: "Scout"))
        XCTAssertEqual(try render(identity: nil, name: "Reviewer"), try render(identity: nil, name: "Rose"))
    }

    @MainActor
    private func render(identity: String?, name: String? = nil) throws -> Data {
        let badge = AgentBadge(identity: identity, fallbackName: name).frame(width: 21, height: 21)
        return try XCTUnwrap(ImageRenderer(content: badge).uiImage?.pngData())
    }
}
