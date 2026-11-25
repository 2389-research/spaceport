// ABOUTME: Tests how travel during a terminal hold becomes arrow keys.
// ABOUTME: Covers the dominant axis, carried remainders and the per-sample cap.
package ai.spaceport.mobile

import org.junit.Assert.*
import org.junit.Test

class TerminalTrackpadTest {
    @Test fun smallTravelSendsNothingAndKeepsTheHoldStationary() {
        val pad = TerminalTrackpad(100f, 100f, step = 10f)
        assertEquals(emptyList<String>(), pad.move(105f, 108f))
        assertFalse(pad.moved)
    }

    @Test fun dominantAxisChoosesTheArrowAndRemaindersCarry() {
        val pad = TerminalTrackpad(0f, 0f, step = 10f)
        assertEquals(listOf("Right", "Right"), pad.move(25f, 3f))
        assertTrue(pad.moved)
        assertEquals(listOf("Right"), pad.move(30f, 3f))
        assertEquals(listOf("Up"), pad.move(30f, -18f))
        assertEquals(listOf("Left"), pad.move(19f, -18f))
    }

    @Test fun oneSampleSendsAtMostEightKeys() {
        val pad = TerminalTrackpad(0f, 0f, step = 10f)
        assertEquals(List(8) { "Down" }, pad.move(0f, 1000f))
    }
}
