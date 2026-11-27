// ABOUTME: Searches retained Herdr pane history and highlights the current match.
// ABOUTME: Cancels pending reads when the pane, query or presentation changes.
package ai.spaceport.mobile

import android.app.Activity
import android.app.AlertDialog
import android.content.ClipData
import android.content.ClipboardManager
import android.graphics.Typeface
import android.text.Editable
import android.text.SpannableStringBuilder
import android.text.Spanned
import android.text.TextWatcher
import android.text.style.StyleSpan
import android.view.inputmethod.EditorInfo
import android.widget.Button
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import java.util.concurrent.ExecutorService
import uniffi.spaceport_mobile.HistoryMatch
import uniffi.spaceport_mobile.HistorySearch
import uniffi.spaceport_mobile.InputTarget

internal class TerminalHistoryDialog(
    private val activity: Activity,
    private val connection: SessionController,
    private val executor: ExecutorService,
    private val onDismiss: () -> Unit
) {
    private var dialog: AlertDialog? = null
    val isShowing get() = dialog?.isShowing == true
    private var original: InputTarget? = null
    private var search: HistorySearch? = null

    private fun cancel() {
        search?.let { it.cancel(); it.close() }
        search = null
    }

    fun show(target: InputTarget) {
        if (dialog != null) return
        original = target
        val form = LinearLayout(activity).apply {
            orientation = LinearLayout.VERTICAL
            val margin = (16 * resources.displayMetrics.density).toInt()
            setPadding(margin, 0, margin, 0)
        }
        val query = EditText(activity).apply {
            hint = "Search this pane"
            setSingleLine()
            imeOptions = EditorInfo.IME_ACTION_SEARCH
            isSaveEnabled = false
        }
        val detail = TextView(activity).apply { typeface = Typeface.MONOSPACE }
        val count = TextView(activity)
        val buttons = LinearLayout(activity)
        fun button(text: String): Button = Button(activity, null, android.R.attr.borderlessButtonStyle).apply {
            this.text = text
            isAllCaps = false
            buttons.addView(this, LinearLayout.LayoutParams(0, -2, 1f))
        }
        val older = button("Older")
        val newer = button("Newer")
        val copy = button("Copy match")
        form.addView(query)
        form.addView(buttons)
        form.addView(count)
        form.addView(detail)
        val sheet = AlertDialog.Builder(activity).setTitle("Find in terminal")
            .setView(ScrollView(activity).apply { addView(form) })
            .setPositiveButton("Find", null).setNegativeButton("Done", null).create()
        var match: HistoryMatch? = null
        fun showMatch(result: HistoryMatch?) {
            match = result
            older.isEnabled = result != null && result.total > 1uL
            newer.isEnabled = older.isEnabled
            copy.isEnabled = result != null
            detail.text = result?.let {
                SpannableStringBuilder(it.before).apply {
                    val start = length
                    append(it.matched)
                    setSpan(StyleSpan(Typeface.BOLD), start, length, Spanned.SPAN_EXCLUSIVE_EXCLUSIVE)
                    append(it.after)
                }
            }
            count.text = result?.let { activity.getString(R.string.terminal_history_count, it.index.toString(), it.total.toString()) }.orEmpty()
        }
        fun find(forward: Boolean, step: Boolean) {
            val text = query.text.toString()
            val session = connection.sessionFor(target) ?: return
            val previous = if (step) match?.cursor else null
            cancel()
            showMatch(null)
            if (text.isEmpty()) return
            val request = try { session.historySearch(target) } catch (e: Exception) {
                count.text = e.message ?: "History unavailable"
                return
            }
            count.setText(R.string.terminal_searching)
            search = request
            executor.execute {
                val outcome = runCatching { request.find(text, forward, previous) }
                activity.runOnUiThread {
                    if (dialog !== sheet || search !== request) return@runOnUiThread
                    search = null
                    request.close()
                    if (connection.sessionFor(target) !== session) return@runOnUiThread
                    outcome.fold({ result ->
                        showMatch(result)
                        if (result == null) count.setText(R.string.terminal_no_matches)
                    }, { count.text = it.message ?: "Search failed" })
                }
            }
        }
        query.addTextChangedListener(object : TextWatcher {
            override fun afterTextChanged(s: Editable?) { cancel(); showMatch(null) }
            override fun beforeTextChanged(s: CharSequence?, start: Int, count: Int, after: Int) = Unit
            override fun onTextChanged(s: CharSequence?, start: Int, before: Int, count: Int) = Unit
        })
        query.setOnEditorActionListener { _, id, _ ->
            if (id == EditorInfo.IME_ACTION_SEARCH) { find(false, false); true } else false
        }
        older.setOnClickListener { find(false, true) }
        newer.setOnClickListener { find(true, true) }
        copy.setOnClickListener {
            match?.let {
                activity.getSystemService(ClipboardManager::class.java)
                    .setPrimaryClip(ClipData.newPlainText("Terminal match", it.matched))
            }
        }
        showMatch(null)
        dialog = sheet
        sheet.setOnDismissListener {
            dialog = null
            original = null
            cancel()
            onDismiss()
        }
        sheet.show()
        sheet.getButton(AlertDialog.BUTTON_POSITIVE).setOnClickListener { find(false, false) }
    }

    fun update(target: InputTarget?) { if (target != original) dialog?.dismiss() }
    fun dispose() { dialog?.dismiss(); cancel() }
}
