// ABOUTME: Turns travel during a held terminal touch into semantic arrow keys.
// ABOUTME: Carries partial steps between samples and caps the keys one sample sends.
package ai.spaceport.mobile

import kotlin.math.abs

/** [step] is the horizontal travel per arrow; vertical travel per arrow is 1.8 times longer. */
internal class TerminalTrackpad(private var x: Float, private var y: Float, private val step: Float) {
    var moved = false
        private set

    fun move(toX: Float, toY: Float): List<String> {
        val vertical = step * 1.8f
        val dx = (toX - x) / step
        val dy = (toY - y) / vertical
        val horizontal = abs(dx) >= abs(dy)
        val distance = if (horizontal) dx else dy
        if (abs(distance) < 1) return emptyList()
        moved = true
        val steps = distance.toInt()
        if (horizontal) {
            x += steps * step
            y = toY
        } else {
            x = toX
            y += steps * vertical
        }
        val key = if (horizontal) (if (steps > 0) "Right" else "Left") else (if (steps > 0) "Down" else "Up")
        return List(minOf(8, abs(steps))) { key }
    }
}
