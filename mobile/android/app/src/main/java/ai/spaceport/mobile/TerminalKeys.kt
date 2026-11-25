// ABOUTME: Names the keys the terminal toolbar can show and the Herdr key each one sends.
// ABOUTME: Saves the chosen toolbar keys as one bounded, ordered list in app preferences.
package ai.spaceport.mobile

internal sealed interface KeyAction {
    /** Holds a modifier for the next key: 1 Shift, 2 Control, 4 Alt. */
    data class Arm(val flag: Int) : KeyAction

    data class Send(val name: String, val modifiers: Int) : KeyAction
}

internal enum class TerminalKey(val saved: String, val title: String, val keycap: String, val action: KeyAction) {
    CTRL("ctrl", "Control", "Ctrl", KeyAction.Arm(2)),
    SHIFT("shift", "Shift", "Shift", KeyAction.Arm(1)),
    ALT("alt", "Alt", "Alt", KeyAction.Arm(4)),
    ESC("esc", "Escape", "Esc", KeyAction.Send("Esc", 0)),
    TAB("tab", "Tab", "Tab", KeyAction.Send("Tab", 0)),
    SHIFT_TAB("shiftTab", "Shift Tab", "⇧Tab", KeyAction.Send("Tab", 1)),
    ENTER("enter", "Enter", "↵", KeyAction.Send("Enter", 0)),
    BACKSPACE("backspace", "Backspace", "⌫", KeyAction.Send("Backspace", 0)),
    DELETE("delete", "Delete", "Del", KeyAction.Send("Delete", 0)),
    LEFT("left", "Left Arrow", "←", KeyAction.Send("Left", 0)),
    UP("up", "Up Arrow", "↑", KeyAction.Send("Up", 0)),
    DOWN("down", "Down Arrow", "↓", KeyAction.Send("Down", 0)),
    RIGHT("right", "Right Arrow", "→", KeyAction.Send("Right", 0)),
    HOME("home", "Home", "Home", KeyAction.Send("Home", 0)),
    END("end", "End", "End", KeyAction.Send("End", 0)),
    PAGE_UP("pageUp", "Page Up", "PgUp", KeyAction.Send("PageUp", 0)),
    PAGE_DOWN("pageDown", "Page Down", "PgDn", KeyAction.Send("PageDown", 0)),
}

/** The toolbar's keys in display order; always between one and [LIMIT] keys. */
internal data class TerminalKeys private constructor(val keys: List<TerminalKey>) {
    val encoded get() = keys.joinToString(",") { it.saved }

    fun with(key: TerminalKey) = if (keys.size >= LIMIT) this else TerminalKeys(keys + key)

    fun without(index: Int) =
        if (keys.size <= 1 || index !in keys.indices) this else TerminalKeys(keys.filterIndexed { i, _ -> i != index })

    fun moved(index: Int, by: Int): TerminalKeys {
        val to = index + by
        if (index !in keys.indices || to !in keys.indices) return this
        return TerminalKeys(keys.toMutableList().apply { add(to, removeAt(index)) })
    }

    companion object {
        const val LIMIT = 16
        val standard = TerminalKeys(
            listOf(
                TerminalKey.CTRL, TerminalKey.ESC, TerminalKey.TAB, TerminalKey.LEFT,
                TerminalKey.UP, TerminalKey.DOWN, TerminalKey.RIGHT, TerminalKey.ENTER,
            )
        )

        /** Reads a saved list, falling back to [standard] when any part is damaged. */
        fun parse(saved: String?): TerminalKeys {
            val slots = saved?.split(",") ?: return standard
            if (slots.size !in 1..LIMIT) return standard
            val keys = slots.map { slot -> TerminalKey.entries.firstOrNull { it.saved == slot } ?: return standard }
            return TerminalKeys(keys)
        }
    }
}
