// ABOUTME: Drives Go To pane actions and the terminal trail through Android controls.
// ABOUTME: Requires a fresh SSH fixture with one pane labeled Pane Tools Origin.
package ai.spaceport.mobile

import androidx.test.platform.app.InstrumentationRegistry
import org.junit.Assert.*
import org.junit.Assume.assumeTrue
import org.junit.Test

class LivePaneActionsTest : LiveMobileUi() {
    @Test fun splitAndReturnToPreviousPane() {
        val args = InstrumentationRegistry.getArguments()
        val session = args.getString("herdrPaneActionsSession")
        assumeTrue("Requires a dedicated pane actions fixture", session != null)
        val profile = HostProfile(requireNotNull(args.getString("herdrHost")),
            args.getString("herdrPort", "22").toInt(), requireNotNull(args.getString("herdrUser")), requireNotNull(session))
        val store = HostStore(instrumentation.targetContext)
        val pin = requireNotNull(args.getString("herdrFingerprint"))
        val existingPin = store.pin(profile)
        require(profile !in store.profiles()) { "Use a fresh test profile" }
        assertTrue(store.trust(profile, pin))
        store.save(profile)
        try {
            launchAtHome().use {
                after("SSH password", { click(session) }, { label("Sign in") != null })
                input(requireNotNull(args.getString("herdrPassword")))
                after("terminal", { click("Sign in") }, { label("Go To") != null })
                after("Go To search", { click("Go To") }, { node { it.isEditable } != null })
                after("origin pane", { input("Tools Orig") }, { actionable("Pane Tools Origin", true) != null })
                after("pane actions", { click("Pane Tools Origin", true) }, { label("Split right") != null })
                assertNotNull(label("Split down"))
                assertNotNull(label("Zoom pane"))
                assertNotNull(label("Rename pane"))
                after("picker after split", { click("Split right") }, { label("Split right") == null })
                after("terminal history", { click("Done") }, { label("Previous pane") != null })
                after("Back confirmed", { click("Previous pane") }, { label("Next pane in history") != null })
                after("Forward confirmed", { click("Next pane in history") }, {
                    label("Previous pane") != null && label("Next pane in history") == null
                })
            }
        } finally {
            AgentCatalog(instrumentation.targetContext).forget(profile)
            store.remove(profile)
            if (existingPin == null) store.forgetPin(profile, pin)
            flushFixturePreferences()
        }
    }
}
