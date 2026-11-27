// ABOUTME: Stores bounded agent observations and marks per SSH profile on this phone.
// ABOUTME: Keeps terminal cells, previews, and credentials out of persistent storage.
package ai.spaceport.mobile

import android.content.Context
import androidx.core.content.edit
import org.json.JSONArray

internal class AgentCatalog(context: Context) {
    private val prefs = context.getSharedPreferences("agent-observations", Context.MODE_PRIVATE)
    private val entries = mutableMapOf<HostProfile, List<AgentRow>>()
    val marks = mutableMapOf<AgentId, AgentMark>()

    private fun key(host: HostProfile) =
        JSONArray(listOf(host.hostname, host.port, host.username, host.session)).toString()

    fun rows(host: HostProfile): List<AgentRow> =
        entries.getOrPut(host) {
            val json =
                runCatching { JSONArray(prefs.getString(key(host), "[]")) }
                    .getOrElse { JSONArray() }
            (0 until minOf(json.length(), 544)).mapNotNull { i ->
                runCatching {
                        val r = json.getJSONArray(i)
                        val row =
                            AgentRow(
                                host,
                                r.getString(0),
                                r.getString(1),
                                r.getString(2),
                                r.getString(3),
                                r.getString(4),
                                r.getString(5).ifEmpty { null },
                                r.getString(6),
                                r.getString(7),
                                r.getString(8),
                                r.getString(9),
                                r.getString(10).toULongOrNull(),
                                r.getString(11).toULongOrNull(),
                                r.getBoolean(12),
                                r.getLong(13),
                            )
                        val mark = AgentMark(r.getBoolean(14), r.getLong(15), r.getString(16))
                        if (mark != AgentMark()) marks[row.key] = mark
                        row
                    }
                    .getOrNull()
            }
        }

    fun observe(host: HostProfile, incoming: List<AgentRow>) {
        val previous = rows(host).associateBy { it.key }
        val agents =
            incoming
                .asSequence()
                .filter { it.name != null }
                .filter { row ->
                    listOf(row.boot, row.workspace, row.tab, row.id, row.name!!).all {
                        it.length in 1..256
                    }
                }
                .map { row ->
                    val old = previous[row.key]
                    var next =
                        row.copy(
                            title = row.title?.take(256),
                            spaceLabel = row.spaceLabel.take(256),
                            tabLabel = row.tabLabel.take(256),
                            paneLabel = row.paneLabel.take(256),
                        )
                    if (
                        old != null &&
                            row.status in listOf("idle", "done") &&
                            old.status in listOf("idle", "done") &&
                            row.seq != null &&
                            row.seq != 0uL &&
                            row.seq == old.seq &&
                            old.firstIdle != null &&
                            (!row.witnessed || old.witnessed) &&
                            (row.firstIdle == null || old.firstIdle < row.firstIdle)
                    ) {
                        next = next.copy(firstIdle = old.firstIdle, witnessed = old.witnessed)
                    }
                    next
                }
                .sortedBy {
                    if (it.status == "blocked") 0 else if (it.status == "working") 1 else 2
                }
                .toList()
        val retained =
            (agents.take(32) + agents.filter { it.key in marks }.take(512)).distinctBy { it.key }
        val ids = retained.mapTo(mutableSetOf()) { it.key }
        marks.keys.removeAll { it.host == host && it !in ids }
        for (row in retained) marks[row.key]?.let {
            val next = it.current(row.status, row.seenAt)
            if (next == AgentMark()) marks.remove(row.key) else marks[row.key] = next
        }
        entries[host] = retained
        save(host)
    }

    fun mark(row: AgentRow, value: AgentMark) {
        if (rows(row.host).none { it.key == row.key }) return
        if (value == AgentMark()) marks.remove(row.key) else marks[row.key] = value
        save(row.host)
    }

    fun forget(host: HostProfile) {
        entries.remove(host)
        marks.keys.removeAll { it.host == host }
        prefs.edit { remove(key(host)) }
    }

    private fun save(host: HostProfile) {
        val records = JSONArray()
        for (r in entries[host].orEmpty()) {
            val m = marks[r.key] ?: AgentMark()
            records.put(
                JSONArray(
                    listOf(
                        r.boot,
                        r.workspace,
                        r.tab,
                        r.id,
                        r.name,
                        r.title.orEmpty(),
                        r.spaceLabel,
                        r.tabLabel,
                        r.paneLabel,
                        r.status,
                        r.seq?.toString().orEmpty(),
                        r.firstIdle?.toString().orEmpty(),
                        r.witnessed,
                        r.seenAt,
                        m.pinned,
                        m.until,
                        m.held,
                    )
                )
            )
        }
        prefs.edit { putString(key(host), records.toString()) }
    }
}
