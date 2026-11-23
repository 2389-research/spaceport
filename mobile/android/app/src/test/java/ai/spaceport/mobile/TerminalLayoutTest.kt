// ABOUTME: Tests the viewport, wire colors and committed keyboard composition.
// ABOUTME: Covers both resized grids and the final text an IME submits.
package ai.spaceport.mobile

import org.junit.Assert.*
import org.junit.Test

class TerminalLayoutTest {
    @Test fun viewportBoundsMatchTheRemoteGrid() {
        assertEquals(GridSize(1, 1, 8, 17), viewport(4, 7, 8f, 17f))
        assertEquals(GridSize(80, 23, 8, 17), viewport(647, 400, 8f, 17f))
        assertEquals(512, viewport(5000, 5000, 8f, 17f).columns)
    }

    @Test fun colorsIncludeCubeGrayAndTrueColor() {
        assertEquals(0xffe5e5e7.toInt(), terminalColor(0u, true))
        assertEquals(0xffcc5555.toInt(), terminalColor(2u, true))
        assertEquals(0xffff0000.toInt(), terminalColor(0x010000c4u, false))
        assertEquals(0xff080808.toInt(), terminalColor(0x010000e8u, false))
        assertEquals(0xffab1234.toInt(), terminalColor(0x02ab1234u, false))
    }

    @Test fun compositionCommitsReplacementOnlyOnce() {
        val ime = TerminalComposition()
        ime.mark("he")
        ime.mark("hel")
        assertEquals("hello", ime.commit("hello"))
        assertEquals("", ime.finish())
        ime.mark("かな")
        assertEquals("かな", ime.finish())
        ime.mark("🚀x")
        assertEquals(0, ime.deleteBefore(1))
        assertEquals("🚀", ime.finish())
        assertEquals(2, ime.deleteBefore(2))
    }
}
