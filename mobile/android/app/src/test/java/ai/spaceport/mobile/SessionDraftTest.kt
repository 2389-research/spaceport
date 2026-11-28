// ABOUTME: Verifies unsent text and screenshot ownership across terminal presentations.
// ABOUTME: Keeps a draft fenced to the original pane until the operator selects another.
package ai.spaceport.mobile

import org.junit.Assert.*
import org.junit.Test
import uniffi.spaceport_mobile.InputTarget

class SessionDraftTest {
    @Test fun retainedDraftNeedsExplicitRetargetAfterReconnectOrPaneReplacement() {
        val first = SessionDraft()
        val second = SessionDraft()
        val original = InputTarget(3u, "boot", "pane-A", null, 7u)
        val other = original.copy(connection = 4u, paneId = "pane-B")
        first.present(original, "First")
        first.note = "Do this later"
        first.png = byteArrayOf(1)
        second.present(other, "Second")
        assertEquals("", second.note)
        assertNull(second.png)
        first.present(other, "Different pane")
        assertEquals(original, first.destination)
        assertEquals("First", first.label)
        assertFalse(first.canSend(other))
        first.retarget(other, "Different pane")
        assertTrue(first.canSend(other))
        assertArrayEquals(byteArrayOf(1),first.png)
        first.clear()
        assertNull(first.png)
        assertEquals("", first.note)
        assertNull(first.destination)
    }
}
