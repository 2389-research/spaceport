// ABOUTME: Verifies foreground observation honors explicit disconnect across process recreation.
// ABOUTME: Checks that a later successful login can enable read-only Home observation again.
package ai.spaceport.mobile

import androidx.test.platform.app.InstrumentationRegistry
import java.util.concurrent.CountDownLatch
import java.util.concurrent.Executors
import java.util.concurrent.TimeUnit
import org.junit.Assert.*
import org.junit.Test

class HomeRosterRefreshTest {
    @Test fun disconnectPersistsUntilTheNextLoginToTheSameSession() {
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        val host = HostProfile("refresh.invalid", 22, "fixture", "one")
        val store = HostStore(context)
        try {
            store.allowHomeObservation(host, false)
            val reopened = HostStore(context)
            assertFalse(reopened.permitsHomeObservation(host))
            assertTrue(reopened.permitsHomeObservation(host.copy(session = "two")))
            reopened.allowHomeObservation(host, true)
            assertTrue(store.permitsHomeObservation(host))
        } finally {
            store.allowHomeObservation(host, true)
        }
    }

    @Test fun failedAndCancelledColdChecksKeepTheSavedRosterAndStopTheSpinner() {
        val instrumentation = InstrumentationRegistry.getInstrumentation()
        val context = instrumentation.targetContext
        // The loopback discard port is local to this test device.
        val host = HostProfile("127.0.0.1", 9, "roster-fixture", "offline")
        val store = HostStore(context)
        val catalog = AgentCatalog(context)
        val worker = Executors.newSingleThreadExecutor()
        val finished = CountDownLatch(1)
        val row = AgentRow(host, "boot", "w1", "t1", "p1", "Scout", null,
            "Workspace", "Tab", "Pane", "working", 1u, null, false, 12345)
        org.junit.Assume.assumeTrue("Requires an empty app catalog", store.profiles().isEmpty())
        require(store.pin(host) == null)
        store.save(host)
        assertTrue(store.trust(host, "disposable-test-pin"))
        assertTrue(store.recordEnrollment(host, "disposable-test-pin"))
        catalog.observe(host, listOf(row), row.seenAt)
        lateinit var refresh: HomeRosterRefresh
        try {
            instrumentation.runOnMainSync {
                refresh = HomeRosterRefresh(store, catalog, emptyMap(), worker) {
                    if (host in refresh.failed && refresh.checking == null) finished.countDown()
                }
                refresh.start()
            }
            assertTrue(finished.await(30, TimeUnit.SECONDS))
            instrumentation.runOnMainSync {
                refresh.start()
                assertEquals(host, refresh.checking)
                refresh.cancel()
                assertNull(refresh.checking)
            }
            worker.shutdown()
            assertTrue(worker.awaitTermination(30, TimeUnit.SECONDS))
            instrumentation.waitForIdleSync()
            assertEquals(listOf(row), AgentCatalog(context).rows(host))
            assertEquals(12345L, catalog.observedAt(host))
            assertTrue(refresh.failed.isEmpty())
        } finally {
            instrumentation.runOnMainSync { refresh.cancel() }
            worker.shutdownNow()
            catalog.forget(host)
            store.remove(host)
            store.forgetPin(host, "disposable-test-pin")
        }
    }
}
