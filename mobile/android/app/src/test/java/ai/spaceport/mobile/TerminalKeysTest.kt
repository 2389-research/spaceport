// ABOUTME: Tests the saved toolbar key list and the Herdr key each toolbar key sends.
// ABOUTME: Covers damaged saved values, list bounds and the modifier keys that arm.
package ai.spaceport.mobile

import org.junit.Assert.*
import org.junit.Test

class TerminalKeysTest {
    @Test fun defaultListMatchesTheOriginalToolbar() {
        assertEquals(
            listOf("Ctrl", "Esc", "Tab", "←", "↑", "↓", "→", "↵"),
            TerminalKeys.standard.keys.map { it.keycap },
        )
    }

    @Test fun savedListRoundTripsAndRejectsDamage() {
        val keys = TerminalKeys.standard.with(TerminalKey.PAGE_UP).without(0)
        assertEquals(keys, TerminalKeys.parse(keys.encoded))
        for (damaged in listOf(null, "", "ctrl,,tab", "ctrl,warp", List(17) { "esc" }.joinToString(","))) {
            assertEquals(TerminalKeys.standard, TerminalKeys.parse(damaged))
        }
    }

    @Test fun listKeepsBetweenOneAndSixteenKeys() {
        var keys = TerminalKeys.parse("esc")
        assertEquals(keys, keys.without(0))
        repeat(20) { keys = keys.with(TerminalKey.TAB) }
        assertEquals(TerminalKeys.LIMIT, keys.keys.size)
        val moved = TerminalKeys.standard.moved(1, -1)
        assertEquals(listOf(TerminalKey.ESC, TerminalKey.CTRL), moved.keys.take(2))
        assertEquals(moved, moved.moved(0, -1))
    }

    @Test fun modifiersArmAndOtherKeysSendHerdrNames() {
        assertEquals(KeyAction.Arm(1), TerminalKey.SHIFT.action)
        assertEquals(KeyAction.Arm(2), TerminalKey.CTRL.action)
        assertEquals(KeyAction.Arm(4), TerminalKey.ALT.action)
        assertEquals(KeyAction.Send("Tab", 1), TerminalKey.SHIFT_TAB.action)
        assertEquals(KeyAction.Send("PageDown", 0), TerminalKey.PAGE_DOWN.action)
        assertEquals(KeyAction.Send("Esc", 0), TerminalKey.ESC.action)
        assertEquals(KeyAction.Send("Backspace", 0), TerminalKey.BACKSPACE.action)
    }
}
