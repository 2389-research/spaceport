// ABOUTME: Exercises the Android terminal toolbar before a writable pane arrives.
// ABOUTME: Checks that Compose is disabled while the button has no destination.
package ai.spaceport.mobile

import android.view.View
import androidx.test.core.app.ActivityScenario
import org.junit.Assert.assertFalse
import org.junit.Test

class TerminalDraftAvailabilityTest {
    @Test fun composeRequiresAWritableDestination() {
        ActivityScenario.launch(MainActivity::class.java).use { app ->
            app.onActivity { activity ->
                val connection = SessionController(activity)
                val screen = TerminalScreen(activity, connection) {}
                try {
                    screen.render(null)
                    val buttons = ArrayList<View>()
                    screen.findViewsWithText(buttons, "Compose", View.FIND_VIEWS_WITH_CONTENT_DESCRIPTION)
                    assertFalse(buttons.single().isEnabled)
                } finally {
                    screen.dispose()
                    connection.close()
                }
            }
        }
    }
}
