// ABOUTME: Accepts a single system share only after the user confirms local staging.
// ABOUTME: Holds no SSH session and leaves the actual destination choice to the composer.
package ai.spaceport.mobile

import android.app.Activity
import android.app.AlertDialog
import android.content.Intent
import android.net.Uri
import android.os.Bundle
import android.widget.Toast
import androidx.core.net.toUri
import java.util.concurrent.Executors

class ShareFileActivity : Activity() {
    private val worker = Executors.newSingleThreadExecutor()
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        // Parse the external URI once. Permission checks and the provider read use the same spelling.
        val uri = androidx.core.content.IntentCompat.getParcelableExtra(intent, Intent.EXTRA_STREAM, Uri::class.java)
            ?.toString()?.toUri()
        if (intent.action != Intent.ACTION_SEND || uri?.scheme != "content" || uri.authority.isNullOrBlank() ||
            (intent.flags and Intent.FLAG_GRANT_READ_URI_PERMISSION) == 0 ||
            checkUriPermission(uri, android.os.Process.myPid(), android.os.Process.myUid(),
                Intent.FLAG_GRANT_READ_URI_PERMISSION) != android.content.pm.PackageManager.PERMISSION_GRANTED) {
            finish()
            return
        }
        AlertDialog.Builder(this).setTitle("Save file to Spaceport?")
            .setMessage("Open Compose in a terminal to attach this file.")
            .setNegativeButton("Cancel") { _, _ -> finish() }
            .setOnCancelListener { finish() }
            .setPositiveButton("Save") { _, _ ->
                worker.execute {
                    val result = runCatching { DraftFile.save(this, DraftFile.read(this, uri)) }
                    runOnUiThread {
                        if (!isDestroyed) {
                            val message = if (result.isSuccess) "File saved. Open Compose to attach it."
                                else if (DraftFile.pending(this) != null) "Attach or discard the previous shared file first."
                                else "Could not save this file. Choose a readable file up to 16 MB."
                            Toast.makeText(this, message, Toast.LENGTH_LONG).show()
                            finish()
                        }
                    }
                }
            }.show()
    }
    override fun onDestroy() { worker.shutdownNow(); super.onDestroy() }
}
