// ABOUTME: Renders the last observed session's aggregate agent status from private local storage.
// ABOUTME: Keeps SSH in the foreground app and timestamps the widget's cached counts.
package ai.spaceport.mobile

import android.app.PendingIntent
import android.appwidget.AppWidgetManager
import android.appwidget.AppWidgetProvider
import android.content.ComponentName
import android.content.Context
import android.text.format.DateUtils
import android.widget.RemoteViews

internal data class AgentCounts(val waiting: Int, val working: Int, val idle: Int) {
    companion object {
        fun of(rows: List<AgentRow>) = AgentCounts(
            rows.count { it.status == "blocked" }, rows.count { it.status == "working" },
            rows.count { it.status == "idle" || it.status == "done" },
        )
    }
}

class AgentStatusWidget : AppWidgetProvider() {
    override fun onUpdate(ctx: Context, manager: AppWidgetManager, ids: IntArray) {
        if (ids.isNotEmpty()) manager.updateAppWidget(ids, contents(ctx))
    }

    companion object {
        internal fun update(ctx: Context) {
            val manager = AppWidgetManager.getInstance(ctx)
            val ids = manager.getAppWidgetIds(ComponentName(ctx, AgentStatusWidget::class.java))
            if (ids.isNotEmpty()) manager.updateAppWidget(ids, contents(ctx))
        }

        internal fun contents(ctx: Context): RemoteViews {
            val store = HostStore(ctx)
            val host = store.lastOpened()
            val token = store.widgetToken()
            val catalog = AgentCatalog(ctx)
            val counts = host?.let(catalog::counts)
            val at = host?.let(catalog::observedAt)
            val views = RemoteViews(ctx.packageName, R.layout.agent_status_widget)
            views.setTextViewText(R.id.widget_waiting, counts?.let { "${it.waiting} waiting for you" } ?: "Open a session")
            views.setTextViewText(R.id.widget_counts, counts?.let { "${it.working} working · ${it.idle} idle" } ?: "")
            views.setTextViewText(R.id.widget_time, at?.let {
                "Observed " + DateUtils.formatDateTime(ctx, it, DateUtils.FORMAT_SHOW_DATE or DateUtils.FORMAT_SHOW_YEAR or DateUtils.FORMAT_SHOW_TIME)
            } ?: "No observation yet")
            for ((id, action) in listOf(R.id.widget_resume to LauncherAction.RESUME, R.id.widget_next to LauncherAction.NEXT)) {
                // Bind even an empty widget to an empty destination, so it cannot switch to a later session.
                val intent = action.intent(ctx, token ?: "empty")
                views.setOnClickPendingIntent(id, PendingIntent.getActivity(ctx, 0, intent, PendingIntent.FLAG_IMMUTABLE))
            }
            return views
        }
    }
}
