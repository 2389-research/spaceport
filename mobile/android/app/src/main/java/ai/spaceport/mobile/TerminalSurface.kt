// ABOUTME: Draws Herdr cells and routes gestures and hardware keys to the selected terminal.
// ABOUTME: Shares one geometry for drawing, wheel coordinates and viewport negotiation.
package ai.spaceport.mobile

import android.content.Context
import android.graphics.Canvas
import android.graphics.Paint
import android.graphics.Typeface
import android.os.Bundle
import android.util.TypedValue
import android.view.KeyEvent
import android.view.MotionEvent
import android.view.View
import android.view.ViewConfiguration
import android.view.accessibility.AccessibilityEvent
import android.view.accessibility.AccessibilityManager
import android.view.accessibility.AccessibilityNodeInfo
import android.view.inputmethod.EditorInfo
import android.view.inputmethod.InputConnection
import android.view.inputmethod.InputMethodManager
import kotlin.math.abs
import kotlin.math.ceil
import uniffi.spaceport_mobile.InputTarget
import uniffi.spaceport_mobile.MobileSession
import uniffi.spaceport_mobile.MobileTerminal

// The session and input route are supplied by the Activity; this view never inflates from XML.
@android.annotation.SuppressLint("ViewConstructor")
internal class TerminalSurface(
    context: Context,
    private val route: (InputTarget, (MobileSession) -> Unit) -> Boolean,
    private val onViewport: () -> Unit,
    private val claimSize: () -> Unit,
    private val onHold: (row: Int) -> Unit,
    fontSize: Int,
) : View(context) {
    var target: InputTarget? = null
        private set
    var terminal: MobileTerminal? = null
        private set
    /** Toolbar modifiers held for the next key: 1 Shift, 2 Control, 4 Alt. */
    var armed = 0
        private set
    var onModifierUsed: () -> Unit = {}
    private val paint = Paint(Paint.ANTI_ALIAS_FLAG).apply { typeface = Typeface.MONOSPACE }
    private val pointSize = TypedValue.applyDimension(TypedValue.COMPLEX_UNIT_SP, fontSize.toFloat(), resources.displayMetrics)
    private val cellWidth = ceil(paint.apply { textSize = pointSize }.measureText("M"))
    private val cellHeight = ceil(paint.fontSpacing)
    private val slop = ViewConfiguration.get(context).scaledTouchSlop
    private var downX = 0f
    private var downY = 0f
    private var trackpad: TerminalTrackpad? = null
    private var multitouch = false
    private val hold = Runnable {
        val destination = gestureTarget
        if (destination != null && destination == target && !dragged) {
            trackpad = TerminalTrackpad(downX, downY, 10 * resources.displayMetrics.density)
            performHapticFeedback(android.view.HapticFeedbackConstants.LONG_PRESS)
        }
    }
    private var scrollYPosition = 0f
    private var dragged = false
    private var gestureTarget: InputTarget? = null
    private var marked = ""
    val gridSize get() = viewport(width, height, cellWidth, cellHeight)

    init {
        isFocusableInTouchMode = true
        isClickable = true
        importantForAccessibility = IMPORTANT_FOR_ACCESSIBILITY_YES
        contentDescription = "Terminal"
        setBackgroundColor(terminalColor(0u, false))
    }

    fun update(next: MobileTerminal?, destination: InputTarget?) {
        val changed = target != destination
        target = destination
        if (changed) {
            marked = ""
            clearModifier()
            if (isFocused) context.getSystemService(InputMethodManager::class.java).restartInput(this)
        }
        if (terminal !== next || changed) {
            terminal = next
            postInvalidateOnAnimation()
            if (isShown && context.getSystemService(AccessibilityManager::class.java).isEnabled) {
                sendAccessibilityEvent(AccessibilityEvent.TYPE_WINDOW_CONTENT_CHANGED)
            }
        }
    }

    fun toggleModifier(flag: Int) {
        armed = armed xor flag
        onModifierUsed()
    }

    private fun clearModifier() {
        if (armed != 0) { armed = 0; onModifierUsed() }
    }

    fun key(destination: InputTarget, name: String, modifiers: Int = 0): Boolean {
        if (destination != target) return false
        val flags = modifiers or armed
        val sent = route(destination) { it.sendKey(destination, name, flags.toUByte()) }
        if (sent) clearModifier()
        return sent
    }

    fun text(destination: InputTarget, value: String): Boolean {
        if (destination != target) return false
        return when {
            value.isEmpty() -> true
            value == "\n" || value == "\r" || value == "\r\n" -> key(destination, "Enter")
            value == "\t" -> key(destination, "Tab")
            armed != 0 && value.codePointCount(0, value.length) == 1 -> key(destination, value)
            else -> route(destination) { it.sendText(destination, value) }.also { if (it) clearModifier() }
        }
    }

    fun composing(destination: InputTarget, value: String) {
        if (destination == target) { marked = value; invalidate() }
    }

    override fun onCheckIsTextEditor() = true
    override fun onCreateInputConnection(outAttrs: EditorInfo): InputConnection? {
        val destination = target ?: return null
        return TerminalInputConnection(this, destination, outAttrs)
    }

    fun hardware(event: KeyEvent, destination: InputTarget): Boolean {
        if (event.action != KeyEvent.ACTION_DOWN || event.isMetaPressed) return false
        val flags = (if (event.isShiftPressed) 1 else 0) or (if (event.isCtrlPressed) 2 else 0) or
            (if (event.isAltPressed) 4 else 0)
        val named = when (event.keyCode) {
            KeyEvent.KEYCODE_DEL -> "Backspace"
            KeyEvent.KEYCODE_FORWARD_DEL -> "Delete"
            KeyEvent.KEYCODE_ENTER, KeyEvent.KEYCODE_NUMPAD_ENTER -> "Enter"
            KeyEvent.KEYCODE_TAB -> if (event.isShiftPressed) "BackTab" else "Tab"
            KeyEvent.KEYCODE_ESCAPE -> "Esc"
            KeyEvent.KEYCODE_DPAD_LEFT -> "Left"
            KeyEvent.KEYCODE_DPAD_RIGHT -> "Right"
            KeyEvent.KEYCODE_DPAD_UP -> "Up"
            KeyEvent.KEYCODE_DPAD_DOWN -> "Down"
            KeyEvent.KEYCODE_MOVE_HOME -> "Home"
            KeyEvent.KEYCODE_MOVE_END -> "End"
            KeyEvent.KEYCODE_PAGE_UP -> "PageUp"
            KeyEvent.KEYCODE_PAGE_DOWN -> "PageDown"
            KeyEvent.KEYCODE_INSERT -> "Insert"
            else -> null
        }
        if (named != null) return key(destination, named, flags)
        val modified = event.isCtrlPressed || event.isAltPressed
        val code = if (modified) event.getUnicodeChar(0) else event.unicodeChar
        if (!Character.isValidCodePoint(code) || Character.isISOControl(code)) return false
        val character = String(Character.toChars(code))
        return if (modified) key(destination, character, flags) else text(destination, character)
    }

    override fun onKeyDown(keyCode: Int, event: KeyEvent): Boolean =
        target?.let { hardware(event, it) } == true || super.onKeyDown(keyCode, event)

    override fun performClick(): Boolean {
        super.performClick()
        claimSize()
        requestFocus()
        context.getSystemService(InputMethodManager::class.java).showSoftInput(this, 0)
        return true
    }

    override fun onTouchEvent(event: MotionEvent): Boolean {
        when (event.actionMasked) {
            MotionEvent.ACTION_DOWN -> {
                downX = event.x
                downY = event.y
                trackpad = null
                postDelayed(hold, ViewConfiguration.getLongPressTimeout().toLong())
                scrollYPosition = event.y
                dragged = false
                multitouch = false
                gestureTarget = target
            }
            // A second finger ends the gesture: event.x and event.y read pointer index 0, whose
            // finger can change, so a held trackpad would otherwise jump and send stray arrows.
            MotionEvent.ACTION_POINTER_DOWN -> {
                removeCallbacks(hold)
                trackpad = null
                multitouch = true
            }
            MotionEvent.ACTION_MOVE -> {
                if (multitouch) return true
                trackpad?.let { pad ->
                    val destination = gestureTarget
                    if (destination == null || destination != target) return true
                    pad.move(event.x, event.y).forEach { key(destination, it) }
                    return true
                }
                if (abs(event.y - downY) > slop || abs(event.x - downX) > slop) {
                    dragged = true
                    removeCallbacks(hold)
                }
                val grid = terminal ?: return true
                val h = minOf(cellHeight, height.toFloat() / grid.height.toInt().coerceAtLeast(1)).coerceAtLeast(1f)
                val lines = ((event.y - scrollYPosition) / h).toInt()
                if (dragged && lines != 0) {
                    scroll(gestureTarget, lines, event.x, downY)
                    scrollYPosition += lines * h
                }
            }
            MotionEvent.ACTION_UP -> {
                removeCallbacks(hold)
                val pad = trackpad
                trackpad = null
                if (gestureTarget == target && !multitouch) {
                    if (pad != null && !pad.moved) cellAt(downX, downY)?.let { (_, row) -> onHold(row) }
                    else if (pad == null && !dragged) performClick()
                }
                gestureTarget = null
            }
            MotionEvent.ACTION_CANCEL -> {
                removeCallbacks(hold)
                trackpad = null
                gestureTarget = null
            }
        }
        return true
    }

    override fun onGenericMotionEvent(event: MotionEvent): Boolean {
        if (event.action == MotionEvent.ACTION_SCROLL) {
            return scroll(target, event.getAxisValue(MotionEvent.AXIS_VSCROLL).toInt(), event.x, event.y)
        }
        return super.onGenericMotionEvent(event)
    }

    /** The painted cell under a point, using the same cell size [onDraw] uses. */
    private fun cellAt(x: Float, y: Float): Pair<Int, Int>? {
        val grid = terminal ?: return null
        val columns = grid.width.toInt().coerceAtLeast(1)
        val rows = grid.height.toInt().coerceAtLeast(1)
        val col = (x / minOf(cellWidth, width.coerceAtLeast(1) / columns.toFloat())).toInt().coerceIn(0, columns - 1)
        val row = (y / minOf(cellHeight, height.coerceAtLeast(1) / rows.toFloat())).toInt().coerceIn(0, rows - 1)
        return col to row
    }

    private fun scroll(destination: InputTarget?, lines: Int, x: Float, y: Float): Boolean {
        if (destination == null || destination != target || lines == 0) return false
        val (col, row) = cellAt(x, y) ?: return false
        return route(destination) { it.scroll(destination, lines.coerceIn(-64, 64).toShort(), col.toUShort(), row.toUShort()) }
    }

    override fun onSizeChanged(w: Int, h: Int, oldw: Int, oldh: Int) = onViewport()

    override fun onDetachedFromWindow() {
        removeCallbacks(hold)
        trackpad = null
        super.onDetachedFromWindow()
    }

    override fun onDraw(canvas: Canvas) {
        val grid = terminal ?: return
        val cols = grid.width.toInt()
        val rows = grid.height.toInt()
        if (cols == 0 || rows == 0 || width == 0 || height == 0) return
        val w = minOf(cellWidth, width.toFloat() / cols)
        val h = minOf(cellHeight, height.toFloat() / rows)
        paint.textSize = pointSize * minOf(w / cellWidth, h / cellHeight)
        val baseline = -paint.ascent()
        val limit = minOf(cols * rows, grid.cells.size)
        // Paint every background first so continuation cells never overpaint a wide glyph.
        for (i in 0 until limit) {
            val cell = grid.cells[i]
            val reversed = (cell.modifier.toInt() and 64) != 0
            if (cell.bg == 0u && !reversed) continue
            val x = (i % cols) * w
            val y = (i / cols) * h
            paint.color = terminalColor(if (reversed) cell.fg else cell.bg, reversed)
            canvas.drawRect(x, y, x + w, y + h, paint)
        }
        for (i in 0 until limit) {
            val cell = grid.cells[i]
            val mod = cell.modifier.toInt()
            val x = (i % cols) * w
            val y = (i / cols) * h
            val reversed = mod and 64 != 0
            paint.color = terminalColor(if (reversed) cell.bg else cell.fg, !reversed)
            paint.alpha = if (mod and 2 != 0) 153 else 255
            if (!cell.skip && mod and 128 == 0 && cell.symbol != " " && cell.symbol.isNotEmpty()) {
                paint.isFakeBoldText = mod and 1 != 0
                paint.textSkewX = if (mod and 4 != 0) -0.20f else 0f
                canvas.drawText(cell.symbol, x, y + baseline, paint)
            }
            if (mod and 8 != 0) canvas.drawRect(x, y + h - 1, x + w, y + h, paint)
            if (mod and 256 != 0) canvas.drawRect(x, y + h / 2, x + w, y + h / 2 + 1, paint)
        }
        val col = grid.cursorX?.toInt() ?: return
        val row = grid.cursorY?.toInt() ?: return
        if (col >= cols || row >= rows) return
        paint.color = 0xccffffff.toInt()
        paint.style = Paint.Style.STROKE
        canvas.drawRect(col * w + 0.5f, row * h + 0.5f, (col + 1) * w - 0.5f, (row + 1) * h - 0.5f, paint)
        paint.style = Paint.Style.FILL
        if (marked.isNotEmpty()) {
            paint.color = terminalColor(0u, false)
            paint.isFakeBoldText = false
            paint.textSkewX = 0f
            canvas.drawRect(col * w, row * h, col * w + paint.measureText(marked), (row + 1) * h, paint)
            paint.color = terminalColor(0u, true)
            paint.isUnderlineText = true
            canvas.drawText(marked, col * w, row * h + baseline, paint)
            paint.isUnderlineText = false
        }
    }

    override fun onInitializeAccessibilityNodeInfo(info: AccessibilityNodeInfo) {
        super.onInitializeAccessibilityNodeInfo(info)
        val grid = terminal ?: return
        val cols = grid.width.toInt()
        if (cols == 0) return
        info.text = buildString {
            for ((i, cell) in grid.cells.withIndex()) {
                if (i > 0 && i % cols == 0) append('\n')
                append(cell.displayText())
                if (length >= 16000) break
            }
        }.trimEnd()
        info.isScrollable = true
        info.addAction(AccessibilityNodeInfo.AccessibilityAction.ACTION_SCROLL_FORWARD)
        info.addAction(AccessibilityNodeInfo.AccessibilityAction.ACTION_SCROLL_BACKWARD)
    }

    override fun performAccessibilityAction(action: Int, arguments: Bundle?): Boolean = when (action) {
        AccessibilityNodeInfo.ACTION_SCROLL_FORWARD -> scroll(target, -8, 0f, 0f)
        AccessibilityNodeInfo.ACTION_SCROLL_BACKWARD -> scroll(target, 8, 0f, 0f)
        else -> super.performAccessibilityAction(action, arguments)
    }
}

/** One row's visible text without trailing blanks, for copying. */
internal fun MobileTerminal.rowText(row: Int): String {
    val columns = width.toInt()
    if (columns == 0 || row !in 0 until height.toInt()) return ""
    val start = row * columns
    return (start until minOf(start + columns, cells.size)).joinToString("") { cells[it].displayText() }.trimEnd()
}

internal fun MobileTerminal.screenText(): String =
    (0 until height.toInt()).joinToString("\n") { rowText(it) }.trimEnd()
