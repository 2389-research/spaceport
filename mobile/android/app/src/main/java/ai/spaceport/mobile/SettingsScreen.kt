// ABOUTME: Shows the phone key, terminal preferences and app version, and edits toolbar keys.
// ABOUTME: Writes the same preferences the live terminal and Compose read.
package ai.spaceport.mobile

import android.app.Activity
import android.app.AlertDialog
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Intent
import android.graphics.Typeface
import android.os.Build
import android.widget.CheckBox
import android.widget.LinearLayout
import android.widget.TextView
import uniffi.spaceport_mobile.enclavePublicKey

internal class SettingsScreen(private val activity: Activity, private val ui: PhoneUi, private val pages: Pages) {
    private val preferences = TerminalPreferences(activity)

    fun build(body: LinearLayout) {
        phoneKey(body)
        ui.heading(body, "Terminal")
        val size = ui.line().apply { minimumHeight = ui.dp(52) }
        size.addView(ui.text("Text size", 17), LinearLayout.LayoutParams(0, -2, 1f))
        val value = ui.text(activity.getString(R.string.settings_text_size, preferences.textSize), 16, ui.muted)
        fun step(by: Int) {
            preferences.textSize += by
            value.text = activity.getString(R.string.settings_text_size, preferences.textSize)
        }
        size.addView(ui.button("−") { step(-1) }.apply { contentDescription = "Smaller text" })
        size.addView(value)
        size.addView(ui.button("+") { step(1) }.apply { contentDescription = "Larger text" })
        body.addView(size)
        val keys = preferences.keys.keys.size
        ui.row(body, "Toolbar keys", if (keys == 1) "1 key" else "$keys keys", action = { pages.go(Page.ToolbarKeys) })
        body.addView(CheckBox(activity).apply {
            setText(R.string.terminal_compose_return)
            setTextColor(ui.ink)
            minHeight = ui.dp(48)
            isChecked = preferences.pressReturn
            setOnCheckedChangeListener { _, checked -> preferences.pressReturn = checked }
        })
        footer(body, "Hold the terminal and drag to move the cursor. Hold still to copy or paste.")
        ui.heading(body, "About")
        val info = activity.packageManager.getPackageInfo(activity.packageName, 0)
        @Suppress("DEPRECATION")
        val code = if (Build.VERSION.SDK_INT >= 28) info.longVersionCode else info.versionCode.toLong()
        body.addView(ui.text("Version ${info.versionName} ($code)", 16, ui.muted).apply { setPadding(ui.dp(3), 0, 0, 0) })
    }

    private fun phoneKey(body: LinearLayout) {
        ui.heading(body, "Phone key")
        val key = ui.text("Looking up this phone’s SSH public key…", 13, ui.muted).apply {
            setTextIsSelectable(true)
            setPadding(ui.dp(3), 0, ui.dp(3), ui.dp(8))
        }
        body.addView(key)
        val actions = ui.line()
        val copy = ui.button("Copy public key") {}.apply { isEnabled = false }
        val share = ui.button("Share") {}.apply { isEnabled = false }
        actions.addView(copy)
        actions.addView(share)
        body.addView(actions)
        val groups = HostGroup.of(pages.hosts.profiles())
        val install = ui.row(body, "Install on a host", if (groups.isEmpty()) "Add a host first" else "Use the account’s SSH password once",
            action = {
                AlertDialog.Builder(activity).setTitle("Choose a host")
                    .setItems(groups.map { it.destination }.toTypedArray()) { _, i -> pages.go(Page.InstallKey(groups[i].account)) }
                    .show()
            })
        install.isEnabled = false
        footer(body, "Android Keystore keeps this P-256 key in secure hardware. It signs SSH logins while the phone is unlocked.")
        pages.background({ runCatching { enclavePublicKey(PhoneKey.open().point) } }) { result ->
            result.onSuccess { public ->
                key.text = public
                key.typeface = Typeface.MONOSPACE
                key.setTextColor(ui.ink)
                copy.isEnabled = true
                share.isEnabled = true
                install.isEnabled = groups.isNotEmpty()
                copy.setOnClickListener {
                    activity.getSystemService(ClipboardManager::class.java).setPrimaryClip(ClipData.newPlainText("SSH public key", public))
                    pages.toast("Public key copied")
                }
                share.setOnClickListener {
                    val send = Intent(Intent.ACTION_SEND).setType("text/plain").putExtra(Intent.EXTRA_TEXT, public)
                    activity.startActivity(Intent.createChooser(send, "Share public key"))
                }
            }.onFailure {
                key.text = (it as? HardwareKeyUnavailable)?.message ?: "This phone could not open its SSH key."
            }
        }
    }

    private fun footer(body: LinearLayout, text: String) =
        body.addView(ui.text(text, 13, ui.muted).apply { setPadding(ui.dp(3), ui.dp(4), ui.dp(3), 0) })
}

/** Orders, adds and removes the toolbar's keys; every change saves at once. */
internal class ToolbarKeyEditor(private val activity: Activity, private val ui: PhoneUi, private val pages: Pages) {
    private val preferences = TerminalPreferences(activity)

    fun build(body: LinearLayout) {
        val keys = preferences.keys
        fun save(next: TerminalKeys) {
            preferences.keys = next
            pages.redraw()
        }
        keys.keys.forEachIndexed { index, key ->
            val row = ui.line().apply {
                background = ui.shape(ui.panel)
                minimumHeight = ui.dp(56)
                setPadding(ui.dp(14), 0, ui.dp(4), 0)
            }
            row.addView(TextView(activity).apply {
                text = key.title
                textSize = 17f
                setTextColor(ui.ink)
            }, LinearLayout.LayoutParams(0, -2, 1f))
            row.addView(ui.button("↑") { save(keys.moved(index, -1)) }.apply {
                contentDescription = "Move ${key.title} earlier"
                isEnabled = index > 0
            })
            row.addView(ui.button("↓") { save(keys.moved(index, 1)) }.apply {
                contentDescription = "Move ${key.title} later"
                isEnabled = index < keys.keys.lastIndex
            })
            row.addView(ui.button("Remove") { save(keys.without(index)) }.apply {
                contentDescription = "Remove ${key.title}"
                isEnabled = keys.keys.size > 1
            })
            body.addView(row, LinearLayout.LayoutParams(-1, -2).apply { bottomMargin = ui.dp(3) })
        }
        val all = TerminalKey.entries
        body.addView(ui.button("Add key") {
            AlertDialog.Builder(activity).setTitle("Add key")
                .setItems(all.map { it.title }.toTypedArray()) { _, i -> save(keys.with(all[i])) }
                .show()
        }.apply { isEnabled = keys.keys.size < TerminalKeys.LIMIT })
        body.addView(ui.button("Restore default keys") { save(TerminalKeys.standard) })
        body.addView(ui.text("Swipe the toolbar to reach keys past the screen edge. Shift, Control and Alt apply to the next key.",
            13, ui.muted).apply { setPadding(ui.dp(3), ui.dp(4), ui.dp(3), 0) })
    }
}
