// ABOUTME: Shows one SSH account's address, trusted host key, phone key and saved Herdr sessions.
// ABOUTME: Confirms before forgetting a host key or removing sessions; Herdr keeps running either way.
package ai.spaceport.mobile

import android.app.Activity
import android.app.AlertDialog
import android.graphics.Typeface
import android.text.InputType
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.ScrollView

internal class HostDetailScreen(
    private val activity: Activity,
    private val ui: PhoneUi,
    private val pages: Pages,
    private val group: HostGroup,
) {
    private val first = group.first

    fun build(body: LinearLayout) {
        ui.heading(body, "Connection")
        detail(body, "Host", first.hostname)
        detail(body, "Account", first.username)
        detail(body, "Port", first.port.toString())
        ui.row(body, "Edit host", "Address, port, account and name", action = ::edit)
        val enrolled = pages.hosts.enrolled(first)
        ui.row(body, if (enrolled) "Phone key verified" else "Install phone key",
            if (enrolled) "Signs in without a password" else "Use the SSH password once",
            action = { pages.go(Page.InstallKey(group.account)) })

        ui.heading(body, "Trusted host key")
        val pin = pages.hosts.pin(first)
        if (pin == null) {
            body.addView(ui.text("No trusted key saved", 16, ui.muted).apply { setPadding(ui.dp(3), 0, 0, 0) })
        } else {
            body.addView(ui.text(pin, 13, ui.ink).apply {
                typeface = Typeface.MONOSPACE
                setTextIsSelectable(true)
                setPadding(ui.dp(3), 0, ui.dp(3), ui.dp(4))
            })
            body.addView(ui.text("Verified on this phone. This key applies to every account on this host and port.",
                13, ui.muted).apply { setPadding(ui.dp(3), 0, ui.dp(3), 0) })
            body.addView(ui.button("Forget host key…") { forget(pin) })
        }

        ui.heading(body, "Tags")
        val tags = pages.hosts.tags(group.account)
        ui.row(body, "Edit tags", tags.joinToString(", "), action = {
            val input = EditText(activity).apply { setSingleLine(); setText(tags.joinToString(", ")) }
            AlertDialog.Builder(activity).setTitle("Host tags")
                .setMessage("Enter up to eight tags, separated by commas.").setView(input)
                .setNegativeButton("Cancel", null).setPositiveButton("Save") { _, _ ->
                    pages.hosts.tag(group.account, input.text.toString())
                    pages.redraw()
                }.show()
        })

        ui.heading(body, "Herdr sessions")
        for (session in group.sessions) {
            ui.row(body, session.session, "Hold to rename or remove", action = { pages.open(session) }, held = { manage(session) })
        }
        body.addView(ui.button("Add session") { ui.askSessionName("Add session", "") { name -> save(emptyList(), first.copy(session = name)) } })
        body.addView(ui.button("Remove host…") {
            confirm("Remove ${title()} from this phone?",
                "This removes its saved sessions from this phone. Herdr keeps running on the host.", "Remove host") {
                pages.hosts.replace(group.sessions, emptyList())
                pages.hosts.rename(group.account, "")
                pages.forget(group.sessions)
                pages.back()
            }
        })
    }

    fun title() = pages.hosts.name(group.account) ?: group.destination

    private fun detail(body: LinearLayout, label: String, value: String) {
        val line = ui.line().apply { minimumHeight = ui.dp(40) }
        line.addView(ui.text(label, 16, ui.muted), LinearLayout.LayoutParams(0, -2, 1f))
        line.addView(ui.text(value, 16).apply { setTextIsSelectable(true) })
        body.addView(line)
    }

    private fun manage(session: HostProfile) {
        AlertDialog.Builder(activity).setTitle(session.session)
            .setItems(arrayOf("Rename", "Remove")) { _, i ->
                if (i == 0) ui.askSessionName("Rename session", session.session) { name -> save(listOf(session), session.copy(session = name)) }
                else confirm("Remove ${session.session} from this phone?", "Its Herdr session keeps running on the host.", "Remove") {
                    pages.hosts.replace(listOf(session), emptyList())
                    pages.forget(listOf(session))
                    if (group.sessions.size == 1) pages.back() else pages.redraw()
                }
            }
            .show()
    }

    private fun save(old: List<HostProfile>, new: HostProfile) {
        if (pages.addSession(old, new)) pages.redraw()
    }

    private fun edit() {
        val form = ui.column().apply { setPadding(ui.dp(20), 0, ui.dp(20), 0) }
        fun field(hint: String, value: String, type: Int = InputType.TYPE_CLASS_TEXT) = EditText(activity).apply {
            this.hint = hint
            inputType = type
            isSingleLine = true
            setText(value)
            form.addView(ui.text(hint, 13, ui.muted))
            form.addView(this)
        }
        val name = field("Name (optional)", pages.hosts.name(group.account).orEmpty())
        val host = field("Hostname", first.hostname)
        val user = field("SSH username", first.username)
        val port = field("SSH port", first.port.toString(), InputType.TYPE_CLASS_NUMBER)
        val dialog = AlertDialog.Builder(activity).setTitle("Edit host")
            .setView(ScrollView(activity).apply { addView(form) })
            .setNegativeButton("Cancel", null).setPositiveButton("Save", null).create()
        dialog.show()
        dialog.getButton(AlertDialog.BUTTON_POSITIVE).setOnClickListener {
            val moved = group.movedTo(host.text.toString().trim().removeSurrounding("[", "]"),
                port.text.toString().toIntOrNull() ?: 0, user.text.toString().trim())
            moved.firstNotNullOfOrNull { it.validationError() }?.let { return@setOnClickListener pages.toast(it) }
            val account = moved.first().account
            // Moving onto another saved host's account would silently merge both groups and relabel it.
            if (account != group.account && pages.hosts.profiles().any { it.account == account })
                return@setOnClickListener pages.toast("Another saved host already uses ${moved.first().destination}")
            if (moved != group.sessions) {
                val tags = pages.hosts.tags(group.account)
                pages.hosts.replace(group.sessions, moved)
                if (account != group.account) pages.hosts.tag(account, tags.joinToString(","))
                pages.forget(group.sessions)
            }
            if (account != group.account) pages.hosts.rename(group.account, "")
            pages.hosts.rename(account, name.text.toString())
            dialog.dismiss()
            if (account == group.account) pages.redraw()
            else {
                pages.back()
                pages.go(Page.HostDetail(account))
            }
        }
    }

    private fun forget(pin: String) =
        confirm("Forget the trusted key for ${first.endpoint}?", "This clears cached agents and marks for this host. Verify its key again before connecting.",
            "Forget host key") {
            if (pages.hosts.forgetPin(first, pin)) pages.closeEndpoint(first.endpoint)
            else pages.toast("The saved host key changed. Review it again.")
            pages.redraw()
        }

    private fun confirm(title: String, message: String, action: String, done: () -> Unit) {
        AlertDialog.Builder(activity).setTitle(title).setMessage(message)
            .setNegativeButton("Cancel", null).setPositiveButton(action) { _, _ -> done() }.show()
    }
}
