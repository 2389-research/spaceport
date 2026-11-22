// ABOUTME: Uses UIKit's marked-text API to exercise the draft editor's paste guard.
// ABOUTME: Verifies that a paste can read the native text only after the IME commits.
import SwiftUI
import UIKit
import XCTest
@testable import SpaceportMobile

@MainActor
final class DraftEditorTests: XCTestCase {
    func testAcceptingMarkedTextVerbatimUpdatesTheDraftBeforeTheNextRefresh() {
        var stored = "A "
        let editor = DraftTextView(content: Binding(get: { stored }, set: { stored = $0 }))
        editor.selectedRange = NSRange(location: 2, length: 0)
        editor.setMarkedText("かな", selectedRange: NSRange(location: 2, length: 0))
        XCTAssertNil(editor.committedText)
        XCTAssertEqual(stored, "A ")
        editor.display(stored)
        XCTAssertEqual(editor.text, "A かな")
        editor.unmarkText()
        XCTAssertEqual(stored, "A かな")
        editor.display(stored)
        XCTAssertEqual(editor.committedText, "A かな")
    }

    func testPasteReadsOnlyCommittedEditorText() {
        let editing = DraftEditing()
        XCTAssertNil(editing.committedText)
        let editor = DraftTextView(content: .constant(""))
        editing.editor = editor
        editor.text = "A "
        editor.selectedRange = NSRange(location: 2, length: 0)
        editor.setMarkedText("に", selectedRange: NSRange(location: 1, length: 0))
        XCTAssertNotNil(editor.markedTextRange)
        XCTAssertNil(editing.committedText)
        editor.insertText("日本")
        editor.unmarkText()
        XCTAssertEqual(editing.committedText, "A 日本")
    }
}
