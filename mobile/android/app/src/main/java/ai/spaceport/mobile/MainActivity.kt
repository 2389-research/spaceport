// ABOUTME: Connects the Android host catalog and agent picker to retained Herdr sessions.
// ABOUTME: Keeps terminal input hidden until the selected agent and surface agree.
package ai.spaceport.mobile

import android.app.AlertDialog
import android.app.Application
import android.content.ClipData
import android.content.ClipboardManager
import android.os.Build
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.text.InputType
import android.view.Gravity
import android.view.WindowInsets
import android.view.WindowManager
import android.widget.*
import androidx.activity.ComponentActivity
import androidx.activity.OnBackPressedCallback
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.ViewModelProvider
import java.util.concurrent.ExecutorService
import java.util.concurrent.Executors
import uniffi.spaceport_mobile.MobileView

internal class RetainedPhone(app: Application) : AndroidViewModel(app) {
    val hosts = HostStore(app)
    val catalog = AgentCatalog(app)
    val sessions = linkedMapOf<HostProfile, SessionController>()
    val previews = mutableMapOf<HostProfile, PanePreviews>()
    var selected: HostProfile? = null
    var pending: AgentRow? = null
    var routed = false
    var revision: ULong? = null
    var toWorkspaces = false
    var routeError: String? = null
    /** Screens opened above Home, newest last. */
    val pages = mutableListOf<Page>()
    val work: ExecutorService = Executors.newSingleThreadExecutor()

    override fun onCleared() {
        sessions.values.forEach { it.close() }
        work.shutdownNow()
    }
}

class MainActivity : ComponentActivity(), Pages {
    private lateinit var phone: RetainedPhone
    private lateinit var ui: PhoneUi
    private lateinit var root: LinearLayout
    private var terminal: TerminalScreen? = null
    private var terminalBar: TextView? = null
    private var homeButton: Button? = null
    private var picker: AgentPicker? = null
    private var page = ""
    private var password: EditText? = null
    private var refreshPhoneKey: (() -> Unit)? = null
    private var lastNotice: String? = null
    private var pageToken = 0L
    private var install: KeyInstallScreen? = null
    private val handler = Handler(Looper.getMainLooper())
    private val refresh = Runnable { if (page == "home" && !isFinishing) showHome() }
    private val active
        get() = phone.sessions[phone.selected]

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        phone = ViewModelProvider(this)[RetainedPhone::class.java]
        ui = PhoneUi(this)
        root =
            ui.column().apply {
                setBackgroundColor(ui.bg)
                isFocusableInTouchMode = true
            }
        if (Build.VERSION.SDK_INT >= 30) {
            @Suppress("DEPRECATION") window.setDecorFitsSystemWindows(false)
            root.setOnApplyWindowInsetsListener { v, insets ->
                val safe =
                    insets.getInsets(
                        WindowInsets.Type.systemBars() or
                            WindowInsets.Type.displayCutout() or
                            WindowInsets.Type.ime()
                    )
                v.setPadding(safe.left, safe.top, safe.right, safe.bottom)
                insets
            }
        }
        if (Build.VERSION.SDK_INT >= 33) setRecentsScreenshotEnabled(false)
        else window.addFlags(WindowManager.LayoutParams.FLAG_SECURE)
        onBackPressedDispatcher.addCallback(
            this,
            object : OnBackPressedCallback(true) {
                override fun handleOnBackPressed() {
                    when {
                        phone.selected != null -> goHome()
                        phone.pages.isNotEmpty() -> back()
                        else -> finish()
                    }
                }
            },
        )
        setContentView(root)
        phone.sessions.forEach { (host, c) -> listen(host, c) }
        if (phone.selected == null) showHome() else render()
    }

    override fun onStart() {
        super.onStart()
        active?.let { if (it.phase == ConnectionPhase.Parked) it.resume() }
        // Pages keep their own state across a trip to the background; only Home refreshes.
        if (phone.selected == null && page == "home") showHome()
    }

    override fun onStop() {
        if (!isChangingConfigurations) {
            if (phone.pending != null && phone.routed)
                phone.routeError = "The session paused. Select the agent again."
            phone.sessions.forEach { (host, c) ->
                observe(host, c)
                phone.previews[host]?.clear()
                c.park()
            }
            password?.text?.clear()
            install?.pause()
        }
        super.onStop()
    }

    override fun onDestroy() {
        handler.removeCallbacks(refresh)
        picker?.dismiss()
        install?.dispose()
        terminal?.dispose()
        phone.sessions.values.forEach { it.changed = null }
        super.onDestroy()
    }

    private fun listen(host: HostProfile, c: SessionController) {
        c.changed = {
            val view = c.view
            if (c.phase == ConnectionPhase.Connected && view != null)
                phone.previews[host]?.observe(view)
            else if (c.phase != ConnectionPhase.Connected) phone.previews[host]?.clear()
            if (phone.selected == host && !isDestroyed) {
                render()
                refreshPhoneKey?.invoke()
            }
        }
    }

    private fun observe(host: HostProfile, c: SessionController) {
        if (c.phase != ConnectionPhase.Connected) return
        val view = c.view?.takeIf { it.bootId.isNotEmpty() && !it.disconnected } ?: return
        phone.catalog.observe(host, AgentBoard.rows(host, view))
    }

    private fun open(host: HostProfile, agent: AgentRow? = null, workspaces: Boolean = false) {
        handler.removeCallbacks(refresh)
        phone.selected = host
        phone.pending = agent
        phone.routed = false
        phone.revision = null
        phone.routeError = null
        phone.toWorkspaces = workspaces
        lastNotice = null
        val reused = phone.sessions.remove(host)
        val c = reused ?: SessionController(applicationContext)
        phone.sessions[host] = c
        phone.previews.getOrPut(host) { PanePreviews() }
        if (phone.sessions.size > 2) {
            val (old, retired) = phone.sessions.entries.first()
            observe(old, retired)
            retired.changed = null
            retired.close()
            phone.sessions.remove(old)
            phone.previews.remove(old)
        }
        listen(host, c)
        if (
            reused == null ||
                c.phase == ConnectionPhase.Disconnected ||
                c.phase == ConnectionPhase.Failed
        )
            c.connect(host)
        else if (c.phase == ConnectionPhase.Parked) c.resume()
        render()
    }

    private fun clear(next: String) {
        pageToken++
        install?.dispose()
        install = null
        if (page == "terminal") terminal?.dispose()
        terminal = null
        terminalBar = null
        homeButton = null
        password = null
        page = next
        root.removeAllViews()
    }

    private fun goHome() {
        val host = phone.selected
        val controller = active
        if (host != null && controller != null) observe(host, controller)
        picker?.dismiss()
        if (phone.pending != null && phone.routed) controller?.cancelGoTo()
        phone.selected = null
        phone.pending = null
        phone.routeError = null
        phone.toWorkspaces = false
        if (controller?.phase == ConnectionPhase.Connected) {
            controller.park()
        } else if (
            host != null && controller != null && controller.phase != ConnectionPhase.Parked
        ) {
            controller.changed = null
            phone.sessions.remove(host)
            phone.previews.remove(host)
            controller.close()
        }
        showHome()
    }

    private fun showHome() {
        phone.pages.lastOrNull()?.let { return showPage(it) }
        clear("home")
        val profiles = phone.hosts.profiles()
        val tiles = profiles.flatMap(phone.catalog::rows)
        val now = System.currentTimeMillis()
        val header = ui.line().apply { setPadding(ui.dp(18), ui.dp(8), ui.dp(8), ui.dp(4)) }
        header.addView(ui.text("Agents", 30, ui.ink, true), LinearLayout.LayoutParams(0, -2, 1f))
        header.addView(ui.button("Settings") { go(Page.Settings) })
        header.addView(ui.button("Add host") { editHost() })
        root.addView(header)
        val body = ui.column().apply { setPadding(ui.dp(14), 0, ui.dp(14), ui.dp(24)) }
        root.addView(ScrollView(this).apply { addView(body) })
        if (tiles.isEmpty()) {
            body.addView(
                ui.text(
                        if (profiles.isEmpty()) "Connect an SSH host to see its Herdr agents."
                        else "Open a session to see its agents here.",
                        16,
                        ui.muted,
                    )
                    .apply { setPadding(ui.dp(4), ui.dp(24), ui.dp(4), ui.dp(16)) }
            )
        }
        AgentBoard.sorted(tiles, phone.catalog.marks, now)
            .groupBy { AgentBoard.section(it, phone.catalog.marks[it.key], now) }
            .forEach { (section, rows) ->
                ui.heading(body, section.label)
                rows.forEach { row ->
                    val details = buildList {
                        add(row.path)
                        if (now - row.seenAt >= 60_000)
                            add(
                                "Seen " +
                                    android.text.format.DateUtils.getRelativeTimeSpanString(
                                        row.seenAt,
                                        now,
                                        60_000,
                                    )
                            )
                        if (profiles.size > 1) add("${row.host.destination} · ${row.host.session}")
                        if (section == AgentSection.SNOOZED)
                            phone.catalog.marks[row.key]?.let {
                                add(
                                    "Until " +
                                        android.text.format.DateFormat.getTimeFormat(
                                                this@MainActivity
                                            )
                                            .format(java.util.Date(it.until))
                                )
                            }
                    }
                    ui.row(
                        body,
                        row.headline,
                        details.joinToString(" · "),
                        row.status,
                        { open(row.host, row) },
                        { marks(row) },
                        swiped = { towardEnd -> swiped(row, towardEnd) },
                        actions = markChoices(row).map { (label, next) -> label to { mark(row, next) } },
                    )
                }
            }
        if (profiles.isNotEmpty()) {
            ui.heading(body, "Workspaces")
            ui.row(
                body,
                "Browse workspaces",
                "Choose a workspace in a saved session",
                action = {
                    if (profiles.size == 1) open(profiles.first(), workspaces = true)
                    else
                        AlertDialog.Builder(this)
                            .setTitle("Choose a session")
                            .setItems(
                                profiles.map { "${it.session} · ${it.destination}" }.toTypedArray()
                            ) { _, i ->
                                open(profiles[i], workspaces = true)
                            }
                            .show()
                },
            )
            for (group in HostGroup.of(profiles)) {
                val title = phone.hosts.name(group.account)?.let { "$it · ${group.destination}" } ?: group.destination
                val heading = ui.line()
                heading.addView(ui.text(title, 14, ui.muted, true).apply {
                    setPadding(ui.dp(3), ui.dp(20), ui.dp(3), ui.dp(7))
                    if (Build.VERSION.SDK_INT >= 28) isAccessibilityHeading = true
                }, LinearLayout.LayoutParams(0, -2, 1f))
                heading.addView(ui.button("Details") { go(Page.HostDetail(group.account)) }.apply {
                    contentDescription = "Host details for ${group.destination}"
                })
                body.addView(heading)
                group.sessions.forEach { host ->
                    ui.row(body, host.session, if (phone.sessions.containsKey(host)) "Connected" else "Saved session",
                        action = { open(host) }, held = { go(Page.HostDetail(group.account)) })
                }
                body.addView(ui.button("New session") {
                    ui.askSessionName("New session", "") { name ->
                        val next = group.first.copy(session = name)
                        if (addSession(emptyList(), next)) open(next)
                    }
                })
            }
        }
        handler.removeCallbacks(refresh)
        phone.catalog.marks.values
            .map { it.until }
            .filter { it > now }
            .minOrNull()
            ?.let { handler.postDelayed(refresh, it - now + 1) }
    }

    /** Every mark change offered for [row], in menu order. The first two are the swipe actions. */
    private fun markChoices(row: AgentRow): List<Pair<String, AgentMark>> {
        val now = System.currentTimeMillis()
        val mark = (phone.catalog.marks[row.key] ?: AgentMark()).current(row.status, now)
        fun snooze(until: Long) = mark.copy(until = until, held = row.status)
        val morning = java.util.Calendar.getInstance().apply {
            add(java.util.Calendar.DAY_OF_YEAR, 1)
            set(java.util.Calendar.HOUR_OF_DAY, 9)
            set(java.util.Calendar.MINUTE, 0)
            set(java.util.Calendar.SECOND, 0)
            set(java.util.Calendar.MILLISECOND, 0)
        }.timeInMillis
        return buildList {
            add((if (mark.pinned) "Unpin" else "Pin") to mark.copy(pinned = !mark.pinned))
            if (mark.until > 0) add("Wake now" to mark.copy(until = 0, held = ""))
            add("Snooze for 1 hour" to snooze(now + 3_600_000L))
            add("Snooze for 3 hours" to snooze(now + 3 * 3_600_000L))
            add("Snooze until tomorrow at 9 AM" to snooze(morning))
        }
    }

    private fun mark(row: AgentRow, next: AgentMark) {
        phone.catalog.mark(row, next)
        showHome()
    }

    private fun marks(row: AgentRow) {
        val choices = markChoices(row)
        AlertDialog.Builder(this)
            .setTitle(row.headline)
            .setItems(choices.map { it.first }.toTypedArray()) { _, i -> mark(row, choices[i].second) }
            .show()
    }

    /** A swipe toward the end pins or unpins; toward the start snoozes for an hour or wakes. */
    private fun swiped(row: AgentRow, towardEnd: Boolean) {
        val choices = markChoices(row)
        mark(row, choices[if (towardEnd) 0 else 1].second)
    }

    private fun editHost() {
        val form = ui.column().apply { setPadding(ui.dp(20), 0, ui.dp(20), 0) }
        fun field(
            hint: String,
            initial: String = "",
            type: Int = InputType.TYPE_CLASS_TEXT,
        ): EditText {
            form.addView(ui.text(hint, 13, ui.muted))
            return EditText(this).apply {
                this.hint = hint
                inputType = type
                isSingleLine = true
                setText(initial)
                form.addView(this)
            }
        }
        val host = field("Hostname")
        val user = field("SSH username")
        val session = field("Herdr session", "default")
        val port = field("SSH port", "22", InputType.TYPE_CLASS_NUMBER)
        val dlg =
            AlertDialog.Builder(this)
                .setTitle("Add host")
                .setView(ScrollView(this).apply { addView(form) })
                .setNegativeButton("Cancel", null)
                .setPositiveButton("Connect", null)
                .create()
        dlg.show()
        dlg.getButton(AlertDialog.BUTTON_POSITIVE).setOnClickListener {
            val raw = host.text.toString().trim()
            val profile =
                HostProfile(
                    raw.removeSurrounding("[", "]"),
                    port.text.toString().toIntOrNull() ?: 0,
                    user.text.toString().trim(),
                    session.text.toString().trim(),
                )
            val error = profile.validationError()
            if (error != null) {
                toast(error)
                return@setOnClickListener
            }
            phone.hosts.save(profile)
            dlg.dismiss()
            open(profile)
        }
    }

    private fun confirmArrival(c: SessionController, host: HostProfile, view: MobileView): Boolean {
        val wanted = phone.pending ?: return true
        val pane = wanted.paneIn(view)
        val revision = phone.revision
        if (phone.routeError == null) {
            if (pane == null) {
                phone.routeError = "This agent is no longer in the observed pane."
            } else if (!phone.routed) {
                phone.routed = true
                phone.revision = c.goTo(pane, view)
                if (phone.revision == null)
                    phone.routeError = c.message ?: "Open the session and select this agent again."
            } else if (revision != null && view.version >= revision && !view.goToPending) {
                if (wanted.hasArrived(view, revision)) {
                    phone.pending = null
                    return true
                }
                phone.routeError =
                    view.error
                        ?: "Herdr could not confirm this agent. Open its session to choose a live pane."
            }
        }
        if (phone.routeError != null) {
            observe(host, c)
            drawStatus(
                "Agent changed",
                phone.routeError.orEmpty(),
                listOf("Back to agents" to ::goHome),
            )
        } else drawStatus("Opening agent", wanted.headline, listOf("Cancel" to ::goHome))
        return false
    }

    private fun render() {
        val c = active ?: return
        val host = phone.selected ?: return
        if (c.phase != ConnectionPhase.Connected && picker != null) picker?.dismiss()
        when (c.phase) {
            ConnectionPhase.Connected -> {
                val view = c.view ?: return
                if (!confirmArrival(c, host, view)) return
                if (page != "terminal") {
                    clear("terminal")
                    val bar = ui.line()
                    homeButton = ui.button("Home", ::goHome)
                    bar.addView(homeButton)
                    terminalBar =
                        ui.text(host.session, 16, ui.ink, true).apply {
                            gravity = Gravity.CENTER
                            maxLines = 1
                        }
                    bar.addView(terminalBar, LinearLayout.LayoutParams(0, -2, 1f))
                    bar.addView(ui.button("Session") { sessionMenu(c, host) })
                    root.addView(bar)
                    terminal = TerminalScreen(this, c) { openPicker(false) }
                    root.addView(terminal, LinearLayout.LayoutParams(-1, 0, 1f))
                }
                terminalBar?.text =
                    view.panes
                        .firstOrNull { it.id == view.focusedPane }
                        ?.let { it.terminalTitle ?: it.agentName ?: it.label } ?: host.session
                val elsewhere = view.panes.any { it.agentName != null && it.status == "blocked" && it.id != view.focusedPane }
                homeButton?.apply {
                    text = if (elsewhere) "● Home" else "Home"
                    contentDescription = if (elsewhere) "Home, an agent in another pane needs you" else "Home"
                }
                terminal?.render(view)
                picker?.update(view)
                if (phone.toWorkspaces) {
                    phone.toWorkspaces = false
                    openPicker(true)
                }
                c.message
                    ?.takeUnless { it == lastNotice }
                    ?.let {
                        lastNotice = it
                        toast(it)
                    }
            }
            ConnectionPhase.Trust -> {
                val fp = c.fingerprint.orEmpty()
                drawStatus(
                    "Verify host key",
                    "Compare this fingerprint with the SSH host before trusting it.\n\n${host.destination}\n\n$fp",
                    listOf("Trust and connect" to { c.trustHost() }, "Cancel" to ::goHome),
                )
            }
            ConnectionPhase.Password -> {
                if (page != "password") {
                    clear("password")
                    val form = statusContent("SSH password", host.destination)
                    password =
                        EditText(this).apply {
                            hint = "Password"
                            inputType =
                                InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_VARIATION_PASSWORD
                            isSingleLine = true
                            isSaveEnabled = false
                            form.addView(this)
                        }
                    val authorize = CheckBox(this).apply {
                        text = getString(R.string.sign_in_authorize)
                        setTextColor(ui.ink)
                        minHeight = ui.dp(48)
                        isChecked = Build.VERSION.SDK_INT >= 28 && c.keyHardwareBacked != false && !c.keyEnrolled
                        isEnabled = isChecked
                    }
                    form.addView(authorize)
                    fun submit() {
                        val entered = password?.text?.toString().orEmpty()
                        password?.text?.clear()
                        c.login(entered, authorize.isChecked)
                    }
                    password?.setOnEditorActionListener { _, _, _ ->
                        submit()
                        true
                    }
                    form.addView(ui.button("Sign in") { submit() })
                    form.addView(ui.button("Use phone key") { c.loginWithKey() })
                    form.addView(ui.button("Phone key") { phoneKey(c) })
                    form.addView(ui.button("Cancel", ::goHome))
                }
            }
            ConnectionPhase.Failed -> {
                val actions = mutableListOf<Pair<String, () -> Unit>>()
                if (c.keyChanged)
                    actions.add(
                        "Review changed key" to
                            {
                                val challenged = c.fingerprint ?: return@to
                                AlertDialog.Builder(this)
                                    .setTitle("Forget trusted host key?")
                                    .setMessage(
                                        "Confirm the key change with the host administrator first. You will need to compare the new fingerprint."
                                    )
                                    .setNegativeButton("Cancel", null)
                                    .setPositiveButton("Forget key") { _, _ ->
                                        if (
                                            active !== c ||
                                                c.phase != ConnectionPhase.Failed ||
                                                !c.keyChanged ||
                                                c.fingerprint != challenged
                                        )
                                            return@setPositiveButton
                                        phone.sessions
                                            .filter { (p, _) ->
                                                p != host && p.endpoint == host.endpoint
                                            }
                                            .keys
                                            .toList()
                                            .forEach { p ->
                                                phone.sessions.remove(p)?.let {
                                                    it.changed = null
                                                    it.close()
                                                }
                                                phone.previews.remove(p)
                                            }
                                        c.forgetHostKey()
                                    }
                                    .show()
                            }
                    )
                else {
                    actions.add("Reconnect" to { c.connect(host) })
                    actions.add("Use password" to { c.usePassword() })
                }
                actions.add("Back to agents" to ::goHome)
                drawStatus("Connection closed", c.message.orEmpty(), actions)
            }
            else ->
                drawStatus(
                    "Connecting",
                    host.destination + " · " + host.session,
                    listOf("Back to agents" to ::goHome),
                )
        }
    }

    private fun statusContent(title: String, detail: String): LinearLayout {
        val col = ui.column().apply { setPadding(ui.dp(24), ui.dp(48), ui.dp(24), ui.dp(20)) }
        root.addView(ScrollView(this).apply { addView(col) })
        col.addView(ui.text(title, 26, ui.ink, true))
        col.addView(
            ui.text(detail, 16, ui.muted).apply {
                setTextIsSelectable(true)
                setPadding(0, ui.dp(16), 0, ui.dp(20))
            }
        )
        return col
    }

    private fun drawStatus(title: String, detail: String, actions: List<Pair<String, () -> Unit>>) {
        val key = title + detail
        if (page == key) return
        clear(key)
        val col = statusContent(title, detail)
        actions.forEach { (label, fn) -> col.addView(ui.button(label, fn)) }
    }

    private fun openPicker(workspaces: Boolean) {
        if (picker != null) return
        val host = phone.selected ?: return
        val c = active ?: return
        terminal?.suspendKeyboardForPicker()
        picker =
            AgentPicker(this, c, host, phone.previews.getValue(host)) {
                picker = null
                terminal?.restoreKeyboard()
            }
        picker?.show(workspaces)
    }

    private fun sessionMenu(c: SessionController, host: HostProfile) {
        val options = mutableListOf("Workspaces", "Phone key", "Disconnect")
        AlertDialog.Builder(this)
            .setTitle(host.destination)
            .setItems(options.toTypedArray()) { _, i ->
                when (i) {
                    0 -> openPicker(true)
                    1 -> phoneKey(c)
                    2 -> {
                        goHome()
                        phone.sessions.remove(host)
                        phone.previews.remove(host)
                        c.changed = null
                        c.close()
                    }
                }
            }
            .show()
    }

    private fun phoneKey(c: SessionController) {
        c.loadPublicKey()
        val dialog =
            AlertDialog.Builder(this)
                .setTitle("Phone key")
                .setMessage("Looking up this phone’s SSH public key…")
                .setNeutralButton("Copy public key", null)
                .setNegativeButton("Done", null)
        if (c.canEnroll)
            dialog.setPositiveButton("Install key") { _, _ ->
                if (active === c && c.canEnroll) c.enrollKey()
            }
        val shown = dialog.show()
        val copy = shown.getButton(AlertDialog.BUTTON_NEUTRAL)
        refreshPhoneKey = refresh@{
            if (!shown.isShowing || isDestroyed) return@refresh
            if (active !== c) {
                shown.dismiss()
                return@refresh
            }
            val public = c.publicKey
            copy.isEnabled = public != null
            shown.getButton(AlertDialog.BUTTON_POSITIVE)?.isEnabled = public != null && c.canEnroll
            when {
                public != null ->
                    shown.setMessage(
                        (if (c.keyEnrolled) "This account has verified this phone’s key."
                        else "Authorize this public key on your SSH host.") + "\n\n" + public
                    )
                c.keyHardwareBacked == false ->
                    shown.setMessage(c.message ?: "Hardware-backed key storage is unavailable.")
                c.message != null -> shown.setMessage(c.message)
            }
        }
        copy.setOnClickListener {
            c.publicKey?.let { key ->
                getSystemService(ClipboardManager::class.java)
                    .setPrimaryClip(ClipData.newPlainText("SSH public key", key))
                toast("Public key copied")
                shown.dismiss()
            }
        }
        shown.setOnDismissListener { refreshPhoneKey = null }
        refreshPhoneKey?.invoke()
    }

    override fun toast(text: String) = Toast.makeText(this, text, Toast.LENGTH_LONG).show()

    override val hosts get() = phone.hosts

    override fun go(page: Page) {
        phone.pages.add(page)
        showPage(page)
    }

    override fun back() {
        phone.pages.removeLastOrNull()
        showHome()
    }

    override fun redraw() = showHome()

    override fun open(profile: HostProfile) {
        phone.pages.clear()
        open(profile, null)
    }

    override fun forget(profiles: Collection<HostProfile>) = profiles.forEach { host ->
        phone.sessions.remove(host)?.let {
            it.changed = null
            it.close()
        }
        phone.previews.remove(host)
        phone.catalog.forget(host)
    }

    override fun closeEndpoint(endpoint: String) =
        phone.sessions.keys.filter { it.endpoint == endpoint }.forEach { host ->
            phone.sessions.remove(host)?.let {
                it.changed = null
                it.close()
            }
            phone.previews.remove(host)
        }

    override fun <T> background(work: () -> T, done: (T) -> Unit) {
        val token = pageToken
        phone.work.execute {
            val result = work()
            handler.post { if (token == pageToken && !isDestroyed) done(result) }
        }
    }

    private fun showPage(target: Page) {
        val group = when (target) {
            is Page.InstallKey -> group(target.account)
            is Page.HostDetail -> group(target.account)
            else -> null
        }
        if ((target is Page.InstallKey || target is Page.HostDetail) && group == null) return back()
        clear("page")
        val title = when (target) {
            Page.Settings -> "Settings"
            Page.ToolbarKeys -> "Toolbar keys"
            is Page.InstallKey -> "Install phone key"
            is Page.HostDetail -> phone.hosts.name(target.account) ?: group?.destination.orEmpty()
        }
        val bar = ui.line().apply { setPadding(ui.dp(4), ui.dp(8), ui.dp(8), ui.dp(4)) }
        bar.addView(ui.button("‹ Back", ::back).apply { contentDescription = "Back" })
        bar.addView(ui.text(title, 20, ui.ink, true).apply { maxLines = 1 }, LinearLayout.LayoutParams(0, -2, 1f))
        root.addView(bar)
        val body = ui.column().apply { setPadding(ui.dp(14), 0, ui.dp(14), ui.dp(24)) }
        root.addView(ScrollView(this).apply { addView(body) })
        when (target) {
            Page.Settings -> SettingsScreen(this, ui, this).build(body)
            Page.ToolbarKeys -> ToolbarKeyEditor(this, ui, this).build(body)
            is Page.InstallKey -> install = KeyInstallScreen(this, ui, this, group!!).also { it.build(body) }
            is Page.HostDetail -> HostDetailScreen(this, ui, this, group!!).build(body)
        }
    }
}
