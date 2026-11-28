// ABOUTME: Keeps the terminal surface alive across picker, Find and Compose presentations.
// ABOUTME: Submits input only to the selected session and restores the prior keyboard state.
package ai.spaceport.mobile

import android.app.Activity
import android.graphics.Rect
import android.graphics.Typeface
import android.os.Build
import android.view.Gravity
import android.view.View
import android.view.WindowInsets
import android.view.inputmethod.InputMethodManager
import android.widget.Button
import android.widget.HorizontalScrollView
import android.widget.LinearLayout
import android.widget.Toast
import java.util.concurrent.Executors
import kotlin.math.roundToInt
import uniffi.spaceport_mobile.InputTarget
import uniffi.spaceport_mobile.MobileSession
import uniffi.spaceport_mobile.MobileView

// The session and input route are supplied by the Activity; this view never inflates from XML.
@android.annotation.SuppressLint("ViewConstructor")
internal class TerminalScreen(
    private val activity: Activity,
    private val connection: SessionController,
    private val onGoTo: () -> Unit
) : LinearLayout(activity) {
    private val executor = Executors.newFixedThreadPool(2)
    private val preferences = TerminalPreferences(activity)
    private val surface = TerminalSurface(context, ::route, ::resize, ::claimSize, ::holdMenu, preferences.textSize)
    private val draft = TerminalDraftDialog(activity, connection, executor, ::restoreKeyboard)
    private val history = TerminalHistoryDialog(activity, connection, executor, ::restoreKeyboard)
    fun refreshDraft() = draft.refreshAttachment()
    val isEditing get() = draft.isShowing || history.isShowing
    private var currentView: MobileView? = null
    private var previousSession: MobileSession? = null
    private var previousSize: GridSize? = null
    private var priorIme: Boolean? = null
    private val compose: Button
    private val back: Button
    private val forward: Button
    private var disposed = false

    init {
        orientation = VERTICAL
        addView(surface, LayoutParams(LayoutParams.MATCH_PARENT, 0, 1f))
        val scroller = HorizontalScrollView(context).apply {
            isHorizontalScrollBarEnabled = false
            setBackgroundColor(0xff252527.toInt())
        }
        val bar = LinearLayout(context).apply { gravity = Gravity.CENTER_VERTICAL }
        scroller.addView(bar)
        addView(scroller, LayoutParams(LayoutParams.MATCH_PARENT, LayoutParams.WRAP_CONTENT))
        fun action(title: String, description: String = title, block: () -> Unit): Button {
            return Button(context, null, android.R.attr.borderlessButtonStyle).apply {
                text = title
                contentDescription = description
                isAllCaps = false
                textSize = 13f
                setTextColor(0xffeeeeee.toInt())
                typeface = Typeface.create("sans-serif-medium", Typeface.NORMAL)
                minWidth = dp(44)
                minimumWidth = dp(44)
                setPadding(dp(9), 0, dp(9), 0)
                setOnClickListener { block() }
                bar.addView(this, LayoutParams(LayoutParams.WRAP_CONTENT, dp(48)))
            }
        }
        action("Go To", block = onGoTo)
        back = action("‹", "Previous pane") { travel(false) }
        forward = action("›", "Next pane in history") { travel(true) }
        compose = action("Compose", block = ::showCompose).apply { isEnabled = false }
        action("Find", block = ::showFind)
        action("Keys", "Keyboard") {
            if (keyboardVisible()) imm().hideSoftInputFromWindow(surface.windowToken, 0) else surface.performClick()
        }
        val modifiers = mutableListOf<Pair<Button, Int>>()
        for (key in preferences.keys.keys) {
            when (val act = key.action) {
                is KeyAction.Arm -> modifiers += action(key.keycap, key.title) { surface.toggleModifier(act.flag) } to act.flag
                is KeyAction.Send -> action(key.keycap, key.title) {
                    surface.target?.let { surface.key(it, act.name, act.modifiers) }
                }
            }
        }
        surface.onModifierUsed = {
            for ((button, flag) in modifiers) {
                val on = surface.armed and flag != 0
                button.setTextColor(if (on) 0xff82e3c6.toInt() else 0xffeeeeee.toInt())
                button.isSelected = on
            }
        }
    }

    /** A stationary hold on the terminal offers copying its text or pasting into it. */
    private fun holdMenu(row: Int) {
        val grid = surface.terminal ?: return
        val destination = surface.target ?: return
        val clipboard = context.getSystemService(android.content.ClipboardManager::class.java)
        val pasted = clipboard.primaryClip?.takeIf { it.itemCount > 0 }?.getItemAt(0)?.coerceToText(context)?.toString()
        val choices = buildList {
            add("Copy line" to { copy(clipboard, grid.rowText(row)) })
            add("Copy screen" to { copy(clipboard, grid.screenText()) })
            if (!pasted.isNullOrEmpty()) add("Paste" to {
                if (surface.target == destination) route(destination) { it.sendPaste(destination, pasted, false) }
            })
        }
        android.app.AlertDialog.Builder(activity)
            .setItems(choices.map { it.first }.toTypedArray()) { _, i -> choices[i].second() }
            .show()
    }

    private fun copy(clipboard: android.content.ClipboardManager, text: String) {
        if (text.isEmpty()) return message("Nothing to copy")
        clipboard.setPrimaryClip(android.content.ClipData.newPlainText("Terminal text", text))
        message("Copied")
    }

    private fun dp(value: Int) = (value * resources.displayMetrics.density).roundToInt()
    private fun imm() = context.getSystemService(InputMethodManager::class.java)
    private fun keyboardVisible(): Boolean {
        if (Build.VERSION.SDK_INT >= 30) return rootWindowInsets?.isVisible(WindowInsets.Type.ime()) == true
        val visible = Rect()
        rootView.getWindowVisibleDisplayFrame(visible)
        return rootView.height - visible.bottom > dp(100)
    }

    fun suspendKeyboardForPicker() {
        if (priorIme == null) priorIme = keyboardVisible()
    }

    fun restoreKeyboard() {
        val show = priorIme ?: return
        priorIme = null
        if (disposed) return
        surface.post {
            if (!disposed && surface.isShown) {
                surface.requestFocus()
                if (show) imm().showSoftInput(surface, 0)
                else imm().hideSoftInputFromWindow(surface.windowToken, 0)
            }
        }
    }

    private fun travel(forward: Boolean) {
        val view = currentView ?: return
        val pane = (if (forward) view.forwardPane else view.backPane) ?: return
        connection.travel(view, pane, forward)
    }

    fun render(view: MobileView?) {
        if (disposed) return
        currentView = view
        back.visibility = if (view?.backPane != null) VISIBLE else GONE
        forward.visibility = if (view?.forwardPane != null) VISIBLE else GONE
        val target = view?.inputTarget?.takeUnless { view.disconnected || view.goToPending }
        compose.isEnabled = target != null
        surface.update(view?.terminal, target)
        history.update(target)
        draft.update(target)
        resize()
    }

    private fun resize() {
        val view = currentView ?: return
        if (surface.width <= 0 || surface.height <= 0) return
        val session = connection.sessionFor(view) ?: return
        val size = surface.gridSize
        if (previousSize == size && previousSession === session) return
        try {
            session.resize(view.bootId, size.columns.toUShort(), size.rows.toUShort(),
                size.cellWidth.toUInt(), size.cellHeight.toUInt())
            previousSize = size
            previousSession = session
        } catch (_: uniffi.spaceport_mobile.MobileException) {
            // A new projection retries the size after Go To or reconnect.
        }
    }

    private fun claimSize() {
        val view = currentView ?: return
        val session = connection.sessionFor(view) ?: return
        try { session.claimSize(view.bootId) }
        catch (e: uniffi.spaceport_mobile.MobileException) { message(e.message ?: "Could not claim terminal size") }
    }

    private fun route(target: InputTarget, action: (MobileSession) -> Unit): Boolean {
        val session = connection.sessionFor(target) ?: return false
        return try {
            action(session)
            true
        } catch (e: uniffi.spaceport_mobile.MobileException) {
            message(e.message ?: "Terminal input failed")
            false
        }
    }

    fun showFind() {
        val target = surface.target
        if (target == null || target.popupId != null) { message("Open a pane to search its history"); return }
        suspendKeyboardForPicker()
        history.show(target)
    }

    fun showCompose() {
        val view = connection.view ?: return
        val target = surface.target ?: return
        suspendKeyboardForPicker()
        draft.show(view, target)
    }

    private fun message(text: String) = Toast.makeText(context, text, Toast.LENGTH_SHORT).show()

    fun dispose() {
        disposed = true
        priorIme = null
        history.dispose()
        draft.dispose()
        surface.update(null, null)
        executor.shutdownNow()
    }
}
