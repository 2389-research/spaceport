// ABOUTME: Opens an explicitly chosen HTTP service in the system browser over real SSH.
// ABOUTME: Exercises another page load while parked and stops the preview after returning.
package ai.spaceport.mobile

import android.content.Intent
import android.graphics.Rect
import android.os.SystemClock
import android.view.InputDevice
import android.view.MotionEvent
import androidx.test.platform.app.InstrumentationRegistry
import org.junit.Assert.*
import org.junit.Assume.assumeTrue
import org.junit.Test

class ServicePreviewTest : LiveMobileUi() {
    private fun tapBrowserLink(text: String) {
        val bounds = Rect()
        checkNotNull(label(text)).getBoundsInScreen(bounds)
        check(!bounds.isEmpty)
        val down = SystemClock.uptimeMillis()
        for (action in listOf(MotionEvent.ACTION_DOWN, MotionEvent.ACTION_UP)) {
            val touch = MotionEvent.obtain(down, SystemClock.uptimeMillis(), action,
                bounds.exactCenterX(), bounds.exactCenterY(), 0)
            try {
                touch.source = InputDevice.SOURCE_TOUCHSCREEN
                check(automation.injectInputEvent(touch, true))
            } finally {
                touch.recycle()
            }
        }
    }

    @Test fun browseAndStop() {
        val args = InstrumentationRegistry.getArguments()
        val remote = args.getString("herdrServicePort")
        assumeTrue("Requires the loopback trial fixture", remote != null)
        val context = instrumentation.targetContext
        val profile = HostProfile(requireNotNull(args.getString("herdrHost")),
            args.getString("herdrPort", "22").toInt(), requireNotNull(args.getString("herdrUser")),
            requireNotNull(args.getString("herdrServiceSession")))
        val store = HostStore(context)
        val fingerprint = requireNotNull(args.getString("herdrFingerprint"))
        val existingPin = store.pin(profile)
        require(profile !in store.profiles())
        assertTrue(store.trust(profile, fingerprint))
        store.save(profile)
        try {
            launchAtHome().use {
                after("SSH password", { click(profile.session) }, { label("Sign in") != null })
                input(requireNotNull(args.getString("herdrPassword")))
                after("terminal", { click("Sign in") }, { label("Go To") != null })
                after("session menu", { click("Session") }, { label("Preview loopback service") != null })
                click("Preview loopback service")
                input(requireNotNull(remote), "Remote port")
                after("browser at the user's chosen loopback service", { click("Open") }, { label("Check again") != null })
                after("fresh response from the SSH host", { tapBrowserLink("Check again") }, { label("Round trip confirmed") != null })
                after("foreground original terminal", {
                    context.startActivity(Intent(context, MainActivity::class.java).addFlags(
                        Intent.FLAG_ACTIVITY_NEW_TASK or Intent.FLAG_ACTIVITY_REORDER_TO_FRONT))
                }, { label("Session") != null })
                after("active preview", { click("Session") }, { label("Stop service preview") != null })
                click("Stop service preview")
                after("stopped preview", { click("Session") }, { label("Preview loopback service") != null })
            }
        } finally {
            AgentCatalog(context).forget(profile)
            store.remove(profile)
            if (existingPin == null) store.forgetPin(profile, fingerprint)
            flushFixturePreferences()
        }
    }
}
