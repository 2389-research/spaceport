// ABOUTME: Builds agent and pane destinations from the shared Herdr projection.
// ABOUTME: Sorts saved observations and searches across terminal titles and locations.
package ai.spaceport.mobile

import java.text.Normalizer
import java.util.Locale
import uniffi.spaceport_mobile.IdleAgeGroup
import uniffi.spaceport_mobile.MobilePane
import uniffi.spaceport_mobile.MobileView
import uniffi.spaceport_mobile.idleAgeGroup

internal data class AgentId(
    val host: HostProfile,
    val boot: String,
    val pane: String,
    val name: String?,
)

internal data class AgentRow(
    val host: HostProfile,
    val boot: String,
    val workspace: String,
    val tab: String,
    val id: String,
    val name: String?,
    val title: String?,
    val spaceLabel: String,
    val tabLabel: String,
    val paneLabel: String,
    val status: String,
    val seq: ULong?,
    val firstIdle: ULong?,
    val witnessed: Boolean,
    val seenAt: Long,
) {
    val key
        get() = AgentId(host, boot, id, name)

    val headline
        get() = title?.takeIf { it.isNotBlank() } ?: name ?: paneLabel

    fun paneIn(view: MobileView): MobilePane? {
        if (view.bootId != boot || view.tabs.none { it.id == tab && it.workspaceId == workspace })
            return null
        return view.panes.firstOrNull { it.id == id && it.tabId == tab && it.agentName == name }
    }

    fun hasArrived(view: MobileView, revision: ULong): Boolean =
        view.goToArrival == revision &&
            !view.goToPending &&
            view.bootId == boot &&
            view.focusedWorkspace == workspace &&
            view.focusedTab == tab &&
            view.focusedPane == id &&
            view.inputTarget?.paneId == id &&
            view.inputTarget?.popupId == null &&
            paneIn(view) != null

    val path
        get() =
            listOfNotNull(name?.takeUnless { it == headline }, spaceLabel, tabLabel)
                .joinToString(" › ")
}

internal data class AgentMark(
    val pinned: Boolean = false,
    val until: Long = 0,
    val held: String = "",
) {
    fun current(status: String, now: Long): AgentMark {
        return if (
            until > 0 &&
                (now >= until || (held != status && status in listOf("blocked", "working")))
        )
            copy(until = 0, held = "")
        else this
    }
}

internal enum class AgentSection(val label: String) {
    PINNED("Pinned"),
    WAITING("Waiting for you"),
    WORKING("Working"),
    RECENT("Idle · under 1 hour"),
    HOURS("Idle · 1–5 hours"),
    OLD("Idle · 5+ hours"),
    IDLE("Idle"),
    OTHER("Other agents"),
    SNOOZED("Snoozed"),
}

internal object AgentBoard {
    fun rows(host: HostProfile, view: MobileView, at: Long): List<AgentRow> {
        val tabs = view.tabs.associateBy { it.id }
        val spaces = view.workspaces.associateBy { it.id }
        return view.panes.mapNotNull { pane ->
            val tab = tabs[pane.tabId] ?: return@mapNotNull null
            val space = spaces[tab.workspaceId] ?: return@mapNotNull null
            AgentRow(
                host,
                view.bootId,
                space.id,
                tab.id,
                pane.id,
                pane.agentName,
                pane.terminalTitle,
                space.label,
                tab.label,
                pane.label,
                pane.status,
                pane.stateChangeSeq,
                pane.idleFirstObservedMs,
                pane.idleOnsetObserved,
                at,
            )
        }
    }

    fun section(row: AgentRow, mark: AgentMark?, now: Long): AgentSection {
        val current = mark?.current(row.status, now)
        if ((current?.until ?: 0) > now) return AgentSection.SNOOZED
        if (current?.pinned == true) return AgentSection.PINNED
        return when (row.status) {
            "blocked" -> AgentSection.WAITING
            "working" -> AgentSection.WORKING
            "idle",
            "done" ->
                if (row.firstIdle == null) AgentSection.IDLE
                else
                    when (idleAgeGroup(row.firstIdle, row.witnessed, row.seenAt.toULong())) {
                        IdleAgeGroup.RECENT -> AgentSection.RECENT
                        IdleAgeGroup.ONE_TO_FIVE_HOURS -> AgentSection.HOURS
                        IdleAgeGroup.FIVE_HOURS_OR_MORE -> AgentSection.OLD
                        IdleAgeGroup.UNPLACED -> AgentSection.IDLE
                    }
            else -> AgentSection.OTHER
        }
    }

    fun sorted(rows: List<AgentRow>, marks: Map<AgentId, AgentMark>, now: Long): List<AgentRow> =
        rows.sortedWith(
            compareBy<AgentRow> { section(it, marks[it.key], now) }
                .thenByDescending {
                    if (section(it, marks[it.key], now) == AgentSection.IDLE) it.seenAt
                    else it.firstIdle?.toLong() ?: it.seenAt
                }
                .thenBy { it.host.hostname }
                .thenBy { it.host.port }
                .thenBy { it.host.username }
                .thenBy { it.host.session }
                .thenByDescending { it.seq ?: 0uL }
                .thenBy { it.id }
        )

    private val accents = Regex("\\p{M}+")

    private fun folded(text: String) =
        Normalizer.normalize(text, Normalizer.Form.NFD).replace(accents, "").lowercase(Locale.ROOT)

    fun search(
        rows: List<AgentRow>,
        query: String,
        recent: List<String> = emptyList(),
    ): List<AgentRow> {
        val words = folded(query).split(Regex("\\s+")).filter(String::isNotEmpty)
        if (words.isEmpty()) return rows
        return rows
            .mapNotNull { row ->
                val names = listOfNotNull(row.name, row.title, row.paneLabel).map(::folded)
                val path = folded(row.path)
                var score = 0
                for (word in words) {
                    score +=
                        when {
                            word in names -> 3
                            names.any { it.startsWith(word) } -> 2
                            names.any { it.contains(word) } -> 1
                            word in path -> 0
                            else -> return@mapNotNull null
                        }
                }
                row to score
            }
            .sortedWith(
                compareByDescending<Pair<AgentRow, Int>> { it.second }
                    .thenBy {
                        recent.indexOf(it.first.id).let { index ->
                            if (index < 0) Int.MAX_VALUE else index
                        }
                    }
            )
            .map { it.first }
    }
}
