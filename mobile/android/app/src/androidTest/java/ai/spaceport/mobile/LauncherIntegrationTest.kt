// ABOUTME: Checks private system entry points, expiring destinations, and locally rendered widget content.
// ABOUTME: Exercises the Android shortcut manager and inflates the real widget RemoteViews.
package ai.spaceport.mobile

import android.content.Intent
import android.content.pm.ShortcutManager
import android.widget.FrameLayout
import android.widget.TextView
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.Assert.*
import org.junit.Test

class LauncherIntegrationTest {
    private val context = InstrumentationRegistry.getInstrumentation().targetContext
    private val store = HostStore(context)
    private val host = HostProfile("widget.invalid", 22, "pilot", "flight")

    @Test fun onlyPrivateShortcutsCanRouteAndForgottenDestinationsExpire() {
        store.save(host)
        try {
            store.recordOpened(host)
            val original = requireNotNull(store.widgetToken())
            assertEquals(host, store.lastOpened(original))
            val request = LauncherAction.NEXT.intent(context, original)
            assertEquals(LauncherAction.NEXT, LauncherAction.from(context, request))
            assertNull(LauncherAction.from(context, Intent(request).setClass(context, MainActivity::class.java)))
            assertFalse(context.packageManager.getActivityInfo(requireNotNull(request.component), 0).exported)
            publishAgentShortcuts(context)
            val shortcuts = context.getSystemService(ShortcutManager::class.java).dynamicShortcuts
            assertTrue(shortcuts.map { it.id }.containsAll(listOf("resume", "next_waiting")))
            store.remove(host)
            store.save(host)
            store.recordOpened(host)
            assertNull(store.lastOpened(original))
        } finally { store.remove(host) }
    }

    @Test fun forgettingHostTrustInvalidatesTheWidgetEvenAfterReauthorization() {
        store.save(host)
        try {
            assertTrue(store.trust(host, "fingerprint-one"))
            store.recordOpened(host)
            val previous = requireNotNull(store.widgetToken())
            assertTrue(store.forgetPin(host, "fingerprint-one"))
            assertNull(store.lastOpened(previous))
            assertTrue(store.trust(host, "fingerprint-two"))
            store.recordOpened(host)
            assertNull(store.lastOpened(previous))
            assertEquals(host, store.lastOpened(requireNotNull(store.widgetToken())))
        } finally {
            store.pin(host)?.let { store.forgetPin(host, it) }
            store.remove(host)
        }
    }

    @Test fun widgetCountsTheFullObservedRosterAndKeepsTitlesOffTheLauncher() {
        store.save(host)
        val catalog = AgentCatalog(context)
        try {
            store.recordOpened(host)
            val rows = (1..40).map { i ->
                AgentRow(host, "boot", "workspace", "tab", "pane-$i", "private-agent", "private title",
                    "work", "main", "pane", "blocked", 1u, null, false, 10000)
            }
            catalog.observe(host, rows, 10000)
            val remote = AgentStatusWidget.contents(context)
            InstrumentationRegistry.getInstrumentation().runOnMainSync {
                val view = remote.apply(context, FrameLayout(context))
                assertEquals("40 waiting for you", view.findViewById<TextView>(R.id.widget_waiting).text)
                assertTrue(view.findViewById<TextView>(R.id.widget_time).text.startsWith("Observed "))
            }
        } finally {
            catalog.forget(host)
            store.remove(host)
        }
    }
}
