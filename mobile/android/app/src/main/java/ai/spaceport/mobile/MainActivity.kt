// ABOUTME: Connects the Android host catalog and agent picker to retained Herdr sessions.
// ABOUTME: Keeps terminal input hidden until the selected agent and surface agree.
package ai.spaceport.mobile

import android.app.AlertDialog
import android.app.Application
import android.content.Intent
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
import androidx.core.net.toUri
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
    var pickingFileFor: Pair<SessionDraft, uniffi.spaceport_mobile.InputTarget>? = null
    var toWorkspaces = false
    var nextWaiting = false
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
    private var createButton: Button? = null
    private var picker: AgentPicker? = null
    private var page = ""
    private var password: EditText? = null
    private var hostEditor: AlertDialog? = null
    private var refreshPhoneKey: (() -> Unit)? = null
    private var lastNotice: String? = null
    private var pageToken = 0L
    private var foreground = false
    private var updateHomeRefresh: (() -> Unit)? = null
    private lateinit var rosterRefresh: HomeRosterRefresh
    private var install: KeyInstallScreen? = null
    private val handler = Handler(Looper.getMainLooper())
    private val refresh = Runnable { if (page == "home" && !isFinishing) showHome() }
    private val active
        get() = phone.sessions[phone.selected]

    private val filePicker = registerForActivityResult(androidx.activity.result.contract.ActivityResultContracts.OpenDocument()) { uri ->
        val owner = phone.pickingFileFor
        phone.pickingFileFor = null
        if (uri != null && owner != null) {
            val (draft, target) = owner
            phone.work.execute {
                val result = runCatching { DraftFile.read(applicationContext, uri) }
                runOnUiThread {
                    if (draft.destination == target) result.onSuccess { draft.file = it; draft.png = null }
                    if (!isDestroyed) {
                        terminal?.refreshDraft()
                        Toast.makeText(this, result.exceptionOrNull()?.message ?: "Check your attachment in Compose.", Toast.LENGTH_SHORT).show()
                    }
                }
            }
        }
    }

    internal fun pickDraftFile(draft: SessionDraft) {
        val target = draft.destination ?: return
        phone.pickingFileFor = draft to target
        filePicker.launch(arrayOf("*/*"))
    }

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        phone = ViewModelProvider(this)[RetainedPhone::class.java]
        rosterRefresh = HomeRosterRefresh(phone.hosts, phone.catalog, phone.sessions, phone.work) {
            if (foreground && page == "home" && phone.selected == null) updateHomeRefresh?.invoke()
        }
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
        publishAgentShortcuts(this)
        if (savedInstanceState == null) launchShortcut(intent)
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        launchShortcut(intent)
    }

    private fun launchShortcut(intent: Intent) {
        val action = LauncherAction.from(this, intent) ?: return
        val widget = intent.data?.schemeSpecificPart
        if (widget == null) getSystemService(android.content.pm.ShortcutManager::class.java).reportShortcutUsed(action.id)
        val host = if (widget == null) phone.selected ?: phone.hosts.lastOpened() else phone.hosts.lastOpened(widget)
        if (host == null || host !in phone.hosts.profiles()) {
            toast(if (widget == null) "Open a saved session first." else "This widget’s session has changed. Choose a saved session.")
            return
        }
        if (phone.pending != null || page !in setOf("home", "terminal") || hostEditor != null || picker != null || terminal?.isEditing == true) {
            toast("Finish the current action, then try the shortcut again.")
            return
        }
        if (phone.selected == host && page == "terminal" && action == LauncherAction.RESUME) return
        if (phone.selected != null) goHome()
        open(host, nextWaiting = action == LauncherAction.NEXT)
    }

    override fun onStart() {
        super.onStart()
        foreground = true
        active?.let { if (it.phase == ConnectionPhase.Parked) it.resume() }
        // Pages keep their own state across a trip to the background; only Home refreshes.
        if (phone.selected == null && page == "home" && hostEditor == null) rosterRefresh.start()
    }

    override fun onStop() {
        foreground = false
        rosterRefresh.cancel()
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
        phone.catalog.observe(host, view)
    }

    private fun open(host: HostProfile, agent: AgentRow? = null, workspaces: Boolean = false, nextWaiting: Boolean = false) {
        rosterRefresh.cancel()
        handler.removeCallbacks(refresh)
        phone.selected = host
        phone.pending = agent
        phone.routed = false
        phone.revision = null
        phone.routeError = null
        phone.toWorkspaces = workspaces
        phone.nextWaiting = nextWaiting
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
        updateHomeRefresh = null
        install?.dispose()
        install = null
        if (page == "terminal") terminal?.dispose()
        terminal = null
        terminalBar = null
        homeButton = null
        createButton = null
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
        phone.nextWaiting = false
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

    private var reviewOnly = false
    private var showQuiet = false

    private fun showHome() {
        phone.pages.lastOrNull()?.let { return showPage(it) }
        val entering = page != "home"
        val previousScroll = if (entering) 0 else (root.getChildAt(1) as? ScrollView)?.scrollY ?: 0
        clear("home")
        val allProfiles = phone.hosts.profiles()
        val tags = phone.hosts.availableTags(allProfiles)
        val selected = phone.hosts.selectedTag
        if (selected.isNotEmpty() && tags.none { it.equals(selected, true) }) phone.hosts.selectedTag = ""
        val profiles = phone.hosts.filtered(phone.hosts.selectedTag, allProfiles)
        val tiles = profiles.flatMap(phone.catalog::rows)
        val now = System.currentTimeMillis()
        val header = ui.line().apply { setPadding(ui.dp(18), ui.dp(8), ui.dp(8), ui.dp(4)) }
        header.addView(ui.text("Agents", 30, ui.ink, true), LinearLayout.LayoutParams(0, -2, 1f))
        val spinner = ProgressBar(this)
        header.addView(spinner, LinearLayout.LayoutParams(ui.dp(24), ui.dp(24)))
        val sessionLabels = mutableMapOf<HostProfile, TextView>()
        header.addView(ui.button("Settings") { go(Page.Settings) })
        header.addView(ui.button("Add host") { editHost() })
        root.addView(header)
        val body = ui.column().apply { setPadding(ui.dp(14), 0, ui.dp(14), ui.dp(24)) }
        root.addView(ScrollView(this).apply {
            addView(body)
            if (previousScroll != 0) post { scrollTo(0, previousScroll) }
        })
        if (tags.isNotEmpty()) {
            val choices = listOf("All hosts") + tags
            body.addView(ui.button(phone.hosts.selectedTag.ifEmpty { "All hosts" }) {
                AlertDialog.Builder(this).setTitle("Filter hosts by tag")
                    .setItems(choices.toTypedArray()) { _, index ->
                        phone.hosts.selectedTag = if (index == 0) "" else choices[index]
                        showHome()
                    }.show()
            })
        }
        if (tiles.any { it.status == "done" } || reviewOnly) {
            body.addView(ui.button(if (reviewOnly) "Show all agents" else "Finished for review") {
                reviewOnly = !reviewOnly
                showHome()
            })
        }
        val visible = AgentBoard.sorted(tiles, phone.catalog.marks, now)
            .filter { !reviewOnly || (it.status == "done" && phone.catalog.marks[it.key]?.settled != true &&
                AgentBoard.section(it, phone.catalog.marks[it.key], now) != AgentSection.SNOOZED) }
        if (visible.isEmpty()) {
            body.addView(
                ui.text(
                        if (reviewOnly) "No agents have finished work to review."
                        else if (profiles.isEmpty()) "Connect an SSH host to see its Herdr agents."
                        else "Open a session to see its agents here.",
                        16,
                        ui.muted,
                    )
                    .apply { setPadding(ui.dp(4), ui.dp(24), ui.dp(4), ui.dp(16)) }
            )
        }
        visible
            .groupBy { AgentBoard.section(it, phone.catalog.marks[it.key], now) }
            .forEach { (section, rows) ->
                if (section == AgentSection.QUIET) {
                    body.addView(ui.button("Quiet (${rows.size}) · " + if (showQuiet) "Hide" else "Show") {
                        showQuiet = !showQuiet
                        showHome()
                    })
                    if (!showQuiet) return@forEach
                } else ui.heading(body, section.label)
                rows.forEach { row ->
                    val details = buildList {
                        add(row.path)
                        add(observationLabel(row.seenAt, now))
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
                        brand = row,
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
                    val row = ui.row(body, host.session, "Saved session",
                        action = { open(host) }, held = { go(Page.HostDetail(group.account)) })
                    sessionLabels[host] = row.findViewById(android.R.id.text2)
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
        fun displayed(rows: List<AgentRow>, clock: Long) = rows.map { row ->
            Triple(row.copy(seenAt = null), observationLabel(row.seenAt, clock),
                AgentBoard.section(row, phone.catalog.marks[row.key], clock))
        }
        val drawn = displayed(tiles, now)
        updateHomeRefresh = {
            val checking = rosterRefresh.checking
            spinner.visibility = if (checking == null) android.view.View.GONE else android.view.View.VISIBLE
            spinner.contentDescription = checking?.let { "Checking agents on ${it.hostname}" }
            for ((host, label) in sessionLabels) {
                label.text = when {
                    host in rosterRefresh.failed -> "Last check unavailable"
                    host in phone.sessions -> "Connected"
                    else -> "Saved session"
                }
            }
            // Keep the active touch and accessible row alive when only the check indicator changes.
            if (displayed(profiles.flatMap(phone.catalog::rows), System.currentTimeMillis()) != drawn) showHome()
        }
        updateHomeRefresh?.invoke()
        if (foreground && entering) rosterRefresh.start()
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
            if (row.status == "done") add((if (mark.settled) "Return to review" else "Mark reviewed") to mark.copy(settled = !mark.settled))
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
        rosterRefresh.cancel()
        showHome()
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
        hostEditor = dlg
        dlg.setOnDismissListener {
            hostEditor = null
            if (foreground && page == "home" && phone.selected == null) rosterRefresh.start()
        }
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
            open(profile)
            dlg.dismiss()
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
                if (phone.nextWaiting && view.inputTarget != null && c.sessionFor(view) != null) {
                    phone.nextWaiting = false
                    val waiting = AgentBoard.rows(host, view).filter { it.name != null && it.status == "blocked" }
                    val focused = waiting.indexOfFirst { it.id == view.focusedPane }
                    if (waiting.isEmpty()) toast("No agents are waiting in this session.")
                    else {
                        val next = waiting[(focused + 1) % waiting.size]
                        if (next.id != view.focusedPane) phone.pending = next
                    }
                }
                if (!confirmArrival(c, host, view)) return
                if (page != "terminal") {
                    clear("terminal")
                    val bar = ui.line()
                    homeButton = ui.button("Home", ::goHome)
                    bar.addView(homeButton)
                    createButton = ui.button("+") { }.apply { contentDescription = "New terminal" }
                    bar.addView(createButton)
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
                createButton?.apply {
                    isEnabled = c.sessionFor(view) != null && view.inputTarget != null &&
                        view.focusedWorkspace != null && view.focusedTab != null
                    setOnClickListener {
                        isEnabled = false
                        c.createTab(view)
                    }
                }
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
                actions.add("Copy connection diagnostics" to { copyDiagnostics(c, host) })
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

    private fun copyDiagnostics(c: SessionController, host: HostProfile) {
        val info = packageManager.getPackageInfo(packageName, 0)
        @Suppress("DEPRECATION")
        val build = if (Build.VERSION.SDK_INT >= 28) info.longVersionCode else info.versionCode.toLong()
        val observed = phone.catalog.observedAt(host)
        getSystemService(ClipboardManager::class.java).setPrimaryClip(
            ClipData.newPlainText("Spaceport diagnostics", c.diagnosticReport(build, observed)))
        toast("Copied connection diagnostics")
    }

    private fun openServicePreview(c: SessionController) {
        val port = EditText(this).apply {
            hint = "Remote port"
            inputType = InputType.TYPE_CLASS_NUMBER
        }
        val dialog = AlertDialog.Builder(this).setTitle("Preview loopback service").setView(port)
            .setMessage("Open one HTTP port on this SSH host in your browser.")
            .setNegativeButton("Cancel", null).setPositiveButton("Open", null).create()
        dialog.show()
        dialog.getButton(AlertDialog.BUTTON_POSITIVE).setOnClickListener {
            val remote = port.text.toString().toIntOrNull()
            if (remote == null || remote !in 1..65535) {
                port.error = "Enter a port from 1 to 65535"
                return@setOnClickListener
            }
            if (active !== c || c.phase != ConnectionPhase.Connected) {
                dialog.dismiss()
                return@setOnClickListener
            }
            try {
                val local = c.startServicePreview(remote)
                startActivity(android.content.Intent(android.content.Intent.ACTION_VIEW,
                    "http://127.0.0.1:$local/".toUri()))
                dialog.dismiss()
            } catch (e: Exception) {
                c.stopServicePreview(::toast)
                toast(e.localizedMessage ?: "Could not open the browser")
            }
        }
    }

    private fun sessionMenu(c: SessionController, host: HostProfile) {
        val options = mutableListOf("Workspaces", "Phone key", "Disconnect", "Copy connection diagnostics")
        if (applicationInfo.flags and android.content.pm.ApplicationInfo.FLAG_DEBUGGABLE != 0) {
            options.add(if (c.previewPort == null) "Preview loopback service" else "Stop service preview")
        }
        AlertDialog.Builder(this)
            .setTitle(host.destination)
            .setItems(options.toTypedArray()) { _, i ->
                when (i) {
                    4 -> if (c.previewPort == null) openServicePreview(c) else c.stopServicePreview(::toast)
                    3 -> copyDiagnostics(c, host)
                    0 -> openPicker(true)
                    1 -> phoneKey(c)
                    2 -> {
                        phone.hosts.allowHomeObservation(host, false)
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
        rosterRefresh.cancel()
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

    override fun closeEndpoint(endpoint: String) {
        forget((phone.hosts.profiles() + phone.sessions.keys).filter { it.endpoint == endpoint }.distinct())
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
