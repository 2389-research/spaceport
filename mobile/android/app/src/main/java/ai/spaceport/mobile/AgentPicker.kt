// ABOUTME: Lets people search or peek without opening another SSH connection.
// ABOUTME: Navigates only through the session and pane captured when a result was shown.
package ai.spaceport.mobile

import android.app.Activity
import android.app.AlertDialog
import android.graphics.Typeface
import android.text.Editable
import android.text.TextWatcher
import android.view.WindowManager
import android.widget.*
import uniffi.spaceport_mobile.MobileView

internal class AgentPicker(
    private val activity: Activity,
    private val connection: SessionController,
    private val host: HostProfile,
    private val previews: PanePreviews,
    private val finished: () -> Unit,
) {
    private val ui = PhoneUi(activity)
    private val body = ui.column()
    private val query =
        EditText(activity).apply {
            hint = "Search agents, titles, workspaces, tabs"
            isSingleLine = true
            inputType =
                android.text.InputType.TYPE_CLASS_TEXT or
                    android.text.InputType.TYPE_TEXT_FLAG_NO_SUGGESTIONS
        }
    private var dialog: AlertDialog? = null
    private var source: MobileView? = null
    private var rows: List<AgentRow> = emptyList()
    private var workspace: String? = null
    private var browse = false
    private var preview: AlertDialog? = null
    private var jump: Pair<AgentRow, ULong>? = null
    private val message = ui.text("", 14, ui.muted)

    fun show(workspaces: Boolean = false) {
        browse = workspaces
        val col =
            ui.column().apply {
                isFocusableInTouchMode = true
                setPadding(ui.dp(16), 0, ui.dp(16), 0)
            }
        col.addView(query)
        col.addView(message.apply { visibility = android.view.View.GONE })
        val modes = ui.line()
        modes.addView(
            ui.button("Agents") {
                browse = false
                workspace = null
                draw()
            }
        )
        modes.addView(
            ui.button("Workspaces") {
                browse = true
                workspace = null
                draw()
            }
        )
        col.addView(modes)
        col.addView(
            ScrollView(activity).apply { addView(body) },
            LinearLayout.LayoutParams(-1, 0, 1f),
        )
        dialog =
            AlertDialog.Builder(activity)
                .setTitle("Go To")
                .setView(col)
                .setNegativeButton("Done", null)
                .create()
        dialog?.setOnDismissListener {
            if (jump != null) connection.cancelGoTo()
            preview?.dismiss()
            finished()
        }
        dialog?.window?.setSoftInputMode(WindowManager.LayoutParams.SOFT_INPUT_STATE_ALWAYS_HIDDEN)
        dialog?.show()
        dialog
            ?.window
            ?.setLayout(-1, (activity.resources.displayMetrics.heightPixels * .83).toInt())
        query.addTextChangedListener(
            object : TextWatcher {
                override fun beforeTextChanged(
                    s: CharSequence?,
                    start: Int,
                    count: Int,
                    after: Int,
                ) {}

                override fun onTextChanged(s: CharSequence?, start: Int, before: Int, count: Int) {
                    draw()
                }

                override fun afterTextChanged(s: Editable?) {}
            }
        )
        connection.view?.let(::update)
    }

    fun dismiss() {
        dialog?.dismiss()
    }

    fun update(view: MobileView) {
        val old = source
        source = view
        if (
            old != null &&
                (old.bootId != view.bootId ||
                    old.inputTarget?.connection != view.inputTarget?.connection)
        )
            preview?.dismiss()
        jump?.let { (row, revision) ->
            if (view.bootId != row.boot || (view.version >= revision && !view.goToPending)) {
                jump = null
                if (row.hasArrived(view, revision)) {
                    dismiss()
                    return
                }
                message.text = view.error ?: "The pane changed before Herdr confirmed focus."
                message.visibility = android.view.View.VISIBLE
            }
        }
        if (
            old != null &&
                old.panes == view.panes &&
                old.tabs == view.tabs &&
                old.workspaces == view.workspaces &&
                old.paneMap == view.paneMap &&
                old.focusedPane == view.focusedPane &&
                old.rosterObservedAtMs == view.rosterObservedAtMs &&
                old.bootId == view.bootId
        )
            return
        rows = AgentBoard.rows(host, view).orEmpty()
        draw()
    }

    private fun jumpTo(row: AgentRow) {
        val live = connection.view
        if (jump == null && live != null && live.bootId == row.boot) {
            val pane = row.paneIn(live)
            val revision = pane?.let { connection.goTo(it, live) }
            if (revision != null) jump = row to revision
            message.text =
                if (revision != null) "Opening " + row.headline
                else "The pane changed. Select it again."
            message.visibility = android.view.View.VISIBLE
        }
    }

    private fun draw() {
        val view = source ?: return
        body.removeAllViews()
        val map = view.paneMap
        if (query.text.isBlank() && map != null) {
            ui.heading(body, "This tab")
            body.addView(
                PaneMapView(activity, view, map) { id ->
                    rows.firstOrNull { it.id == id && it.tab == view.focusedTab }?.let(::jumpTo)
                },
                LinearLayout.LayoutParams(-1, -2),
            )
        }
        fun pane(row: AgentRow) {
            ui.row(
                body,
                row.headline,
                row.path,
                row.status,
                action = { jumpTo(row) },
                held = {
                    val latest = connection.view
                    val excerpt =
                        latest?.takeIf { row.paneIn(it) != null }?.let { previews.peek(it, row.id) }
                    val label =
                        when (excerpt?.first) {
                            "Last seen" ->
                                "Last seen " +
                                    android.text.format.DateUtils.getRelativeTimeSpanString(
                                        excerpt.second.at
                                    )
                            else -> excerpt?.first
                        }
                    val content =
                        ui.column().apply {
                            setPadding(ui.dp(24), ui.dp(8), ui.dp(24), 0)
                            addView(ui.text(row.path, 14, ui.muted))
                            addView(ui.text(label ?: "No screen preview yet", 14, ui.muted))
                            if (excerpt != null)
                                addView(
                                    ui.text(excerpt.second.text, 13).apply {
                                        typeface = Typeface.MONOSPACE
                                    }
                                )
                        }
                    preview =
                        AlertDialog.Builder(activity)
                            .setTitle(row.headline)
                            .setView(content)
                            .setPositiveButton("Done", null)
                            .show()
                },
            )
        }
        if (query.text.isNotBlank()) {
            val matches = AgentBoard.search(rows, query.text.toString(), previews.recent)
            matches.forEach(::pane)
            if (matches.isEmpty()) body.addView(ui.text("No matching panes", 15, ui.muted))
        } else if (browse) {
            val selected = workspace
            if (selected == null)
                view.workspaces.forEach { space ->
                    ui.row(
                        body,
                        space.label,
                        "${rows.count { it.workspace==space.id }} panes",
                        action = {
                            workspace = space.id
                            draw()
                        },
                    )
                }
            else {
                view.tabs
                    .filter { it.workspaceId == selected }
                    .forEach { tab ->
                        ui.heading(body, tab.label)
                        rows.filter { it.tab == tab.id }.forEach(::pane)
                    }
            }
        } else {
            val agents =
                rows
                    .filter { it.name != null }
                    .sortedWith(
                        compareBy<AgentRow> {
                                when (it.status) {
                                    "working" -> 0
                                    "blocked" -> 1
                                    "idle",
                                    "done" -> 2
                                    else -> 3
                                }
                            }
                            .thenByDescending { it.seq ?: 0uL }
                    )
            agents.forEach(::pane)
            if (agents.isEmpty()) body.addView(ui.text("No agents in this session", 15, ui.muted))
        }
    }
}
