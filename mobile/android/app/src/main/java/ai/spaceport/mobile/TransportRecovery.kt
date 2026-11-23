// ABOUTME: Permits one fresh transport after an established connection drops.
// ABOUTME: Preserves uncertain pane selection for the operator to resolve.
package ai.spaceport.mobile

internal class TransportRecovery {
    private var available = true

    fun allow() { available = true }

    fun take(established: Boolean, transportLost: Boolean, focusInterrupted: Boolean): Boolean {
        if (!available || !established || !transportLost || focusInterrupted) return false
        available = false
        return true
    }
}
