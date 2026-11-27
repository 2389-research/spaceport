// ABOUTME: Verifies agent priority, scoped marks, and search on an Android device.
// ABOUTME: Runs idle classification through the same native library used by the app.
package ai.spaceport.mobile

import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith
import uniffi.spaceport_mobile.MobilePane
import uniffi.spaceport_mobile.MobileTab
import uniffi.spaceport_mobile.MobileView
import uniffi.spaceport_mobile.MobileWorkspace

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
    fun persistedIdleAgeUsesOnlyConfirmedRosterObservations() {
        val context =
            androidx.test.platform.app.InstrumentationRegistry.getInstrumentation().targetContext
        val catalog = AgentCatalog(context)
        val view =
            MobileView(
                version = 1u,
                bootId = "boot",
                workspaces = listOf(MobileWorkspace("w", "Work", "idle")),
                tabs = listOf(MobileTab("t", "w", "Tab", "idle")),
                panes = listOf(MobilePane(1u, "Scout", "p", "t", "Pane", "idle", 1_000u, true)),
                paneMap = null,
                focusedWorkspace = "w",
                focusedTab = "t",
                focusedPane = "p",
                terminal = null,
                inputTarget = null,
                error = null,
                disconnected = false,
                hostChallenge = null,
                goToPending = false,
                goToArrival = null,
                rosterObservedAtMs = 3_600_000u,
            )
        catalog.forget(host)
        try {
            catalog.observe(host, AgentBoard.rows(host, view))
            catalog.observe(host, AgentBoard.rows(host, view.copy(rosterObservedAtMs = null)))
            val row = AgentCatalog(context).rows(host).single()
            assertEquals(AgentSection.RECENT, AgentBoard.section(row, null, 7_200_000))
            catalog.observe(host, AgentBoard.rows(host, view.copy(rosterObservedAtMs = 3_601_001u)))
            val refreshed = AgentCatalog(context).rows(host).single()
            assertEquals(AgentSection.HOURS, AgentBoard.section(refreshed, null, 7_200_000))
            catalog.observe(host, AgentBoard.rows(host, view.copy(panes = emptyList())))
            assertTrue(AgentCatalog(context).rows(host).isEmpty())
        } finally {
            catalog.forget(host)
        }
    }

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
