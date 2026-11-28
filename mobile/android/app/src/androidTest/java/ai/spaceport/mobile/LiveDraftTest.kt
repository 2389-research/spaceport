// ABOUTME: Opens two disposable SSH sessions and follows an unsent draft through Home and rotation.
// ABOUTME: Confirms a draft remains private to the session that owned its original pane.
package ai.spaceport.mobile

import androidx.test.platform.app.InstrumentationRegistry
import org.junit.Assert.*
import org.junit.Assume.assumeTrue
import org.junit.Test

class LiveDraftTest : LiveMobileUi() {
    @Test fun draftSurvivesAnotherSessionAndActivityRecreation() {
        val args = InstrumentationRegistry.getArguments()
        val first = args.getString("herdrDraftSession")
        val second = args.getString("herdrOtherDraftSession")
        assumeTrue("Requires two disposable sessions with shell panes", first != null && second != null)
        val context = instrumentation.targetContext
        val store = HostStore(context)
        val a = HostProfile(requireNotNull(args.getString("herdrHost")), args.getString("herdrPort", "22").toInt(),
            requireNotNull(args.getString("herdrUser")), requireNotNull(first))
        val b = a.copy(session = requireNotNull(second))
        val pin = requireNotNull(args.getString("herdrFingerprint"))
        val previous = store.pin(a)
        require(a != b && listOf(a,b).none { it in store.profiles() })
        assertTrue(store.trust(a, pin))
        store.save(a)
        store.save(b)
        try {
            launchAtHome().use { app ->
                fun login(host: HostProfile) {
                    after("login for ${host.session}", { click(host.session) }, { label("Sign in") != null || label("Compose") != null })
                    if (label("Sign in") != null) {
                        input(requireNotNull(args.getString("herdrPassword")))
                        after("terminal", { click("Sign in") }, { label("Compose") != null })
                    }
                }
                fun compose() = after("composer", { click("Compose") }, { node { it.isEditable && it.hintText == "Message" } != null })
                fun home() {
                    after("leave composer", { click("Cancel") }, { label("Home") != null })
                    after("Home", { tapHome() }, { label("Add host") != null })
                }
                login(a)
                compose()
                input("Only session A should see this", "Message")
                home()
                login(b)
                compose()
                assertNull(label("Only session A should see this"))
                home()
                login(a)
                compose()
                assertNotNull(label("Only session A should see this"))
                app.recreate()
                compose()
                assertNotNull(label("Only session A should see this"))
            }
        } finally {
            for (host in listOf(a,b)) {
                AgentCatalog(context).forget(host)
                store.remove(host)
            }
            if (previous == null) store.forgetPin(a, pin)
            flushFixturePreferences()
        }
    }
}
