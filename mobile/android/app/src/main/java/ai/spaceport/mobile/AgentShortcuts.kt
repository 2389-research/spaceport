// ABOUTME: Publishes launcher actions through a private alias for the existing session activity.
// ABOUTME: Supplies immutable widget actions bound to the session the widget described.
package ai.spaceport.mobile

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.pm.ShortcutInfo
import android.content.pm.ShortcutManager
import android.graphics.drawable.Icon
import android.net.Uri

internal enum class LauncherAction(val id: String, val title: String, val icon: Int) {
    RESUME("resume", "Resume terminal", android.R.drawable.ic_media_play),
    NEXT("next_waiting", "Next waiting agent", android.R.drawable.ic_media_next);

    fun intent(ctx: Context, widgetToken: String? = null): Intent =
        Intent("${ctx.packageName}.shortcut.$id").apply {
            component = entry(ctx)
            flags = Intent.FLAG_ACTIVITY_CLEAR_TOP or Intent.FLAG_ACTIVITY_SINGLE_TOP
            // Data differentiates PendingIntents. An old widget tap cannot silently change hosts.
            if (widgetToken != null) data = Uri.fromParts("spaceport-widget", widgetToken, null)
        }

    companion object {
        private fun entry(ctx: Context) = ComponentName(ctx.packageName, "ai.spaceport.mobile.AgentShortcut")
        fun from(ctx: Context, intent: Intent): LauncherAction? =
            if (intent.component != entry(ctx)) null
            else entries.firstOrNull { intent.action == "${ctx.packageName}.shortcut.${it.id}" }
    }
}

internal fun publishAgentShortcuts(ctx: Context) {
    val manager = ctx.getSystemService(ShortcutManager::class.java)
    val existing = manager.dynamicShortcuts.map { it.id }
    val missing = LauncherAction.entries.filter { it.id !in existing }
    if (missing.isEmpty()) return
    manager.addDynamicShortcuts(missing.map { action ->
        ShortcutInfo.Builder(ctx, action.id)
            .setShortLabel(action.title)
            .setIcon(Icon.createWithResource(ctx, action.icon))
            .setIntent(action.intent(ctx))
            .build()
    })
}
