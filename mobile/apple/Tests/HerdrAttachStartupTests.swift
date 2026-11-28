// ABOUTME: Exercises the direct-attach prelude captured from Herdr 0.9.5 on a PTY.
// ABOUTME: Checks fragmentation and preserves application queries, malformed prefixes and early EOF.
import Foundation
import XCTest
@testable import SpaceportMobile

final class HerdrAttachStartupTests: XCTestCase {
    private let reset = "\u{1b}[?1006l\u{1b}[?1016l\u{1b}[?1015l\u{1b}[?1005l\u{1b}[?1003l\u{1b}[?1002l\u{1b}[?1000l\u{1b}[?9l"
    private let probe = "\u{1b}[?1016$p\u{1b}[c"
    private var modes: String { reset + "\u{1b}[?1049h" + reset + "\u{1b}[?2031l" }

    func testCachedScreenSurvivesStartupUntilPaneOutputArrives() {
        var startup = HerdrAttachStartup()
        for byte in Data((modes + probe).utf8) {
            _ = startup.receive(Data([byte]))
            XCTAssertFalse(startup.hasPaneOutput)
        }
        _ = startup.receive(Data())
        XCTAssertFalse(startup.hasPaneOutput)
        _ = startup.receive(Data("live".utf8))
        XCTAssertTrue(startup.hasPaneOutput)

        var otherHost = HerdrAttachStartup()
        _ = otherHost.receive(Data("prompt".utf8))
        XCTAssertTrue(otherHost.hasPaneOutput)
        var coalesced = HerdrAttachStartup()
        _ = coalesced.receive(Data((modes + probe + "prompt").utf8))
        XCTAssertTrue(coalesced.hasPaneOutput)
    }

    func testOnlyTheOuterProbeIsRemovedAcrossEverySplit() {
        let pane = "\u{1b}[?2004h$ " + probe + "echo"
        let bytes = Data((modes + probe + pane).utf8)
        let expected = Data((modes + pane).utf8)
        for split in 0...bytes.count {
            var startup = HerdrAttachStartup()
            var result = startup.receive(bytes.prefix(split))
            result += startup.receive(bytes.dropFirst(split))
            result += startup.finish()
            XCTAssertEqual(result, expected, "split \(split)")
        }
        var startup = HerdrAttachStartup()
        let fragmented = bytes.reduce(into: Data()) { output, byte in
            output += startup.receive(Data([byte]))
        }
        XCTAssertEqual(fragmented, expected)
        XCTAssertEqual(startup.receive(bytes), bytes, "The matcher retires after startup.")
    }

    func testMismatchesAndEveryTruncatedPrefixPassThroughUnchanged() {
        let capture = Data((modes + probe).utf8)
        for count in 0..<capture.count {
            let prefix = Data(capture.prefix(count))
            var truncated = HerdrAttachStartup()
            XCTAssertEqual(truncated.receive(prefix) + truncated.finish(), prefix)
            XCTAssertTrue(truncated.finish().isEmpty)
            var unknown = HerdrAttachStartup()
            let suffix = Data([0xff]) + capture
            let visiblePrefix = unknown.receive(prefix)
            XCTAssertEqual(visiblePrefix + unknown.receive(suffix) + unknown.finish(), prefix + suffix)
        }
        var unknown = HerdrAttachStartup()
        XCTAssertEqual(unknown.receive(Data(probe.utf8)), Data(probe.utf8))
        let output = Data(repeating: 0x5a, count: 64 * 1024)
        XCTAssertEqual(unknown.receive(output), output)
    }
}
