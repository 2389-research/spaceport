// ABOUTME: Verifies agent priority, scoped marks, and search on an Android device.
// ABOUTME: Runs idle classification through the same native library used by the app.
package ai.spaceport.mobile

import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class AgentBoardTest {
    private val host = HostProfile("test.invalid", 22, "tester", "main")

    private fun agent(id: String, status: String, seq: ULong = 1u) =
        AgentRow(
            host,
            "boot",
            "w",
            "t",
            id,
            id,
            "Title $id",
            "Flight",
            "Checklist",
            id,
            status,
            seq,
            null,
            false,
            600000L,
        )

    @Test
    fun homeUsesStatusThenSequenceAndSearchSpansTitleAndLocation() {
        val rows =
            listOf(
                agent("Old", "idle", 1u),
                agent("Runner", "working"),
                agent("New", "idle", 3u),
                agent("Review", "blocked"),
            )
        assertEquals(
            listOf("Review", "Runner", "New", "Old"),
            AgentBoard.sorted(rows, emptyMap(), 700000).map { it.id },
        )
        assertEquals(listOf("Review"), AgentBoard.search(rows, "flight review").map { it.id })
    }

    @Test
    fun unknownIdleTimesUseDaemonSequenceEvenWhenFirstSeenInADifferentOrder() {
        val older = agent("Older", "idle", 1u).copy(firstIdle = 590000u)
        val newer = agent("Newer", "idle", 5u).copy(firstIdle = 580000u)
        assertEquals(
            listOf(newer, older),
            AgentBoard.sorted(listOf(older, newer), emptyMap(), 700000),
        )
    }

    @Test
    fun savedMarksSurviveRestartAndDoNotFollowAReusedPane() {
        val context =
            androidx.test.platform.app.InstrumentationRegistry.getInstrumentation().targetContext
        val catalog = AgentCatalog(context)
        catalog.forget(host)
        try {
            val row = agent("Scout", "idle")
            catalog.observe(host, listOf(row))
            catalog.mark(row, AgentMark(pinned = true))
            val restored = AgentCatalog(context)
            assertEquals(listOf(row), restored.rows(host))
            assertEquals(true, restored.marks[row.key]?.pinned)
            restored.observe(host, listOf(row.copy(boot = "new-boot")))
            assertFalse(restored.marks.containsKey(row.key))
        } finally {
            catalog.forget(host)
        }
    }

    @Test
    fun snoozeWakesForNewWorkOrDeadlineAndRetainsPin() {
        val old = agent("Scout", "idle")
        val mark = AgentMark(true, 900000L, "idle")
        assertEquals(AgentSection.SNOOZED, AgentBoard.section(old, mark, 800000))
        assertEquals(AgentSection.PINNED, AgentBoard.section(old, mark, 900000))
        assertEquals(
            AgentSection.PINNED,
            AgentBoard.section(old.copy(status = "blocked"), mark, 800000),
        )
        assertNotEquals(old.key, old.copy(boot = "replacement").key)
    }
}
