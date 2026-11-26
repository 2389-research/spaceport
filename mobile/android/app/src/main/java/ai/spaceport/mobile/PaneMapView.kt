// ABOUTME: Draws the focused Herdr tab's split layout from the pane rectangles Herdr reports.
// ABOUTME: Each tile is a real button, so taps and screen readers reach panes the same way.
package ai.spaceport.mobile

import android.annotation.SuppressLint
import android.content.Context
import android.graphics.Color
import android.view.Gravity
import android.view.ViewGroup
import android.widget.TextView
import kotlin.math.roundToInt
import uniffi.spaceport_mobile.MobilePaneMap
import uniffi.spaceport_mobile.MobileView

internal data class PaneTile(val id: String, val left: Float, val top: Float, val right: Float, val bottom: Float)

/** Scales Herdr's cell rectangles to a [width] by [height] area. */
internal fun paneTiles(map: MobilePaneMap, width: Float, height: Float): List<PaneTile> {
    if (map.columns.toInt() == 0 || map.rows.toInt() == 0) return emptyList()
    val dx = width / map.columns.toFloat()
    val dy = height / map.rows.toFloat()
    return map.panes.map {
        PaneTile(it.id, it.x.toFloat() * dx, it.y.toFloat() * dy,
            (it.x.toFloat() + it.width.toFloat()) * dx, (it.y.toFloat() + it.height.toFloat()) * dy)
    }
}

// Built in code with the view and its selection callback; never inflated from XML.
@SuppressLint("ViewConstructor")
internal class PaneMapView(context: Context, view: MobileView, private val map: MobilePaneMap, select: (String) -> Unit) :
    ViewGroup(context) {
    private val ui = PhoneUi(context)
    private val maxHeight = ui.dp(240)

    init {
        contentDescription = "Pane map"
        val panes = view.panes.filter { it.tabId == view.focusedTab }.associateBy { it.id }
        for (rect in map.panes) {
            val pane = panes[rect.id] ?: continue
            val selected = pane.id == view.focusedPane
            val status = when (pane.status) {
                "blocked" -> "Needs you"
                "working" -> "Working"
                "idle", "done" -> "Idle"
                else -> null
            }
            addView(TextView(context).apply {
                tag = rect.id
                text = listOfNotNull(pane.agentName ?: pane.label, status).joinToString("\n")
                textSize = 13f
                setTextColor(ui.ink)
                gravity = Gravity.TOP or Gravity.START
                setPadding(ui.dp(8), ui.dp(6), ui.dp(8), ui.dp(6))
                background = ui.shape(ui.panel, 14).apply {
                    setStroke(ui.dp(if (selected) 2 else 1),
                        if (selected) (if (pane.status == "blocked") ui.accent else Color.WHITE) else ui.muted)
                }
                isSelected = selected
                contentDescription = listOfNotNull(pane.label, status, if (selected) "Selected" else null).joinToString(", ")
                setOnClickListener { select(pane.id) }
            })
        }
    }

    override fun onMeasure(widthMeasureSpec: Int, heightMeasureSpec: Int) {
        val width = MeasureSpec.getSize(widthMeasureSpec)
        val height = minOf((width / 1.8f).roundToInt(), maxHeight)
        setMeasuredDimension(width, height)
        val tiles = paneTiles(map, width.toFloat(), height.toFloat()).associateBy { it.id }
        for (i in 0 until childCount) {
            val child = getChildAt(i)
            val tile = tiles[child.tag] ?: continue
            child.measure(
                MeasureSpec.makeMeasureSpec(((tile.right - tile.left).roundToInt() - 2).coerceAtLeast(0), MeasureSpec.EXACTLY),
                MeasureSpec.makeMeasureSpec(((tile.bottom - tile.top).roundToInt() - 2).coerceAtLeast(0), MeasureSpec.EXACTLY),
            )
        }
    }

    override fun onLayout(changed: Boolean, l: Int, t: Int, r: Int, b: Int) {
        val tiles = paneTiles(map, (r - l).toFloat(), (b - t).toFloat()).associateBy { it.id }
        for (i in 0 until childCount) {
            val child = getChildAt(i)
            val tile = tiles[child.tag] ?: continue
            val left = tile.left.roundToInt() + 1
            val top = tile.top.roundToInt() + 1
            child.layout(left, top, left + child.measuredWidth, top + child.measuredHeight)
        }
    }
}
