// ABOUTME: Verifies host onboarding, cached agent order, pinning and a Home jump against real SSH.
// ABOUTME: Uses the disposable seed-agent-order fixture and removes its phone-side observations.
package ai.spaceport.mobile

import android.app.UiAutomation
import android.content.res.Configuration
import android.view.KeyEvent
import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import androidx.test.core.app.ActivityScenario
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.Assert.*
import org.junit.Assume.assumeTrue
import org.junit.Test

class LiveHomeTest : LiveMobileUi() {
    private fun labels(app: ActivityScenario<MainActivity>): List<String> {
        val values = mutableListOf<String>()
        fun collect(view: View) {
            if (view is TextView) values.add(view.text.toString())
            if (view is ViewGroup) for (i in 0 until view.childCount) collect(view.getChildAt(i))
        }
        app.onActivity { collect(it.window.decorView) }
        return values
    }

    @Test fun addsVerifiedHostPinsAgentAndJumpsBackAfterRotation() {
        val args = InstrumentationRegistry.getArguments()
        val session = args.getString("herdrHomeSession")
        assumeTrue("Requires a disposable Home fixture", session != null)
        val profile = HostProfile(requireNotNull(args.getString("herdrHost")),
            args.getString("herdrPort", "22").toInt(), requireNotNull(args.getString("herdrUser")), requireNotNull(session))
        val context = instrumentation.targetContext
        val store = HostStore(context)
        val expected = requireNotNull(args.getString("herdrFingerprint"))
        require(store.pin(profile) == null && profile !in store.profiles()) { "Use a fresh test endpoint and profile" }
        val reviewer = "Review the launch checklist"
        try {
            launchAtHome().use { app ->
                after("host editor", { click("Add host") }, { label("Connect") != null })
                input(profile.hostname, "Hostname")
                input(profile.username, "SSH username")
                input(profile.session, "Herdr session")
                input(profile.port.toString(), "SSH port")
                after("expected fingerprint", { click("Connect") }, {
                    node { it.text?.contains(expected) == true } != null && label("Trust and connect") != null
                })
                after("SSH login", { click("Trust and connect") }, { label("Sign in") != null })
                input(requireNotNull(args.getString("herdrPassword")))
                after("connected terminal", { click("Sign in") }, { label("Go To") != null && label("Terminal") != null })
                after("cached agents", { tapHome() }, { label("Add host") != null && actionable(reviewer, long = true) != null })
                app.moveToState(androidx.lifecycle.Lifecycle.State.CREATED)
                val observed = requireNotNull(AgentCatalog(context).observedAt(profile))
                after("parked roster verified again", { app.moveToState(androidx.lifecycle.Lifecycle.State.RESUMED) }, {
                    val latest = AgentCatalog(context).observedAt(profile)
                    latest != null && latest > observed && actionable(reviewer, long = true) != null
                })
                val home = labels(app)
                val expectedOrder = listOf("Waiting for you", reviewer, "Working", "Scout", "Zephyr", "Atlas", "Workspaces")
                // A remote shell may set a terminal title; the row then shows the agent name in its path.
                val positions = expectedOrder.map { name ->
                    home.indexOfFirst { it == name || it.startsWith("$name ›") }
                }
                assertTrue("Missing Home labels: ${expectedOrder.filterIndexed { index, _ -> positions[index] < 0 }}", positions.none { it < 0 })
                assertEquals("Incorrect agent order", positions.sorted(), positions)
                assertTrue(home.any { it.startsWith("Idle") })
                after("agent actions", { click(reviewer, long = true) }, { label("Pin") != null })
                after("Pinned section", { click("Pin") }, { label("Pinned") != null })
                after("launcher opens a waiting agent", {
                    app.onActivity { it.startActivity(LauncherAction.NEXT.intent(it)) }
                }, {
                    node { it.contentDescription == "Terminal" && it.text?.contains("Review is ready") == true } != null
                })
                after("return from launcher destination", { tapHome() }, { label("Pinned") != null })
                after("jump to Reviewer", { click(reviewer) }, {
                    label(reviewer) != null && node {
                        it.contentDescription == "Terminal" && it.text?.contains("Review is ready") == true
                    } != null
                })
                after("back to Home", { instrumentation.sendKeyDownUpSync(KeyEvent.KEYCODE_BACK) },
                    { label("Pinned") != null })
                if (args.getString("captureHome") == "true") {
                    // Optional QA capture waits for the compositor to finish the Back transition.
                    automation.waitForIdle(500, 5_000)
                    val capture = requireNotNull(automation.takeScreenshot())
                    try {
                        context.cacheDir.resolve("fixture-home.png").outputStream().use {
                            check(capture.compress(android.graphics.Bitmap.CompressFormat.PNG, 100, it))
                        }
                    } finally { capture.recycle() }
                }
                val from = context.resources.configuration.orientation
                val rotation = if (from == Configuration.ORIENTATION_LANDSCAPE)
                    UiAutomation.ROTATION_FREEZE_0 else UiAutomation.ROTATION_FREEZE_90
                after("rotated Home", { assertTrue(automation.setRotation(rotation)) }, {
                    context.resources.configuration.orientation != from && label("Pinned") != null
                })
                assertTrue(labels(app).contains(reviewer))
            }
            launchAtHome().use { app ->
                val cached = labels(app)
                assertTrue(cached.contains("Pinned") && cached.contains(reviewer))
            }
        } finally {
            automation.setRotation(UiAutomation.ROTATION_UNFREEZE)
            AgentCatalog(context).forget(profile)
            store.remove(profile)
            store.forgetPin(profile, expected)
            flushFixturePreferences()
        }
    }
}
