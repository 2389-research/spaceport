// ABOUTME: Keeps recovery bounded and refuses to repeat a pending pane jump.
// ABOUTME: Exercises the same eligibility check used after foreground resumption.
package ai.spaceport.mobile

import org.junit.Assert.*
import org.junit.Test

class TransportRecoveryTest {
    @Test fun requiresAnEstablishedLostConnectionAndConsumesOneAttempt() {
        val retry = TransportRecovery()
        assertFalse(retry.take(established = false, transportLost = true, focusInterrupted = false))
        assertFalse(retry.take(established = true, transportLost = false, focusInterrupted = false))
        assertFalse(retry.take(established = true, transportLost = true, focusInterrupted = true))
        assertTrue(retry.take(established = true, transportLost = true, focusInterrupted = false))
        assertFalse(retry.take(established = true, transportLost = true, focusInterrupted = false))
        retry.allow()
        assertTrue(retry.take(established = true, transportLost = true, focusInterrupted = false))
    }
}
