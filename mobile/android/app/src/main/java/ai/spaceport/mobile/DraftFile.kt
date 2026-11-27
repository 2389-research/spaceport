// ABOUTME: Reads a bounded content URI and stores one incoming share in private app storage.
// ABOUTME: Publishes the whole file atomically so the composer never sees partial data.
package ai.spaceport.mobile

import android.content.Context
import android.content.ContentResolver
import android.net.Uri
import android.provider.OpenableColumns
import android.system.Os
import java.io.*

internal data class DraftFile(val name: String, val bytes: ByteArray) {
    companion object {
        const val LIMIT = 16 * 1024 * 1024

        fun read(context: Context, uri: Uri): DraftFile {
            require(uri.scheme == ContentResolver.SCHEME_CONTENT) { "Choose a document from a file provider." }
            val resolver = context.contentResolver
            val name = resolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME), null, null, null)?.use {
                if (it.moveToFirst()) it.getString(0) else null
            } ?: "attachment"
            require(name.isNotBlank() && name !in listOf(".", "..") && '/' !in name &&
                name.toByteArray().size <= 255 && name.none(Char::isISOControl)) { "Invalid document name." }
            return DraftFile(name, requireNotNull(resolver.openInputStream(uri)).use(::readBytes))
        }

        internal fun readBytes(input: InputStream): ByteArray {
            val output = ByteArrayOutputStream()
            val buffer = ByteArray(64 * 1024)
            while (true) {
                val count = input.read(buffer)
                if (count < 0) return output.toByteArray()
                require(count > 0 && output.size() + count <= LIMIT) { "Choose a file up to 16 MB." }
                output.write(buffer, 0, count)
            }
        }

        private fun directory(context: Context) = File(context.noBackupFilesDir, "incoming-draft")
        fun pending(context: Context): File? = directory(context).listFiles()?.singleOrNull()

        fun save(context: Context, file: DraftFile) {
            val temporary = File(context.noBackupFilesDir, java.util.UUID.randomUUID().toString())
            check(temporary.mkdir()) { "Could not prepare the share." }
            val entry = File(temporary, java.util.UUID.randomUUID().toString())
            try {
                DataOutputStream(entry.outputStream().buffered()).use {
                    it.writeUTF(file.name)
                    it.write(file.bytes)
                }
                Os.rename(temporary.path, directory(context).path)
            } finally {
                entry.delete()
                temporary.delete()
            }
        }

        fun load(entry: File): DraftFile =
            DataInputStream(entry.inputStream().buffered()).use { DraftFile(it.readUTF(), readBytes(it)) }

        fun remove(entry: File): Boolean {
            if (!entry.delete()) return false
            // delete() only removes an empty directory. A newer share has its own entry.
            entry.parentFile?.delete()
            return true
        }
    }
}
