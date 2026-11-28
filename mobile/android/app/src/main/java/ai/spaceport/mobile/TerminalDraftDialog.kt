// ABOUTME: Keeps an unsent draft in memory and shows its exact pane destination.
// ABOUTME: Stages a screenshot over SSH and retains it only after the same pane accepts the paste.
package ai.spaceport.mobile

import android.app.Activity
import android.app.AlertDialog
import android.content.ClipboardManager
import android.content.ContentResolver
import android.text.InputFilter
import android.widget.Button
import android.widget.CheckBox
import android.widget.EditText
import android.widget.LinearLayout
import android.widget.ScrollView
import android.widget.TextView
import androidx.core.widget.doAfterTextChanged
import java.util.concurrent.ExecutorService
import uniffi.spaceport_mobile.InputTarget
import uniffi.spaceport_mobile.MobileView

internal class TerminalDraftDialog(
    private val activity: Activity,
    private val connection: SessionController,
    private val executor: ExecutorService,
    private val onDismiss: () -> Unit
) {
    private var dialog: AlertDialog? = null
    val isShowing get() = dialog?.isShowing == true
    private val draft = connection.draft
    private var refresh: (() -> Unit)? = null
    private var current: InputTarget? = null
    private val preferences = TerminalPreferences(activity)

    private var activeTransfer: uniffi.spaceport_mobile.MobileFileTransfer? = null
    fun refreshAttachment() { refresh?.invoke() }

    fun update(target: InputTarget?) {
        if (target == current) return
        current = target
        refresh?.invoke()
    }

    private fun name(view: MobileView, target: InputTarget): String {
        val pane = view.panes.find { it.id == target.paneId }
        val tab = view.tabs.find { it.id == pane?.tabId }
        val workspace = view.workspaces.find { it.id == tab?.workspaceId }
        val paneName = pane?.terminalTitle?.takeIf { it.isNotBlank() } ?: pane?.agentName ?: pane?.label ?: target.paneId
        return listOfNotNull(connection.profile?.hostname, connection.profile?.session, workspace?.label, tab?.label, paneName, target.popupId?.let { "Popup $it" })
            .filter { it.isNotBlank() }.joinToString(" › ")
    }

    fun show(view: MobileView, target: InputTarget) {
        if (dialog != null) return
        current = target
        draft.present(target, name(view, target))
        val form = LinearLayout(activity).apply {
            orientation = LinearLayout.VERTICAL
            val pad = (16 * resources.displayMetrics.density).toInt()
            setPadding(pad, 0, pad, 0)
        }
        val endpoint = TextView(activity)
        val retarget = Button(activity).apply { setText(R.string.terminal_use_selected); isAllCaps = false }
        val input = EditText(activity).apply {
            hint = "Message"
            setText(draft.note)
            doAfterTextChanged { draft.note = it.toString() }
            setSelection(length())
            minLines = 2
            maxLines = 6
            isSaveEnabled = false
            filters = arrayOf(InputFilter.LengthFilter(65536))
        }
        val attach = Button(activity).apply { isAllCaps = false }
        val fileButton = Button(activity).apply { isAllCaps = false }
        var displayedShare = DraftFile.pending(activity)
        val shared = Button(activity).apply { setText(R.string.draft_attach_shared); isAllCaps = false }
        val enter = CheckBox(activity).apply {
            setText(R.string.terminal_compose_return)
            isChecked = preferences.pressReturn
            setOnCheckedChangeListener { _, checked -> preferences.pressReturn = checked }
        }
        val status = TextView(activity)
        listOf(endpoint, retarget, input, attach, fileButton, shared, enter, status).forEach { form.addView(it) }
        val sheet = AlertDialog.Builder(activity).setTitle("Compose")
            .setView(ScrollView(activity).apply { addView(form) })
            .setPositiveButton("Send", null).setNegativeButton("Cancel", null).create()
        var busy = false
        fun redraw() {
            endpoint.text = activity.getString(R.string.terminal_compose_to, draft.label)
            retarget.visibility = if (current != null && !draft.canSend(current)) android.view.View.VISIBLE else android.view.View.GONE
            retarget.isEnabled = !busy
            attach.text = if (draft.png == null) "Paste screenshot" else "Remove screenshot"
            attach.isEnabled = !busy && draft.file == null
            fileButton.text = draft.file?.let { "Remove ${it.name}" } ?: "Choose file"
            fileButton.isEnabled = !busy && draft.png == null
            displayedShare = DraftFile.pending(activity)
            shared.visibility = if (displayedShare != null) android.view.View.VISIBLE else android.view.View.GONE
            shared.isEnabled = !busy && draft.file == null && draft.png == null

            input.isEnabled = !busy
            enter.isEnabled = !busy
            sheet.getButton(AlertDialog.BUTTON_POSITIVE)?.isEnabled = !busy && draft.canSend(current)
        }
        dialog = sheet
        refresh = ::redraw
        retarget.setOnClickListener {
            val selected = current
            val live = connection.view
            if (selected != null && live?.inputTarget == selected) {
                draft.retarget(selected, name(live, selected))
                status.text = ""
                redraw()
            }
        }
        fileButton.setOnClickListener {
            if (draft.file != null) { draft.file = null; redraw() }
            else (activity as? MainActivity)?.pickDraftFile(draft)
        }
        shared.setOnClickListener {
            val entry = displayedShare ?: return@setOnClickListener
            AlertDialog.Builder(activity).setTitle("Shared file").setNegativeButton("Cancel", null)
                .setNeutralButton("Discard") { _, _ -> DraftFile.remove(entry); redraw() }
                .setPositiveButton("Attach") { _, _ ->
                    val target = draft.destination
                    busy = true
                    redraw()
                    executor.execute {
                        val result = runCatching { DraftFile.load(entry) }
                        activity.runOnUiThread {
                            if (dialog !== sheet) return@runOnUiThread
                            busy = false
                            if (draft.destination == target) result.onSuccess {
                                if (DraftFile.remove(entry)) draft.file = it
                            }
                            status.text = result.exceptionOrNull()?.message.orEmpty()
                            redraw()
                        }
                    }
                }.show()
        }
        attach.setOnClickListener {
            if (draft.png != null) {
                draft.png = null
                status.text = ""
                redraw()
                return@setOnClickListener
            }
            val clip = activity.getSystemService(ClipboardManager::class.java).primaryClip
            val uri = clip?.takeIf { it.itemCount > 0 && it.description.hasMimeType("image/*") }
                ?.getItemAt(0)?.uri?.takeIf { it.scheme == ContentResolver.SCHEME_CONTENT }
            if (uri == null) {
                status.setText(R.string.terminal_copy_image)
                return@setOnClickListener
            }
            busy = true
            status.setText(R.string.terminal_reading_image)
            redraw()
            executor.execute {
                val outcome = runCatching {
                    requireNotNull(activity.contentResolver.openInputStream(uri)) { "Could not open clipboard image" }
                        .use(::readScreenshot)
                }
                activity.runOnUiThread {
                    if (dialog !== sheet || !sheet.isShowing) return@runOnUiThread
                    busy = false
                    outcome.fold({ draft.png = it; status.setText(R.string.terminal_image_attached) },
                        { status.text = it.message ?: "Could not read screenshot" })
                    redraw()
                }
            }
        }
        sheet.setOnDismissListener {
            if (dialog !== sheet) return@setOnDismissListener
            activeTransfer?.cancel()
            refresh = null
            dialog = null
            onDismiss()
        }
        sheet.show()
        redraw()
        sheet.getButton(AlertDialog.BUTTON_POSITIVE).setOnClickListener {
            val original = draft.destination ?: return@setOnClickListener
            val session = connection.sessionFor(original) ?: return@setOnClickListener
            val text = input.text.toString()
            val image = draft.png
            val file = draft.file
            if (busy || image == null && file == null && text.isEmpty()) return@setOnClickListener
            val pressReturn = enter.isChecked
            fun complete() {
                draft.clear()
                sheet.dismiss()
            }
            if (image == null && file == null) {
                try {
                    session.sendPaste(original, text, pressReturn)
                    complete()
                } catch (e: Exception) { status.text = e.message ?: "Could not send draft" }
                return@setOnClickListener
            }
            busy = true
            redraw()
            val transfer = uniffi.spaceport_mobile.MobileFileTransfer()
            activeTransfer = transfer
            if (file != null) {
                val progress = object : Runnable {
                    override fun run() {
                        if (activeTransfer !== transfer || dialog !== sheet) return
                        status.text = activity.getString(R.string.draft_upload_progress, file.name,
                            android.text.format.Formatter.formatShortFileSize(activity, transfer.bytesSent().toLong()),
                            android.text.format.Formatter.formatShortFileSize(activity, file.bytes.size.toLong()))
                        status.postDelayed(this, 250)
                    }
                }
                progress.run()
            } else status.setText(R.string.terminal_sending_image)
            executor.execute {
                val outcome = runCatching {
                    if (file != null) session.stageDraftFile(original, file.name, file.bytes, transfer)
                    else session.stageDraftImage(original, requireNotNull(image), transfer)
                }
                activity.runOnUiThread {
                    if (activeTransfer === transfer) activeTransfer = null
                    transfer.close()
                    val staged = outcome.getOrNull()
                    try {
                        if (dialog !== sheet || !sheet.isShowing) return@runOnUiThread
                        busy = false
                        if (staged == null) {
                            status.text = outcome.exceptionOrNull()?.message ?: "Could not stage the attachment"
                        } else if (connection.sessionFor(original) !== session || current != original ||
                            draft.destination != original || draft.file !== file ||
                            draft.png !== image || draft.note != text) {
                            status.setText(R.string.terminal_changed_destination)
                        } else {
                            val path = staged.pastePath()
                            session.sendPaste(original, if (text.isEmpty()) path else "$path $text", pressReturn)
                            staged.keep()
                            complete()
                        }
                    } catch (e: Exception) {
                        status.text = e.message ?: "Could not send the attachment"
                    } finally {
                        staged?.close()
                        if (dialog === sheet) redraw()
                    }
                }
            }
        }
    }

    fun dispose() {
        activeTransfer?.cancel()
        val sheet = dialog
        dialog = null
        refresh = null
        sheet?.dismiss()
    }
}
