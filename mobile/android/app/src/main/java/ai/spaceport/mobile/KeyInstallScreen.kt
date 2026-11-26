// ABOUTME: Collects an SSH password once to install this phone's key on a saved account.
// ABOUTME: Shows an unknown fingerprint for explicit trust and never keeps the password after leaving.
package ai.spaceport.mobile

import android.app.Activity
import android.text.InputType
import android.widget.EditText
import android.widget.LinearLayout

internal class KeyInstallScreen(
    private val activity: Activity,
    private val ui: PhoneUi,
    private val pages: Pages,
    private val group: HostGroup,
) {
    private val installer = KeyInstaller(pages.hosts)
    private var password: EditText? = null

    fun build(body: LinearLayout) {
        body.addView(ui.text(group.destination, 20, ui.ink, true).apply { setPadding(ui.dp(3), ui.dp(12), 0, ui.dp(8)) })
        if (pages.hosts.enrolled(group.first)) {
            body.addView(ui.text("This account has verified this phone’s key. Spaceport signs in with it.", 16, ui.muted))
            return
        }
        val field = EditText(activity).apply {
            hint = "SSH password"
            inputType = InputType.TYPE_CLASS_TEXT or InputType.TYPE_TEXT_VARIATION_PASSWORD
            isSingleLine = true
            isSaveEnabled = false
        }
        password = field
        val status = ui.text("", 15, ui.muted)
        val install = ui.button("Install key") {}
        fun run(trusting: String?) {
            val secret = field.text.toString()
            if (secret.isEmpty()) return
            field.isEnabled = false
            install.isEnabled = false
            status.text = activity.getString(R.string.install_checking)
            pages.background({ installer.install(group.first, secret, trusting) }) { outcome ->
                field.isEnabled = true
                install.isEnabled = true
                when (outcome) {
                    InstallOutcome.Installed -> {
                        field.text.clear()
                        pages.toast("Key installed and verified")
                        pages.redraw()
                    }
                    is InstallOutcome.Challenge ->
                        if (outcome.changed) {
                            field.text.clear()
                            status.text = activity.getString(R.string.install_changed_host)
                        } else {
                            status.text = ""
                            trust(outcome.fingerprint) { run(it) }
                        }
                    is InstallOutcome.Failed -> status.text = outcome.message
                }
            }
        }
        install.setOnClickListener { run(null) }
        field.setOnEditorActionListener { _, _, _ -> run(null); true }
        body.addView(field)
        body.addView(install)
        body.addView(status)
        body.addView(ui.text(
            "Spaceport uses this password once to add the phone’s public key to authorized_keys. " +
                "It then checks a fresh SSH login that uses only the key. Herdr does not start.",
            13, ui.muted).apply { setPadding(ui.dp(3), ui.dp(12), ui.dp(3), 0) })
    }

    private fun trust(fingerprint: String, accept: (String) -> Unit) {
        android.app.AlertDialog.Builder(activity)
            .setTitle("Verify host key")
            .setMessage("Compare this fingerprint with the SSH host before trusting it.\n\n${group.first.endpoint}\n\n$fingerprint")
            .setNegativeButton("Cancel") { _, _ -> password?.text?.clear() }
            .setPositiveButton("Trust host") { _, _ -> accept(fingerprint) }
            .setCancelable(false)
            .show()
    }

    /** The page is leaving; drop the typed password. */
    fun dispose() {
        password?.text?.clear()
        password = null
    }
}
