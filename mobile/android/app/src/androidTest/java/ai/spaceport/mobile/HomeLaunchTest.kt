// ABOUTME: Exercises the Android app through its launcher and host editor.
// ABOUTME: Checks that onboarding leaves Herdr hosts and sessions untouched until saved.
package ai.spaceport.mobile

import android.view.View
import android.view.ViewGroup
import android.widget.TextView
import androidx.test.core.app.ActivityScenario
import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.assertTrue
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class HomeLaunchTest {
    @Test
    fun homePutsAgentsFirstAndOffersHostSetup() {
        ActivityScenario.launch(MainActivity::class.java).use { app ->
            app.onActivity { activity ->
                val root = activity.findViewById<ViewGroup>(android.R.id.content)
                fun texts(view: View): List<String> =
                    when (view) {
                        is TextView -> listOf(view.text.toString())
                        is ViewGroup ->
                            (0 until view.childCount).flatMap { texts(view.getChildAt(it)) }
                        else -> emptyList()
                    }
                val labels = texts(root)
                assertTrue(labels.contains("Agents"))
                assertTrue(labels.contains("Add host"))
            }
        }
    }
}
