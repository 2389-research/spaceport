// ABOUTME: Offers addressed pane actions inside Go To without selecting the pane.
// ABOUTME: Keeps rename and explicit zoom tied to the row the user opened.
package ai.spaceport.mobile

import android.app.Activity
import android.app.AlertDialog
import android.widget.EditText
import android.widget.LinearLayout
import uniffi.spaceport_mobile.MobilePaneAction
import uniffi.spaceport_mobile.MobileView

internal class PaneActionMenu(
    private val activity: Activity,
    private val apply: (MobilePaneAction) -> Unit,
) {
    fun addTo(content: LinearLayout, view: MobileView, row: AgentRow) {
        val offers = view.paneActions ?: return
        val tab = view.tabs.firstOrNull { it.id == row.tab && it.workspaceId == row.workspace } ?: return
        val ui = PhoneUi(activity)
        if (offers.split) {
            content.addView(ui.button("Split right") { apply(MobilePaneAction.SplitRight) })
            content.addView(ui.button("Split down") { apply(MobilePaneAction.SplitDown) })
        }
        if (offers.zoom) {
            content.addView(ui.button(if (tab.zoomed) "Restore panes" else "Zoom pane") {
                apply(MobilePaneAction.Zoom(!tab.zoomed))
            })
        }
        if (offers.rename) {
            content.addView(ui.button("Rename pane") {
                val field = EditText(activity).apply {
                    isSingleLine = true
                    setText(row.paneLabel)
                    selectAll()
                }
                AlertDialog.Builder(activity)
                    .setTitle("Rename pane")
                    .setView(field)
                    .setNegativeButton("Cancel", null)
                    .setPositiveButton("Rename") { _, _ -> apply(MobilePaneAction.Rename(field.text.toString())) }
                    .show()
            })
        }
    }
}
