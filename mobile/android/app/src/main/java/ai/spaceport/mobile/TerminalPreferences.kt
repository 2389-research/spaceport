// ABOUTME: Reads and saves the terminal's text size, toolbar keys and Compose return choice.
// ABOUTME: Settings and the live terminal share these values through one preferences file.
package ai.spaceport.mobile

import android.content.Context
import androidx.core.content.edit

internal class TerminalPreferences(context: Context) {
    private val prefs = context.applicationContext.getSharedPreferences("terminal", Context.MODE_PRIVATE)

    var textSize: Int
        get() = prefs.getInt("textSize", DEFAULT_TEXT_SIZE).coerceIn(TEXT_SIZES)
        set(value) = prefs.edit { putInt("textSize", value.coerceIn(TEXT_SIZES)) }

    var keys: TerminalKeys
        get() = TerminalKeys.parse(prefs.getString("toolbarKeys", null))
        set(value) = prefs.edit { putString("toolbarKeys", value.encoded) }

    /** Compose presses Return after pasting unless someone turned it off. */
    var pressReturn: Boolean
        get() = prefs.getBoolean("composeReturn", true)
        set(value) = prefs.edit { putBoolean("composeReturn", value) }

    companion object {
        const val DEFAULT_TEXT_SIZE = 13
        val TEXT_SIZES = 9..24
    }
}
