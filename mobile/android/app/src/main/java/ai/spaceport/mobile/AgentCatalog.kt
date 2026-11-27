// ABOUTME: Stores bounded agent observations and marks per SSH profile on this phone.
// ABOUTME: Keeps terminal cells, previews, and credentials out of persistent storage.
package ai.spaceport.mobile

import android.content.Context
import androidx.core.content.edit
import org.json.JSONArray
import uniffi.spaceport_mobile.MobileView

internal class AgentCatalog(context: Context) {
    private val app = context.applicationContext
    private val prefs = context.getSharedPreferences("agent-observations", Context.MODE_PRIVATE)
    private val entries = mutableMapOf<HostProfile, List<AgentRow>>()
    val marks = mutableMapOf<AgentId, AgentMark>()

    private fun key(host: HostProfile) =
        JSONArray(listOf(host.hostname, host.port, host.username, host.session)).toString()

    fun observedAt(host: HostProfile): Long? =
        prefs.getLong("observed/${key(host)}", 0).takeIf { it > 0 }

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
                                observedAt(host),
                                r.optString(17).ifEmpty { null },
                            )
                        val mark = AgentMark(r.getBoolean(14), r.getLong(15), r.getString(16), r.optBoolean(18))
                        if (mark != AgentMark()) marks[row.key] = mark
                        row
                    }
                    .getOrNull()
            }
        }

    fun counts(host: HostProfile): AgentCounts? {
        val raw = prefs.getString("counts/${key(host)}", null) ?: return null
        return try {
            val counts = JSONArray(raw)
            AgentCounts(counts.getInt(0), counts.getInt(1), counts.getInt(2))
        } catch (_: org.json.JSONException) {
            null
        }
    }

    fun observe(host: HostProfile, view: MobileView) {
        val at = view.observedAt ?: return
        observe(host, AgentBoard.rows(host, view), at)
    }

    fun observe(host: HostProfile, incoming: List<AgentRow>, at: Long?) {
        val previousObservation = observedAt(host)
        if (at == null || (previousObservation != null && at < previousObservation)) return
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
                            seenAt = at,
                            identity = row.identity?.takeIf { it.matches(brandIdentity) },
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
            var next = it.current(row.status, at ?: 0L)
            if (next.settled && previous[row.key]?.seq != row.seq) next = next.copy(settled = false)
            if (next == AgentMark()) marks.remove(row.key) else marks[row.key] = next
        }
        entries[host] = retained
        save(host, at, AgentCounts.of(agents))
        AgentStatusWidget.update(app)
    }

    fun mark(row: AgentRow, value: AgentMark) {
        val current = rows(row.host).firstOrNull { it.key == row.key } ?: return
        if (value.settled && (current.status != "done" || current.seq != row.seq)) return
        if (value == AgentMark()) marks.remove(row.key) else marks[row.key] = value
        save(row.host, observedAt(row.host))
    }

    fun forget(host: HostProfile) {
        entries.remove(host)
        marks.keys.removeAll { it.host == host }
        prefs.edit { remove(key(host)); remove("observed/${key(host)}"); remove("counts/${key(host)}") }
        AgentStatusWidget.update(app)
    }

    private fun save(host: HostProfile, at: Long?, counts: AgentCounts? = null) {
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
                        r.identity.orEmpty(),
                        m.settled,
                    )
                )
            )
        }
        prefs.edit {
            putString(key(host), records.toString())
            if (counts != null) putString("counts/${key(host)}",
                JSONArray(listOf(counts.waiting, counts.working, counts.idle)).toString())
            if (at != null) putLong("observed/${key(host)}", at)
        }
    }
}
