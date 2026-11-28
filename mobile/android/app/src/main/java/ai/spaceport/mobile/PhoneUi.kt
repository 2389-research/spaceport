// ABOUTME: Shares typography, touch targets, and colors across Android navigation screens.
// ABOUTME: Uses native Android controls for accessibility, input, and focus.
package ai.spaceport.mobile

import android.content.Context
import android.graphics.Color
import android.graphics.Typeface
import android.graphics.drawable.GradientDrawable
import android.view.Gravity
import android.view.View
import android.view.GestureDetector
import android.view.MotionEvent
import android.widget.*
import androidx.core.view.ViewCompat
import kotlin.math.abs

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
            // A disabled button must look unavailable, not like every other accent action.
            setTextColor(android.content.res.ColorStateList(
                arrayOf(intArrayOf(-android.R.attr.state_enabled), intArrayOf()),
                intArrayOf(Color.argb(110, 182, 177, 188), accent),
            ))
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
        swiped: ((towardEnd: Boolean) -> Unit)? = null,
        actions: List<Pair<String, () -> Unit>> = emptyList(),
        brand: AgentRow? = null,
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
        if (brand?.name != null) {
            val drawable = agentBrand(ctx, brand.identity)
            val badge = if (drawable != null) android.widget.ImageView(ctx).apply {
                setImageDrawable(drawable)
                if (brand.identity != "omo") setColorFilter(ink)
            } else text(brand.name.take(1).uppercase(), 16, ink, true).apply { gravity = Gravity.CENTER }
            badge.importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
            row.addView(badge, LinearLayout.LayoutParams(dp(22), dp(22)).apply { marginEnd = dp(10) })
        }
        val labels = column()
        labels.addView(
            text(title, 17, ink, true).apply {
                maxLines = 2
                ellipsize = android.text.TextUtils.TruncateAt.END
            }
        )
        val description = if (brand?.name == null) detail else {
            val state = if (brand.status == "blocked") "Waiting for you" else brand.status.replaceFirstChar { it.uppercase() }
            listOf(detail, state).filter(String::isNotBlank).joinToString(" · ")
        }
        if (description.isNotBlank()) labels.addView(text(description, 13, muted).apply { id = android.R.id.text2; maxLines = 2 })
        row.addView(labels, LinearLayout.LayoutParams(0, -2, 1f))
        row.addView(
            text("›", 23, muted).apply {
                importantForAccessibility = View.IMPORTANT_FOR_ACCESSIBILITY_NO
            }
        )
        if (swiped != null) swipes(row, swiped)
        for ((label, act) in actions) ViewCompat.addAccessibilityAction(row, label) { _, _ -> act(); true }
        parent.addView(row, LinearLayout.LayoutParams(-1, -2).apply { bottomMargin = dp(3) })
        return row
    }

    /** A mostly horizontal fling on [row] reports its direction instead of clicking the row. */
    // Taps still reach the row's own click handling; TalkBack reaches swipes through row actions.
    @android.annotation.SuppressLint("ClickableViewAccessibility")
    private fun swipes(row: View, swiped: (towardEnd: Boolean) -> Unit) {
        val detector = GestureDetector(ctx, object : GestureDetector.SimpleOnGestureListener() {
            override fun onFling(e1: MotionEvent?, e2: MotionEvent, velocityX: Float, velocityY: Float): Boolean {
                val start = e1 ?: return false
                val dx = e2.x - start.x
                if (abs(dx) < dp(72) || abs(dx) < abs(e2.y - start.y) * 2) return false
                swiped((dx > 0) != (row.layoutDirection == View.LAYOUT_DIRECTION_RTL))
                return true
            }
        })
        row.setOnTouchListener { view, event ->
            if (!detector.onTouchEvent(event)) return@setOnTouchListener false
            // The row saw the press; cancel it so the swipe neither clicks nor stays pressed.
            MotionEvent.obtain(event).apply { action = MotionEvent.ACTION_CANCEL }.also {
                view.onTouchEvent(it)
                it.recycle()
            }
            true
        }
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
