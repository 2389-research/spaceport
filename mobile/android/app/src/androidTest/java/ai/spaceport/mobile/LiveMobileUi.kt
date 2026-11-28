// ABOUTME: Drives native controls and observes accessibility events in live SSH fixture tests.
// ABOUTME: Bounds each wait by its failure timeout and never logs supplied credentials.
package ai.spaceport.mobile

import android.os.Bundle
import android.view.accessibility.AccessibilityNodeInfo
import androidx.test.core.app.ActivityScenario
import androidx.test.platform.app.InstrumentationRegistry
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import org.junit.Assert.assertTrue

abstract class LiveMobileUi {
    protected val instrumentation = InstrumentationRegistry.getInstrumentation()
    protected val automation get() = instrumentation.uiAutomation

    // The instrumentation runner can terminate the app before asynchronous fixture removals finish.
    @Suppress("ApplySharedPref")
    protected fun flushFixturePreferences() {
        for (name in listOf("spaceport.hosts", "agent-observations")) {
            check(instrumentation.targetContext.getSharedPreferences(name, 0).edit().commit())
        }
    }

    protected fun launchAtHome(): ActivityScenario<MainActivity> {
        lateinit var app: ActivityScenario<MainActivity>
        after("Home launch", { app = ActivityScenario.launch(MainActivity::class.java) },
            { label("Add host") != null })
        return app
    }

    protected fun node(predicate: (AccessibilityNodeInfo) -> Boolean): AccessibilityNodeInfo? {
        fun visit(item: AccessibilityNodeInfo): AccessibilityNodeInfo? {
            if (predicate(item)) return item
            for (i in 0 until item.childCount) item.getChild(i)?.let { visit(it)?.let { found -> return found } }
            return null
        }
        return automation.rootInActiveWindow?.let(::visit)
    }

    protected fun label(value: String) = node { it.text?.toString().equals(value, true) || it.contentDescription == value }

    protected fun actionable(value: String, long: Boolean = false): AccessibilityNodeInfo? {
        var result: AccessibilityNodeInfo? = null
        node { item ->
            if (item.text?.toString().equals(value, true) || item.contentDescription == value) {
                var ancestor: AccessibilityNodeInfo? = item
                while (ancestor != null && ancestor.isVisibleToUser) {
                    val accepts = if (long) ancestor.isLongClickable else ancestor.isClickable
                    if (accepts && ancestor.isEnabled && ancestor.refresh()) {
                        result = ancestor
                        break
                    }
                    ancestor = ancestor.parent
                }
            }
            result != null
        }
        return result
    }

    protected fun click(value: String, long: Boolean = false) {
        fun find() = actionable(value, long)
        var control = find()
        if (control == null) {
            val ready = CountDownLatch(1)
            automation.setOnAccessibilityEventListener { if (find() != null) ready.countDown() }
            try {
                control = find()
                if (control == null) {
                    assertTrue("No actionable control named $value", ready.await(10, TimeUnit.SECONDS))
                    control = find()
                }
            } finally { automation.setOnAccessibilityEventListener(null) }
        }
        val action = if (long) AccessibilityNodeInfo.ACTION_LONG_CLICK else AccessibilityNodeInfo.ACTION_CLICK
        check(requireNotNull(control) { "The control $value disappeared" }.performAction(action)) {
            "Click failed for $value"
        }
    }

    protected fun tapHome() {
        val announcement = node { it.contentDescription?.startsWith("Home,") == true }?.contentDescription
        click(announcement?.toString() ?: "Home")
    }

    protected fun input(value: String, hint: String? = null) {
        val editor = requireNotNull(node { it.isEditable && (hint == null || it.hintText?.toString() == hint) }) { "No active editor" }
        check(editor.performAction(AccessibilityNodeInfo.ACTION_SET_TEXT, Bundle().apply {
            putCharSequence(AccessibilityNodeInfo.ACTION_ARGUMENT_SET_TEXT_CHARSEQUENCE, value)
        }))
    }

    protected fun after(description: String, action: () -> Unit, reached: () -> Boolean) {
        try {
            automation.executeAndWaitForEvent(action, { reached() }, 30_000)
        } catch (e: java.util.concurrent.TimeoutException) {
            assertTrue("Timeout waiting for $description", reached())
        }
    }

}
