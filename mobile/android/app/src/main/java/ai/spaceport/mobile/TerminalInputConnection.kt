// ABOUTME: Translates Android IME callbacks to terminal input for one exact pane.
// ABOUTME: Keeps marked text local and ignores callbacks from a superseded keyboard.
package ai.spaceport.mobile

import android.text.InputType
import android.view.KeyEvent
import android.view.inputmethod.BaseInputConnection
import android.view.inputmethod.EditorInfo
import uniffi.spaceport_mobile.InputTarget

internal class TerminalInputConnection(
    private val surface: TerminalSurface,
    private val destination: InputTarget,
    attributes: EditorInfo
) : BaseInputConnection(surface, false) {
    private val composition = TerminalComposition()
    private var closed = false
    private val active get() = !closed && surface.target == destination

    init {
        attributes.inputType = InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS or
            InputType.TYPE_TEXT_VARIATION_VISIBLE_PASSWORD
        attributes.imeOptions = EditorInfo.IME_ACTION_NONE or EditorInfo.IME_FLAG_NO_EXTRACT_UI
        attributes.imeOptions = attributes.imeOptions or EditorInfo.IME_FLAG_NO_PERSONALIZED_LEARNING
        attributes.initialSelStart = 0
        attributes.initialSelEnd = 0
    }

    override fun setComposingText(text: CharSequence?, newCursorPosition: Int): Boolean {
        if (!active) return false
        composition.mark(text?.toString().orEmpty())
        surface.composing(destination, composition.marked)
        return true
    }

    override fun commitText(text: CharSequence?, newCursorPosition: Int): Boolean {
        if (!active) return false
        val committed = composition.commit(text?.toString().orEmpty())
        surface.composing(destination, "")
        return surface.text(destination, committed)
    }

    override fun finishComposingText(): Boolean {
        if (!active) return false
        val committed = composition.finish()
        surface.composing(destination, "")
        return surface.text(destination, committed)
    }

    override fun getTextBeforeCursor(n: Int, flags: Int): CharSequence =
        if (active) composition.marked.takeLast(n.coerceAtLeast(0)) else ""
    override fun getTextAfterCursor(n: Int, flags: Int): CharSequence = ""
    override fun getSelectedText(flags: Int): CharSequence = ""

    override fun deleteSurroundingText(beforeLength: Int, afterLength: Int): Boolean {
        if (!active || beforeLength !in 0..256 || afterLength !in 0..256) return false
        val remaining = composition.deleteBefore(beforeLength)
        surface.composing(destination, composition.marked)
        repeat(remaining) { if (!surface.key(destination, "Backspace")) return false }
        repeat(afterLength) { if (!surface.key(destination, "Delete")) return false }
        return true
    }

    override fun deleteSurroundingTextInCodePoints(beforeLength: Int, afterLength: Int): Boolean {
        if (!active || beforeLength !in 0..256) return false
        val value = composition.marked
        val local = minOf(value.codePointCount(0, value.length), beforeLength)
        val start = value.offsetByCodePoints(value.length, -local)
        return deleteSurroundingText(value.length - start + beforeLength - local, afterLength)
    }

    override fun sendKeyEvent(event: KeyEvent): Boolean =
        active && (event.action == KeyEvent.ACTION_UP || surface.hardware(event, destination))
    override fun performEditorAction(actionCode: Int): Boolean = active && surface.key(destination, "Enter")

    override fun closeConnection() {
        closed = true
        composition.mark("")
        super.closeConnection()
    }
}
