// ABOUTME: Reads saved Home rosters sequentially while the Home screen is in the foreground.
// ABOUTME: Cancels the exact SSH request on navigation and fences late replies against newer observations.
package ai.spaceport.mobile

import android.os.Handler
import android.os.Looper
import java.util.concurrent.Executor
import uniffi.spaceport_mobile.MobileRosterRequest
import uniffi.spaceport_mobile.MobileView
import uniffi.spaceport_mobile.SshAuth

internal class RosterRead(val read: () -> MobileView, val cancel: () -> Unit)

internal class HomeRosterRefresh(
    private val hosts: HostStore,
    private val catalog: AgentCatalog,
    private val sessions: Map<HostProfile, SessionController>,
    private val executor: Executor,
    private val changed: () -> Unit,
) {
    private val main = Handler(Looper.getMainLooper())
    private var pass = 0L
    private var pending: RosterRead? = null
    var checking: HostProfile? = null
        private set
    val failed = mutableSetOf<HostProfile>()

    fun cancel() {
        pass++
        pending?.cancel?.invoke()
        pending = null
        checking = null
        failed.clear()
    }

    fun start() {
        cancel()
        val token = pass
        val profiles = hosts.profiles().filter { host ->
            hosts.permitsHomeObservation(host) &&
                (sessions[host]?.phase == ConnectionPhase.Parked || hosts.enrolled(host))
        }.iterator()

        fun next() {
            if (pass != token) return
            checking = null
            pending = null
            while (profiles.hasNext()) {
                val host = profiles.next()
                val pin = hosts.pin(host) ?: continue
                val previouslySeen = catalog.observedAt(host)
                val read = try {
                    sessions[host]?.parkedRoster() ?: if (hosts.enrolled(host)) cold(host, pin) else null
                } catch (_: Exception) {
                    failed.add(host)
                    null
                } ?: continue
                checking = host
                pending = read
                changed()
                executor.execute {
                    val result = runCatching(read.read)
                    main.post {
                        if (token != pass) return@post
                        if (hosts.pin(host) == pin && host in hosts.profiles() &&
                            hosts.permitsHomeObservation(host) && catalog.observedAt(host) == previouslySeen) {
                            val view = result.getOrNull()
                            if (view?.observedAt != null && view.bootId.isNotEmpty() && !view.disconnected) {
                                catalog.observe(host, view)
                            } else failed.add(host)
                        }
                        next()
                    }
                }
                return
            }
            changed()
        }
        next()
    }

    private fun cold(host: HostProfile, pin: String): RosterRead {
        val probe = MobileRosterRequest()
        // Ownership crosses the worker boundary. The main thread only cancels a live request.
        return RosterRead({
            probe.use {
                check(hosts.pin(host) == pin && hosts.enrolled(host) && hosts.permitsHomeObservation(host))
                val signer = PhoneKey.open(createIfMissing = false)
                it.read(host.sshHost(pin), SshAuth.SecureEnclave(signer.point, signer), host.session)
            }
        }, {
            // The worker may have closed the UniFFI handle while its main-thread reply is pending.
            runCatching { probe.cancel() }
        })
    }
}
