// ABOUTME: Shares a real Android document through the exported activity and inspects the private inbox.
// ABOUTME: Confirms binary content survives the provider and a second share cannot replace it.
package ai.spaceport.mobile

import android.content.ContentValues
import android.content.Intent
import android.os.Build
import android.provider.MediaStore
import androidx.test.core.app.ActivityScenario
import org.junit.Assert.*
import org.junit.Assume.assumeTrue
import org.junit.Test

class DraftFileShareTest : LiveMobileUi() {
    @Test fun confirmedShareKeepsItsBytesAndNameUntilAttached() {
        assumeTrue(Build.VERSION.SDK_INT >= 29)
        val context = instrumentation.targetContext
        var original: java.io.File? = null
        assumeTrue("Requires an empty test inbox", DraftFile.pending(context) == null)
        val resolver = context.contentResolver
        val name = "spaceport-test-${java.util.UUID.randomUUID()}.txt"
        val bytes = byteArrayOf(0, 12, -1)
        val uri = requireNotNull(resolver.insert(MediaStore.Downloads.EXTERNAL_CONTENT_URI, ContentValues().apply {
            put(MediaStore.MediaColumns.DISPLAY_NAME, name)
            put(MediaStore.MediaColumns.MIME_TYPE, "text/plain")
        }))
        try {
            requireNotNull(resolver.openOutputStream(uri)).use { it.write(bytes) }
            val intent = Intent(context, ShareFileActivity::class.java).apply {
                action = Intent.ACTION_SEND
                type = "text/plain"
                putExtra(Intent.EXTRA_STREAM, uri)
                clipData = android.content.ClipData.newRawUri("file", uri)
                addFlags(Intent.FLAG_GRANT_READ_URI_PERMISSION or Intent.FLAG_ACTIVITY_NEW_TASK)
            }
            ActivityScenario.launch<ShareFileActivity>(intent).use {
                after("confirmed local share", { click("Save") }, { DraftFile.pending(context) != null })
            }
            val entry = requireNotNull(DraftFile.pending(context))
            original = entry
            val file = DraftFile.load(entry)
            assertEquals(name, file.name)
            assertArrayEquals(bytes, file.bytes)
            assertTrue(runCatching { DraftFile.save(context, DraftFile("another", byteArrayOf(1))) }.isFailure)
            assertArrayEquals(bytes, DraftFile.load(entry).bytes)
            assertTrue(DraftFile.remove(entry))
            DraftFile.save(context, DraftFile("replacement", bytes))
            original = DraftFile.pending(context)
            assertFalse(DraftFile.remove(entry))
            assertEquals("replacement", DraftFile.load(requireNotNull(original)).name)
        } finally {
            resolver.delete(uri, null, null)
            original?.let(DraftFile::remove)
        }
    }
}
