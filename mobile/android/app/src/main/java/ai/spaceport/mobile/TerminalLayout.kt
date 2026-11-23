// ABOUTME: Computes Herdr viewport sizes and converts its wire palette to ARGB.
// ABOUTME: Holds IME composition until the keyboard commits the final text.
package ai.spaceport.mobile

import kotlin.math.roundToInt

internal data class GridSize(val columns: Int, val rows: Int, val cellWidth: Int, val cellHeight: Int)

internal fun viewport(width: Int, height: Int, cellWidth: Float, cellHeight: Float) = GridSize(
    (width / cellWidth).toInt().coerceIn(1, 512),
    (height / cellHeight).toInt().coerceIn(1, 512),
    cellWidth.roundToInt().coerceAtLeast(1),
    cellHeight.roundToInt().coerceAtLeast(1)
)

private val palette = intArrayOf(
    0x121212, 0xcc5555, 0x64b66a, 0xc7a74e, 0x598ad1, 0xb877c5, 0x55b8b6, 0xc8c8c8,
    0x777777, 0xf98080, 0x90df96, 0xf2d878, 0x8db6ed, 0xd89ee8, 0x84dedb, 0xffffff
)
private val levels = intArrayOf(0, 95, 135, 175, 215, 255)

internal fun terminalColor(value: UInt, foreground: Boolean): Int {
    val index = value.toInt() and 255
    val rgb = when (value shr 24) {
        2u -> value.toInt() and 0xffffff
        1u -> when (index) {
            in 0..15 -> palette[index]
            in 16..231 -> {
                val cube = index - 16
                (levels[cube / 36] shl 16) or (levels[cube / 6 % 6] shl 8) or levels[cube % 6]
            }
            else -> (8 + 10 * (index - 232)) * 0x010101
        }
        else -> if (value in 1u..16u) palette[value.toInt() - 1] else if (foreground) 0xe5e5e7 else 0x1c1c1e
    }
    return 0xff000000.toInt() or rgb
}

internal class TerminalComposition {
    var marked = ""
        private set

    fun mark(text: String) { marked = text }
    fun commit(text: String): String { marked = ""; return text }
    fun finish(): String = commit(marked)

    fun deleteBefore(characters: Int): Int {
        val used = minOf(characters, marked.length)
        marked = marked.dropLast(used)
        if (marked.lastOrNull()?.isHighSurrogate() == true) marked = marked.dropLast(1)
        return characters - used
    }
}
