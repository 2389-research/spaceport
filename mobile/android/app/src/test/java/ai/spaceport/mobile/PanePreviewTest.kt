// ABOUTME: Checks concealment and wide glyph continuations in readable pane text.
// ABOUTME: Keeps pane previews consistent with the terminal accessibility surface.
package ai.spaceport.mobile

import org.junit.Assert.assertEquals
import org.junit.Test
import uniffi.spaceport_mobile.MobileCell

class PanePreviewTest {
    @Test fun previewTextOmitsHiddenContentAndContinuationCells() {
        val cell = MobileCell("x", 0u, 0u, 0u, false)
        assertEquals("x", cell.displayText())
        assertEquals(" ", cell.copy(symbol = "secret", modifier = 128u).displayText())
        assertEquals("", cell.copy(skip = true).displayText())
    }
}
