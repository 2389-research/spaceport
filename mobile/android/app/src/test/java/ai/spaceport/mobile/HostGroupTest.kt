// ABOUTME: Tests how saved sessions gather under the SSH account they share.
// ABOUTME: Covers case-insensitive hosts, separate ports and accounts, and renamed addresses.
package ai.spaceport.mobile

import org.junit.Assert.*
import org.junit.Test

class HostGroupTest {
    private val build = HostProfile("build.example.org", 22, "deploy", "default")

    @Test fun sessionsOnOneAccountShareAGroup() {
        val groups = HostGroup.of(
            listOf(build, build.copy(hostname = "Build.Example.org", session = "next"),
                build.copy(port = 2222), build.copy(username = "root"))
        )
        assertEquals(3, groups.size)
        val shared = groups.single { it.sessions.size == 2 }
        assertEquals(listOf("default", "next"), shared.sessions.map { it.session })
        assertEquals("deploy@build.example.org", shared.destination)
    }

    @Test fun movingAGroupKeepsEverySessionName() {
        val group = HostGroup.of(listOf(build, build.copy(session = "next"))).single()
        val moved = group.movedTo("10.0.0.5", 2200, "ops")
        assertEquals(listOf("default", "next"), moved.map { it.session })
        assertTrue(moved.all { it.hostname == "10.0.0.5" && it.port == 2200 && it.username == "ops" })
    }
}
