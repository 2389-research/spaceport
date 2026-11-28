// ABOUTME: Owns one remote Herdr session and verifies SSH trust before login.
// ABOUTME: Keeps the session alive across backgrounds and fences stale terminal input.
package ai.spaceport.mobile

import android.content.Context
import android.os.Handler
import android.os.Looper
import java.util.concurrent.Executors
import uniffi.spaceport_mobile.InputTarget
import uniffi.spaceport_mobile.MobileException
import uniffi.spaceport_mobile.MobilePane
import uniffi.spaceport_mobile.MobileSession
import uniffi.spaceport_mobile.MobileView
import uniffi.spaceport_mobile.SshAuth
import uniffi.spaceport_mobile.SshException
import uniffi.spaceport_mobile.SshHost
import uniffi.spaceport_mobile.checkSshHost
import uniffi.spaceport_mobile.enclavePublicKey

enum class ConnectionPhase { Disconnected, Connecting, Trust, Password, Connected, Parked, Failed }

/** Mutate from the UI thread. Workers may use sessionFor with their captured input target. */
class SessionController(context: Context) {
    private val store = HostStore(context)
    private val main = Handler(Looper.getMainLooper())
    private val io = Executors.newSingleThreadExecutor()
    private val keys = Executors.newSingleThreadExecutor()
    internal val draft = SessionDraft()
    private val recovery = TransportRecovery()
    private var native: MobileSession? = null
    private var paneTrail: uniffi.spaceport_mobile.MobilePaneTrail? = null
    @Volatile private var generation = 0L
    private var closed = false
    private var pinnedAtOpen: String? = null
    private var parkedBoot: String? = null
    private var reopenOnResume = false
    private var passwordLogin = false
    private var rememberKeyOnConnect = false
    private var attemptedEnrollment = false
    private var enrollOnConnect = false
    private var loadingKey = false
    private var pendingPoll: Runnable? = null
    private var rereadView = false

    private data class Presentation(val view: MobileView, val session: MobileSession)
    @Volatile private var presentation: Presentation? = null
    var changed: (() -> Unit)? = null
    var profile: HostProfile? = null
        private set
    var previewPort: Int? = null
        private set

    fun startServicePreview(remote: Int): Int {
        require(remote in 1..65535)
        val handle = checkNotNull(native?.takeIf { phase == ConnectionPhase.Connected })
        val local = handle.startServicePreview(remote.toUShort()).toInt()
        previewPort = local
        return local
    }

    private var stoppingPreview = false

    fun stopServicePreview(onError: (String) -> Unit) {
        val session = native ?: return
        val port = previewPort ?: return
        if (stoppingPreview) return
        stoppingPreview = true
        io.execute {
            val result = runCatching { session.stopServicePreview() }
            main.post {
                if (native !== session || previewPort != port) return@post
                stoppingPreview = false
                result.onSuccess { previewPort = null }.onFailure { onError("The preview is still closing. Try Stop again.") }
            }
        }
    }

    private enum class Stage(val label: String) {
        VERIFY("SSH host verification"), OPEN("SSH login / Herdr attach"), ROSTER("Herdr handshake / roster"),
        CONNECTED("Connected"), RESUME("Verify retained session")
    }
    private var stage = Stage.VERIFY
    private var lastFailure: Stage? = null
    private var lastCapabilities: String? = null
    private var lastRoster: Long? = null

    private fun connectionStage(): Stage =
        if (stage == Stage.OPEN && native?.diagnosticCapabilities() != null) Stage.ROSTER else stage

    fun diagnosticReport(build: Long, cachedObservation: Long?): String {
        val caps = native?.diagnosticCapabilities() ?: lastCapabilities ?: "unavailable"
        val observed = (lastRoster ?: cachedObservation)?.let(java.time.Instant::ofEpochMilli)?.toString() ?: "unknown"
        return listOf("Spaceport Android build $build", "Stage: ${connectionStage().label}",
            "Last failure: ${lastFailure?.label ?: "none"}", "Last negotiated capabilities: $caps",
            "Last roster observation: $observed").joinToString("\n")
    }

    var phase = ConnectionPhase.Disconnected
        private set
    var view: MobileView? = null
        private set
    var message: String? = null
        private set
    var fingerprint: String? = null
        private set
    var keyChanged = false
        private set
    val keyEnrolled get() = profile?.let(store::enrolled) == true
    var enrolling = false
        private set
    val canEnroll get() = keyHardwareBacked != false && passwordLogin && !attemptedEnrollment && !enrolling && phase == ConnectionPhase.Connected
    var publicKey: String? = null
        private set
    var keyHardwareBacked: Boolean? = if (android.os.Build.VERSION.SDK_INT < 28) false else null
        private set

    fun connect(profile: HostProfile) {
        if (closed) return
        retire()
        recovery.allow()
        keyChanged = false
        fingerprint = null
        this.profile = profile
        val invalid = profile.validationError()
        if (invalid != null) return fail(invalid)
        startLogin(profile, requestPassword = !store.enrolled(profile))
    }

    private fun startLogin(p: HostProfile, requestPassword: Boolean) {
        stage = Stage.VERIFY
        val pin = store.pin(p)
        pinnedAtOpen = pin
        keyChanged = false
        fingerprint = null
        if (!requestPassword && pin != null) {
            open(p, pin, null)
            return
        }
        val token = generation
        show(ConnectionPhase.Connecting)
        io.execute {
            if (token != generation) return@execute
            try {
                checkSshHost(p.sshHost(pin))
                main.post {
                    if (token == generation) {
                        if (pin != null && store.pin(p) == pin) show(ConnectionPhase.Password)
                        else fail("SSH host trust changed. Connect again to verify this host.")
                    }
                }
            } catch (e: SshException.UnknownHost) {
                main.post { if (token == generation) challenge(e.fingerprint, false) }
            } catch (e: SshException.ChangedHost) {
                main.post { if (token == generation) challenge(e.fingerprint, true) }
            } catch (e: Exception) {
                main.post { if (token == generation) fail(diagnostic(e)) }
            }
        }
    }

    private fun challenge(value: String, changedKey: Boolean) {
        stage = Stage.VERIFY
        if (changedKey) lastFailure = connectionStage()
        fingerprint = value
        keyChanged = changedKey
        if (changedKey) show(ConnectionPhase.Failed, "The SSH host key changed. Verify the new fingerprint with the host operator.")
        else show(ConnectionPhase.Trust)
    }

    fun trustHost() {
        val p = profile ?: return
        val value = fingerprint ?: return
        if (phase != ConnectionPhase.Trust || keyChanged) return
        if (!store.trust(p, value)) return fail("A different fingerprint is already saved for this host.")
        pinnedAtOpen = value
        // The authenticated connection checks the pinned key again before sending a credential.
        show(ConnectionPhase.Password)
    }

    /** Call only after explicit confirmation to forget this failed endpoint's original pin. */
    fun forgetHostKey() {
        val p = profile ?: return
        val pin = pinnedAtOpen ?: return
        if (phase != ConnectionPhase.Failed || !keyChanged) return
        if (!store.forgetPin(p, pin)) return fail("The saved host fingerprint changed. Connect again.")
        connect(p)
    }

    fun usePassword() {
        val p = profile ?: return
        if (closed || keyChanged) return
        retire()
        startLogin(p, requestPassword = true)
    }

    /** [authorize] installs and verifies this phone's key once the password session connects. */
    fun login(password: String, authorize: Boolean = false) {
        enrollOnConnect = authorize
        authenticate(password)
    }

    /** Use a public key the operator installed on a new host themselves. */
    fun loginWithKey() = authenticate(null)

    private fun authenticate(password: String?) {
        val p = profile ?: return
        if (phase != ConnectionPhase.Password) return
        val pin = pinnedAtOpen
        if (pin == null || store.pin(p) != pin) return fail("Verify this host again before signing in.")
        open(p, pin, password)
    }

    private fun open(p: HostProfile, pin: String, password: String?) {
        val token = generation
        stage = Stage.OPEN
        passwordLogin = password != null
        rememberKeyOnConnect = password == null && !store.enrolled(p)
        show(ConnectionPhase.Connecting)
        io.execute {
            if (token != generation) return@execute
            try {
                val auth = if (password == null) {
                    val key = PhoneKey.open()
                    SshAuth.SecureEnclave(key.point, key)
                } else SshAuth.Password(password)
                if (token != generation) return@execute
                val session = MobileSession.open(p.sshHost(pin), auth, p.session, 80u, 24u, 8u, 16u)
                main.post {
                    if (token != generation) {
                        session.disconnect()
                        session.close()
                    } else {
                        paneTrail?.let(session::restorePaneTrail)
                        native = session
                        poll(session, token, 0u)
                    }
                }
            } catch (e: Exception) {
                main.post {
                    if (token == generation) {
                        if (e is HardwareKeyUnavailable) keyHardwareBacked = false
                        fail(diagnostic(e))
                    }
                }
            }
        }
    }

    private fun poll(session: MobileSession, token: Long, version: ULong) {
        if (token != generation) return
        val after = if (rereadView) 0uL else version
        rereadView = false
        io.execute {
            if (token != generation) return@execute
            try {
                val update = session.poll(after)
                main.post {
                    if (token != generation) return@post
                    // A focus request can invalidate a projection already travelling to the UI.
                    if (update != null && !rereadView) receive(session, update)
                    if (token == generation) schedule { poll(session, token, update?.version ?: version) }
                }
            } catch (e: Exception) {
                main.post { if (token == generation) fail(diagnostic(e)) }
            }
        }
    }

    private fun receive(session: MobileSession, update: MobileView) {
        if (update.disconnected || update.hostChallenge != null) {
            val challenge = update.hostChallenge
            if (challenge != null) {
                retire()
                challenge(challenge.fingerprint, challenge.changed)
            } else {
                lost(session, update.error ?: "The SSH connection closed.")
            }
            return
        }
        view = update
        val ready = update.bootId.isNotEmpty()
        presentation = if (ready && !update.goToPending) Presentation(update, session) else null
        if (ready) {
            stage = Stage.CONNECTED
            update.rosterObservedAtMs?.toLong()?.takeIf { it >= 0 }?.let { lastRoster = it }
            if (phase != ConnectionPhase.Connected) profile?.let {
                store.allowHomeObservation(it, true)
                store.recordOpened(it)
            }
            phase = ConnectionPhase.Connected
            if (rememberKeyOnConnect) {
                rememberKeyOnConnect = false
                val p = profile
                val pin = pinnedAtOpen
                if (p != null && pin != null) store.recordEnrollment(p, pin)
            }
        }
        if (update.error != null) message = update.error
        changed?.invoke()
        if (ready && enrollOnConnect && canEnroll) {
            enrollOnConnect = false
            enrollKey()
        }
    }

    fun sessionFor(target: InputTarget): MobileSession? = presentation?.takeIf { it.view.inputTarget == target }?.session

    /** Resizing and explicitly claiming the pane can be needed before Herdr permits input. */
    fun sessionFor(view: MobileView): MobileSession? = presentation?.takeIf { it.view === view }?.session

    private fun currentSession(from: MobileView): MobileSession? {
        val active = presentation ?: return null
        val current = active.view
        if (current.bootId != from.bootId) return null
        val connection = from.inputTarget?.connection
        return active.session.takeIf { from === current || (connection != null && connection == current.inputTarget?.connection) }
    }

    fun goTo(pane: MobilePane, from: MobileView): ULong? {
        val session = currentSession(from) ?: return null
        val tab = from.tabs.firstOrNull { it.id == pane.tabId } ?: return null
        return try {
            val request = session.goToPane(from.bootId, tab.workspaceId, tab.id, pane.id, pane.agentName)
            refreshInput()
            request
        } catch (e: Exception) {
            notice(diagnostic(e))
            null
        }
    }

    fun travel(from: MobileView, pane: String, forward: Boolean): ULong? {
        val session = currentSession(from) ?: return null
        return try {
            session.travelToPane(from.bootId, pane, forward).also { refreshInput() }
        } catch (e: Exception) {
            notice(diagnostic(e))
            null
        }
    }

    internal fun editPane(from: MobileView, row: AgentRow, action: uniffi.spaceport_mobile.MobilePaneAction) =
        navigate(from) { it.applyPaneAction(from.bootId, row.workspace, row.tab, row.id, action) }

    fun cancelGoTo() {
        if (phase != ConnectionPhase.Connected) return
        native?.cancelGoTo()
        refreshInput()
    }

    fun createTab(from: MobileView) {
        val workspace = from.focusedWorkspace ?: return
        val tab = from.focusedTab ?: return
        if (sessionFor(from) == null || from.inputTarget == null) return
        navigate(from) { it.createTab(from.bootId, workspace, tab) }
    }

    fun focusWorkspace(id: String, from: MobileView) = navigate(from) { it.focusWorkspace(from.bootId, id) }
    fun focusTab(id: String, from: MobileView) = navigate(from) { it.focusTab(from.bootId, id) }

    private fun navigate(from: MobileView, action: (MobileSession) -> Unit) {
        val session = currentSession(from) ?: return
        try {
            action(session)
            refreshInput()
        } catch (e: Exception) {
            notice(diagnostic(e))
        }
    }

    private fun refreshInput() {
        presentation = null
        rereadView = true
    }

    fun park() {
        if (closed || phase == ConnectionPhase.Parked) return
        val session = native
        val boot = view?.bootId ?: parkedBoot
        if (session != null && !boot.isNullOrEmpty()) {
            recovery.allow()
            invalidate()
            session.park()
            parkedBoot = boot
            view = null
            show(ConnectionPhase.Parked)
        } else if (phase == ConnectionPhase.Connecting) {
            retire()
            reopenOnResume = true
            show(ConnectionPhase.Parked)
        }
    }

    internal fun parkedRoster(): RosterRead? {
        if (phase != ConnectionPhase.Parked) return null
        val session = native ?: return null
        val boot = parkedBoot ?: return null
        val request = session.beginParkedRoster(boot)
        return RosterRead({ session.waitParkedRoster(boot, request) }, { session.cancelParkedRoster(request) })
    }

    fun resume() {
        if (phase != ConnectionPhase.Parked) return
        val p = profile ?: return
        if (reopenOnResume) return connect(p)
        val session = native ?: return
        val boot = parkedBoot ?: return
        val token = generation
        show(ConnectionPhase.Connecting)
        try {
            stage = Stage.RESUME
            val request = session.beginResume(boot)
            checkResume(session, boot, request, token)
        } catch (e: Exception) {
            lost(session, diagnostic(e))
        }
    }

    private fun checkResume(session: MobileSession, boot: String, request: String, token: Long) {
        if (generation != token) return
        io.execute {
            if (generation != token) return@execute
            try {
                val confirmed = session.checkResume(boot, request)
                val update = if (confirmed) session.poll(0u) else null
                main.post {
                    if (generation != token) return@post
                    if (!confirmed) schedule { checkResume(session, boot, request, token) }
                    else if (update == null || update.bootId != boot || update.disconnected) {
                        lost(session, "Herdr could not confirm the original connection. Connect again.")
                    } else {
                        parkedBoot = null
                        receive(session, update)
                        if (token == generation) poll(session, token, update.version)
                    }
                }
            } catch (e: Exception) {
                main.post {
                    if (token == generation) lost(session, diagnostic(e), resumeTimeout = e is MobileException.ResumeTimeout)
                }
            }
        }
    }

    private fun lost(session: MobileSession, reason: String, resumeTimeout: Boolean = false) {
        val p = profile
        lastFailure = connectionStage()
        val interrupted = session.focusInterrupted()
        val established = phase == ConnectionPhase.Connected || parkedBoot != null
        if (p != null && recovery.take(established, session.transportLost() || resumeTimeout, interrupted)) {
            // Start a new login without retaining credentials or replaying navigation or terminal input.
            retire()
            startLogin(p, requestPassword = !store.enrolled(p))
        } else {
            fail(if (interrupted) "The connection closed during a focus change. Connect and select the pane again." else reason)
        }
    }

    fun loadPublicKey() {
        if (closed || loadingKey || publicKey != null) return
        loadingKey = true
        keys.execute {
            try {
                val text = enclavePublicKey(PhoneKey.open().point)
                main.post {
                    if (!closed) {
                        loadingKey = false
                        publicKey = text
                        keyHardwareBacked = true
                        changed?.invoke()
                    }
                }
            } catch (e: Exception) {
                main.post {
                    if (!closed) {
                        loadingKey = false
                        if (e is HardwareKeyUnavailable) keyHardwareBacked = false
                        notice(diagnostic(e))
                    }
                }
            }
        }
    }

    fun enrollKey() {
        val session = native ?: return
        val p = profile ?: return
        val pin = pinnedAtOpen ?: return
        if (!canEnroll || store.pin(p) != pin) return
        enrolling = true
        val token = generation
        changed?.invoke()
        keys.execute {
            try {
                if (generation != token) throw SshException.Cancelled()
                val key = PhoneKey.open()
                if (generation != token) throw SshException.Cancelled()
                session.authorizePhone(key.point, key)
                val text = enclavePublicKey(key.point)
                main.post {
                    if (native !== session) return@post
                    enrolling = false
                    attemptedEnrollment = true
                    publicKey = text
                    keyHardwareBacked = true
                    if (store.recordEnrollment(p, pin)) notice("Device key installed and verified.")
                    else notice("The host fingerprint changed during enrollment. Verify the host again.")
                }
            } catch (e: Exception) {
                main.post {
                    if (native !== session) return@post
                    enrolling = false
                    attemptedEnrollment = true
                    if (e is HardwareKeyUnavailable) keyHardwareBacked = false
                    notice(diagnostic(e))
                }
            }
        }
    }

    fun disconnect() {
        if (closed) return
        draft.clear()
        retire()
        fingerprint = null
        keyChanged = false
        show(ConnectionPhase.Disconnected)
    }

    fun close() {
        if (closed) return
        closed = true
        draft.clear()
        changed = null
        retire()
        phase = ConnectionPhase.Disconnected
        paneTrail?.close()
        paneTrail = null
        io.shutdown()
        keys.shutdown()
    }

    private fun schedule(action: () -> Unit) {
        val next = Runnable(action)
        pendingPoll = next
        main.postDelayed(next, 35)
    }

    private fun invalidate() {
        generation++
        presentation = null
        pendingPoll?.let(main::removeCallbacks)
        pendingPoll = null
        rereadView = false
    }

    private fun retire() {
        stoppingPreview = false
        previewPort = null
        invalidate()
        native?.let { session ->
            session.diagnosticCapabilities()?.let { lastCapabilities = it }
            session.savedPaneTrail()?.let {
                paneTrail?.close()
                paneTrail = it
            }
            session.disconnect()
            io.execute { session.close() }
        }
        native = null
        view = null
        parkedBoot = null
        reopenOnResume = false
        passwordLogin = false
        rememberKeyOnConnect = false
        enrollOnConnect = false
        enrolling = false
        attemptedEnrollment = false
    }

    private fun fail(text: String) {
        lastFailure = connectionStage()
        retire()
        show(ConnectionPhase.Failed, text)
    }

    private fun show(next: ConnectionPhase, text: String? = null) {
        phase = next
        message = text
        changed?.invoke()
    }

    private fun notice(text: String) {
        message = text
        changed?.invoke()
    }
}

internal fun HostProfile.sshHost(pin: String?) = SshHost(hostname, port.toUShort(), username, pin)

private fun diagnostic(error: Exception): String = when (error) {
    is HardwareKeyUnavailable -> error.message.orEmpty()
    is SshException.Authentication -> "SSH authentication failed."
    is SshException.Timeout -> "The SSH host did not answer in time."
    is SshException.Cancelled -> "This SSH operation ended. Reconnect to try again."
    is SshException.Enrollment, is SshException.Channel -> "The host refused the request. Reconnect with a password to retry enrollment."
    is SshException.UnverifiedEnrollment -> "The host may have installed the key. Spaceport could not verify key-only login."
    is SshException.ChangedHost, is SshException.UnknownHost -> "The host presented an unexpected key during verification."
    is SshException -> "Could not establish a verified SSH connection."
    is MobileException.Client -> error.reason
    is MobileException.ResumeTimeout, is MobileException.ResumeUnconfirmed -> "Herdr could not confirm the retained connection. Connect again."
    is MobileException.StaleView, is MobileException.NavigationPending -> "The pane changed. Select it again."
    else -> error.localizedMessage?.takeIf { it.isNotBlank() } ?: "The connection operation failed."
}
