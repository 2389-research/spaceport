// ABOUTME: Drives Settings, the toolbar key editor and Host Detail, including refused host edits.
// ABOUTME: Uses a saved host that is never contacted and removes every value it saves.
package ai.spaceport.mobile

import org.junit.After
import org.junit.Assert.*
import org.junit.Before
import org.junit.Test

class PagesTest : LiveMobileUi() {
    private val context get() = instrumentation.targetContext
    private val first = HostProfile("fixture.invalid", 22, "tester", "default")
    private val second = first.copy(session = "next")
    private val elsewhere = HostProfile("elsewhere.invalid", 22, "tester", "default")

    @Before @After fun reset() {
        val store = HostStore(context)
        store.replace(store.profiles().filter { it.hostname == first.hostname || it.hostname == elsewhere.hostname }, emptyList())
        store.pin(first)?.let { store.forgetPin(first, it) }
        store.rename(first.account, "")
        store.rename(elsewhere.account, "")
        check(context.getSharedPreferences("terminal", 0).edit().clear().commit())
        flushFixturePreferences()
    }

    @Test fun tagsSurviveReloadWithoutChangingSessions() {
        val store = HostStore(context)
        store.replace(emptyList(), listOf(first, second, elsewhere))
        store.tag(first.account, "Work, work, home")
        val loaded = HostStore(context)
        assertEquals(listOf("Work", "home"), loaded.tags(first.account))
        assertEquals(setOf(first, second), loaded.filtered("WORK").toSet())
        loaded.tag(first.account, "")
        assertEquals(loaded.profiles(), loaded.filtered("Work"))
    }

    @Test fun settingsSaveTextSizeAndToolbarKeys() {
        val preferences = TerminalPreferences(context)
        launchAtHome().use {
            after("Settings", { click("Settings") }, { label("Toolbar keys") != null })
            after("larger text", { click("Larger text") }, { label("14 sp") != null })
            assertEquals(14, preferences.textSize)
            after("key editor", { click("Toolbar keys") }, { label("Restore default keys") != null })
            after("Control removed", { click("Remove Control") }, { label("Control") == null })
            assertEquals(TerminalKey.ESC, preferences.keys.keys.first())
            after("Escape moved", { click("Move Escape later") }, { preferences.keys.keys.first() == TerminalKey.TAB })
            after("defaults", { click("Restore default keys") }, { label("Control") != null })
            assertEquals(TerminalKeys.standard, preferences.keys)
            after("Settings again", { click("Back") }, { label("Toolbar keys") != null && label("8 keys") != null })
        }
    }

    @Test fun hostDetailRenamesSessionsAndForgetsTheTrustedKey() {
        val store = HostStore(context)
        store.replace(emptyList(), listOf(first, second))
        assertTrue(store.trust(first, "SHA256:fixture"))
        flushFixturePreferences()
        launchAtHome().use {
            after("details", { click("Host details for tester@fixture.invalid") }, {
                label("SHA256:fixture") != null && label("next") != null
            })
            after("forget confirmation", { click("Forget host key…") }, { label("Forget host key") != null })
            after("forgotten", { click("Forget host key") }, { label("No trusted key saved") != null })
            assertNull(store.pin(first))
            after("session menu", { click("next", long = true) }, { label("Rename") != null })
            after("rename", { click("Rename") }, { label("Rename session") != null })
            input("renamed", "default")
            after("renamed", { click("Save") }, { label("renamed") != null })
            assertEquals(setOf(first, first.copy(session = "renamed")), store.profiles().filter { it.hostname == first.hostname }.toSet())
            after("Home", { click("Back") }, { label("Add host") != null && label("renamed") != null })
        }
    }

    @Test fun editingAHostOntoAnotherSavedHostIsRefused() {
        val store = HostStore(context)
        store.replace(emptyList(), listOf(first, elsewhere))
        store.rename(elsewhere.account, "Elsewhere")
        flushFixturePreferences()
        launchAtHome().use {
            after("details", { click("Host details for tester@fixture.invalid") }, { label("Edit host") != null })
            after("edit dialog", { click("Edit host") }, { label("Save") != null })
            input("elsewhere.invalid", "Hostname")
            after("save refused", { click("Save") }, { label("Save") != null })
            // Both hosts keep their own sessions and labels; nothing merged.
            assertEquals(setOf(first, elsewhere),
                store.profiles().filter { it.hostname == first.hostname || it.hostname == elsewhere.hostname }.toSet())
            assertEquals("Elsewhere", store.name(elsewhere.account))
        }
    }
}
