// ABOUTME: Bounds clipboard image input before decoding it to a fresh PNG.
// ABOUTME: Drops source metadata and limits both bitmap memory and SSH payload size.
package ai.spaceport.mobile

import android.graphics.Bitmap
import android.graphics.BitmapFactory
import java.io.ByteArrayOutputStream
import java.io.InputStream

private const val IMAGE_LIMIT = 16 * 1024 * 1024

internal fun readScreenshot(input: InputStream): ByteArray {
    val bytes = ByteArrayOutputStream()
    val chunk = ByteArray(32 * 1024)
    while (true) {
        val count = input.read(chunk)
        if (count == -1) break
        require(bytes.size() + count <= IMAGE_LIMIT) { "Image is too large. Crop it and try again." }
        bytes.write(chunk, 0, count)
    }
    val encoded = bytes.toByteArray()
    val options = BitmapFactory.Options().apply { inJustDecodeBounds = true }
    BitmapFactory.decodeByteArray(encoded, 0, encoded.size, options)
    require(options.outWidth > 0 && options.outHeight > 0) { "Could not read this image" }
    val longest = maxOf(options.outWidth, options.outHeight)
    require(longest <= 32768 && options.outWidth.toLong() * options.outHeight <= 64_000_000L) {
        "Image dimensions are too large"
    }
    options.inJustDecodeBounds = false
    options.inSampleSize = 1
    while (longest / options.inSampleSize > 2048) options.inSampleSize *= 2
    val bitmap = requireNotNull(BitmapFactory.decodeByteArray(encoded, 0, encoded.size, options)) {
        "Could not decode this image"
    }
    try {
        bytes.reset()
        check(bitmap.compress(Bitmap.CompressFormat.PNG, 100, bytes)) { "Could not encode screenshot" }
        require(bytes.size() <= IMAGE_LIMIT) { "Screenshot exceeds the host image limit" }
        return bytes.toByteArray()
    } finally {
        bitmap.recycle()
    }
}
