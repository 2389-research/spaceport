// ABOUTME: Allows one fresh SSH attempt for a presented, previously established session.
// ABOUTME: Defers that attempt during interruption and never queues terminal input.
struct TransportRecovery {
    enum Action { case stop, deferUntilActive, reconnect }
    private var active = true
    private var available = true
    private var deferred = false

    mutating func allowRetry() { available = true }

    mutating func sceneInactive(established: Bool) {
        if active && established { allowRetry() }
        active = false
    }

    mutating func sceneActive() -> Bool {
        active = true
        let retry = deferred
        deferred = false
        return retry
    }

    mutating func loss(eligible: Bool, presented: Bool, focusPending: Bool = false) -> Action {
        guard eligible, presented, available, !focusPending else { return .stop }
        available = false
        if active { return .reconnect }
        deferred = true
        return .deferUntilActive
    }
}
