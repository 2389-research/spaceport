// ABOUTME: Shares typography, touch targets, and colors across Android navigation screens.
// ABOUTME: Uses native Android controls for accessibility, input, and focus.
package ai.spaceport.mobile

import android.content.Context
import android.graphics.Color
import android.graphics.Typeface
import android.graphics.drawable.GradientDrawable
import android.view.Gravity
import android.view.View
import android.widget.*

internal class PhoneUi(private val ctx: Context) {
    val ink = Color.rgb(241, 240, 244)
    val muted = Color.rgb(182, 177, 188)
    val bg = Color.rgb(20, 19, 23)
    val panel = Color.rgb(38, 36, 43)
    val accent = Color.rgb(255, 153, 194)

    fun dp(n: Int) = (ctx.resources.displayMetrics.density * n + .5f).toInt()

    fun shape(color: Int, radius: Int = 12) =
        GradientDrawable().apply {
            setColor(color)
            cornerRadius = dp(radius).toFloat()
        }

    fun column() = LinearLayout(ctx).apply { orientation = LinearLayout.VERTICAL }

    fun line() = LinearLayout(ctx).apply { gravity = Gravity.CENTER_VERTICAL }

    fun text(value: String, size: Int = 16, color: Int = ink, bold: Boolean = false) =
        TextView(ctx).apply {
            text = value
            textSize = size.toFloat()
            setTextColor(color)
            if (bold) setTypeface(typeface, Typeface.BOLD)
        }

    fun button(value: String, action: () -> Unit) =
        Button(ctx).apply {
            text = value
            isAllCaps = false
            textSize = 15f
            setTextColor(accent)
            minHeight = dp(48)
            minWidth = 0
            setBackgroundColor(Color.TRANSPARENT)
            setOnClickListener { action() }
        }

    fun heading(parent: LinearLayout, title: String) =
        text(title, 14, muted, true).also {
            it.setPadding(dp(3), dp(20), dp(3), dp(7))
            it.accessibilityHeadingCompat()
            parent.addView(it)
        }

    fun row(
        parent: LinearLayout,
        title: String,
        detail: String,
        status: String? = null,
        action: () -> Unit,
        held: (() -> Unit)? = null,
    ): LinearLayout {
        val row =
            line().apply {
                background = shape(panel)
                minimumHeight = dp(60)
                setPadding(dp(14), dp(10), dp(12), dp(10))
                isFocusable = true
                setOnClickListener { action() }
                if (held != null)
                    setOnLongClickListener {
                        held()
                        true
                    }
            }
        if (status != null) {
            val color =
                when (status) {
                    "working" -> Color.rgb(121, 175, 255)
                    "blocked" -> accent
                    else -> muted
                }
            row.addView(
                View(ctx).apply { background = shape(color) },
                LinearLayout.LayoutParams(dp(7), dp(7)).apply { marginEnd = dp(10) },
            )
        }
        val labels = column()
        labels.addView(
            text(title, 17, ink, true).apply {
                maxLines = 2
                ellipsize = android.text.TextUtils.TruncateAt.END
            }
        )
        if (detail.isNotBlank()) labels.addView(text(detail, 13, muted).apply { maxLines = 2 })
        row.addView(labels, LinearLayout.LayoutParams(0, -2, 1f))
        row.addView(
            text("›", 23, muted).apply {
                importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
            }
        )
        parent.addView(row, LinearLayout.LayoutParams(-1, -2).apply { bottomMargin = dp(3) })
        return row
    }

    /** Asks for a Herdr session name; an empty answer means "default". */
    fun askSessionName(title: String, initial: String, done: (String) -> Unit) {
        val input = EditText(ctx).apply {
            hint = "default"
            isSingleLine = true
            setText(initial)
        }
        android.app.AlertDialog.Builder(ctx).setTitle(title)
            .setView(LinearLayout(ctx).apply {
                setPadding(dp(20), 0, dp(20), 0)
                addView(input, LinearLayout.LayoutParams(-1, -2))
            })
            .setNegativeButton("Cancel", null)
            .setPositiveButton("Save") { _, _ -> done(input.text.toString().trim().ifEmpty { "default" }) }
            .show()
    }

    private fun View.accessibilityHeadingCompat() {
        if (android.os.Build.VERSION.SDK_INT >= 28) isAccessibilityHeading = true
    }
}

internal val HostProfile.destination: String
    get() {
        val address = if (':' in hostname) "[$hostname]" else hostname
        return "$username@$address" + if (port == 22) "" else ":$port"
    }
