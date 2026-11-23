// ABOUTME: Exercises the real Android bitmap codec used for pasted screenshots.
// ABOUTME: Checks untrusted image input and the PNG bytes handed to the SSH stager.
package ai.spaceport.mobile

import android.graphics.Bitmap
import android.graphics.BitmapFactory
import android.graphics.Color
import androidx.test.ext.junit.runners.AndroidJUnit4
import java.io.ByteArrayOutputStream
import org.junit.Assert.*
import org.junit.Test
import org.junit.runner.RunWith

@RunWith(AndroidJUnit4::class)
class TerminalScreenshotTest {
    @Test fun pngAttachmentContainsTheCopiedPixels() {
        val image = Bitmap.createBitmap(3, 2, Bitmap.Config.ARGB_8888)
        image.eraseColor(Color.MAGENTA)
        val source = ByteArrayOutputStream()
        image.compress(Bitmap.CompressFormat.PNG, 100, source)
        image.recycle()
        val attachment = readScreenshot(source.toByteArray().inputStream())
        val decoded = BitmapFactory.decodeByteArray(attachment, 0, attachment.size)
        assertEquals(3, decoded.width)
        assertEquals(2, decoded.height)
        assertEquals(Color.MAGENTA, decoded.getPixel(2, 1))
        decoded.recycle()
    }

    @Test fun invalidImageFailsWithoutAnAttachment() {
        try {
            readScreenshot("broken".byteInputStream())
            fail("expected invalid image")
        } catch (e: IllegalArgumentException) {
            assertTrue(e.message.orEmpty().contains("image"))
        }
    }
}
