// ABOUTME: Keeps a few display-only excerpts for panes visited in the current SSH session.
// ABOUTME: Clears excerpts when the connection, daemon boot, or foreground session ends.
package ai.spaceport.mobile

import uniffi.spaceport_mobile.MobileCell
import uniffi.spaceport_mobile.MobileView

internal data class PaneExcerpt(
    val text: String,
    val at: Long,
    val tab: String,
    val agent: String?,
)

internal class PanePreviews {
    private var previous: MobileView? = null
    private var connection: ULong? = null
    private val excerpts = linkedMapOf<String, PaneExcerpt>()
    val recent = mutableListOf<String>()

    fun clear() {
        previous = null
        connection = null
        excerpts.clear()
        recent.clear()
    }

    fun observe(view: MobileView) {
        val target = view.inputTarget
        if (previous?.bootId != view.bootId || (target != null && connection != target.connection))
            clear()
        val old = previous
        if (
            old != null &&
                (old.inputTarget?.paneId != target?.paneId ||
                    target?.popupId != null ||
                    view.goToPending)
        ) {
            old.focusedPane?.let { pane ->
                excerpt(old, pane)?.let {
                    excerpts.remove(pane)
                    excerpts[pane] = it
                    if (excerpts.size > 12) excerpts.remove(excerpts.keys.first())
                }
            }
        }
        if (target != null) connection = target.connection
        target
            ?.takeIf { it.popupId == null && !view.goToPending }
            ?.paneId
            ?.let { pane ->
                if (recent.firstOrNull() != pane) {
                    recent.remove(pane)
                    recent.add(0, pane)
                    if (recent.size > 12) recent.removeAt(12)
                }
            }
        previous = view
    }

    fun peek(view: MobileView, pane: String): Pair<String, PaneExcerpt>? {
        if (view.bootId != previous?.bootId) return null
        excerpt(view, pane)?.let {
            return "Visible screen" to it
        }
        val current = view.panes.firstOrNull { it.id == pane } ?: return null
        return excerpts[pane]
            ?.takeIf { it.tab == current.tabId && it.agent == current.agentName }
            ?.let { "Last seen" to it }
    }

    private fun excerpt(view: MobileView, pane: String): PaneExcerpt? {
        val target = view.inputTarget ?: return null
        val terminal = view.terminal ?: return null
        val width = terminal.width.toInt()
        if (
            target.popupId != null ||
                target.paneId != pane ||
                target.bootId != view.bootId ||
                view.focusedPane != pane ||
                view.goToPending ||
                width <= 0
        )
            return null
        val current = view.panes.firstOrNull { it.id == pane } ?: return null
        val lines =
            terminal.cells
                .chunked(width)
                .map { line -> line.joinToString("") { it.displayText() }.trimEnd().take(500) }
                .dropLastWhile { it.isBlank() }
                .takeLast(8)
        return if (lines.isEmpty()) null
        else
            PaneExcerpt(
                lines.joinToString("\n"),
                System.currentTimeMillis(),
                current.tabId,
                current.agentName,
            )
    }
}

internal fun MobileCell.displayText(): String =
    when {
        skip -> ""
        modifier.toInt() and 128 != 0 -> " "
        else -> symbol
    }
