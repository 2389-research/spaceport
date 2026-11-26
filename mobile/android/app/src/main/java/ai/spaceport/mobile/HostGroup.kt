// ABOUTME: Gathers saved Herdr sessions under the SSH account that reaches them.
// ABOUTME: One group shares an address, a trusted host key and phone-key enrollment.
package ai.spaceport.mobile

internal data class HostGroup(val account: String, val sessions: List<HostProfile>) {
    /** Any session stands for the group's address; they differ only by session name. */
    val first get() = sessions.first()
    val destination get() = first.destination

    /** The same session names on another address; trust and enrollment start over there. */
    fun movedTo(hostname: String, port: Int, username: String) =
        sessions.map { it.copy(hostname = hostname, port = port, username = username) }

    companion object {
        fun of(profiles: List<HostProfile>): List<HostGroup> =
            profiles.groupBy { it.account }.map { (account, sessions) ->
                HostGroup(account, sessions.sortedBy { it.session })
            }
    }
}
