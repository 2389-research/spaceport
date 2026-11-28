// ABOUTME: Drives Go To, native IME input, Compose and Find against a disposable Herdr session.
// ABOUTME: Requires a fresh seed-agent-order session because replying completes its Reviewer prompt.
package ai.spaceport.mobile

import android.view.View
import android.view.ViewGroup
import android.view.inputmethod.EditorInfo
import androidx.test.core.app.ActivityScenario
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.Assert.*
import org.junit.Assume.assumeTrue
import org.junit.Test

class LiveTerminalTest : LiveMobileUi() {
    private fun surface(root: View): TerminalSurface? {
        if (root is TerminalSurface) return root
        if (root is ViewGroup) for (i in 0 until root.childCount) surface(root.getChildAt(i))?.let { return it }
        return null
    }

    @Test fun switchesPanesAndSendsTheComposedReplyToTheReviewer() {
        val args = InstrumentationRegistry.getArguments()
        val session = args.getString("herdrTerminalSession")
        assumeTrue("Requires a fresh disposable terminal fixture", session != null)
        val profile = HostProfile(requireNotNull(args.getString("herdrHost")),
            args.getString("herdrPort", "22").toInt(), requireNotNull(args.getString("herdrUser")), requireNotNull(session))
        val store = HostStore(instrumentation.targetContext)
        val pin = requireNotNull(args.getString("herdrFingerprint"))
        val existingPin = store.pin(profile)
        require(profile !in store.profiles()) { "Use a fresh test profile" }
        assertTrue(store.trust(profile, pin))
        store.save(profile)
        try {
            launchAtHome().use { app ->
                after("SSH password", { click(session) }, { label("Sign in") != null })
                input(requireNotNull(args.getString("herdrPassword")))
                after("terminal", { click("Sign in") }, { label("Go To") != null })
                fun selectedPane(): String? {
                    var selected: String? = null
                    app.onActivity { selected = surface(it.window.decorView)?.target?.paneId }
                    return selected
                }
                val initialPane = selectedPane()
                fun scout() = node { it.text == "Scout" || it.text?.startsWith("Scout ›") == true }
                after("Go To agent rows", { click("Go To") }, { scout() != null })
                after("Scout terminal", { click(requireNotNull(scout()?.text).toString()) }, {
                    label("Terminal") != null && selectedPane()?.let { it != initialPane } == true
                })
                after("Reviewer in picker", { click("Go To") }, { label("Review the launch checklist") != null })
                after("Reviewer terminal", { click("Review the launch checklist") }, {
                    node { it.contentDescription == "Terminal" && it.text?.contains("Review is ready") == true } != null
                })
                app.onActivity {
                    val terminal = requireNotNull(surface(it.window.decorView))
                    val ime = requireNotNull(terminal.onCreateInputConnection(EditorInfo()))
                    assertTrue(ime.setComposingText("wron", 1))
                    assertTrue(ime.commitText("android-", 1))
                    ime.closeConnection()
                }
                after("draft editor", { click("Compose") }, { label("Press Return after pasting") != null })
                input("flight")
                after("remote acknowledgement", { click("Send") }, {
                    node { it.contentDescription == "Terminal" && it.text?.contains("accepted-android-flight") == true } != null
                })
                after("history editor", { click("Find") }, { label("Find in terminal") != null })
                input("accepted-android-flight")
                after("history match", { click("Find") }, { node { it.text?.startsWith("Match 1 of ") == true } != null })
                assertNotNull(label("Copy match"))
                after("terminal after Find", { instrumentation.sendKeyDownUpSync(android.view.KeyEvent.KEYCODE_BACK) }, { label("New terminal") != null })
                val before = requireNotNull(selectedPane())
                after("new terminal", { click("New terminal") }, { selectedPane()?.let { it != before } == true })
            }
        } finally {
            AgentCatalog(instrumentation.targetContext).forget(profile)
            store.remove(profile)
            if (existingPin == null) store.forgetPin(profile, pin)
            flushFixturePreferences()
        }
    }
}
