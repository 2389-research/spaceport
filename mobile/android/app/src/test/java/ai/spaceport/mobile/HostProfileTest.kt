// ABOUTME: Exercises which SSH targets share host trust and account authorization.
// ABOUTME: Covers invalid route fields before a connection can open.
package ai.spaceport.mobile

import org.junit.Assert.*
import org.junit.Test

class HostProfileTest {
    @Test fun trustBelongsToHostAndPortWhileEnrollmentBelongsToAccount() {
        val host = HostProfile("Build.Example.org", 22, "deploy", "default")
        val second = host.copy(hostname = "build.example.org", session = "next")
        assertEquals(host.endpoint, second.endpoint)
        assertEquals(host.account, second.account)
        assertNotEquals(host.endpoint, host.copy(port = 2222).endpoint)
        assertNotEquals(host.account, host.copy(username = "root").account)
        assertEquals(host.endpoint, host.copy(username = "root").endpoint)
    }

    @Test fun refusesUnroutableNamesAndOutOfRangePorts() {
        val valid = HostProfile("::1", 22, "deploy", "main")
        assertNull(valid.validationError())
        for (profile in listOf(valid.copy(port = 65536), valid.copy(hostname = "host/name"),
                valid.copy(username = ""), valid.copy(session = "../else"), valid.copy(session = "."))) {
            assertNotNull(profile.validationError())
        }
    }
}
