// ABOUTME: Saves SSH routes and host fingerprints in the app's private preferences.
// ABOUTME: Records device-key enrollment only for the matching account and pinned host.
package ai.spaceport.mobile

import android.content.Context
import androidx.core.content.edit
import org.json.JSONArray

class HostStore(context: Context) {
    private val app = context.applicationContext
    private val prefs = context.applicationContext.getSharedPreferences("spaceport.hosts", Context.MODE_PRIVATE)

    fun profiles(): List<HostProfile> = prefs.getStringSet("profiles", emptySet()).orEmpty().map { encoded ->
        val parts = JSONArray(encoded)
        HostProfile(parts.getString(0), parts.getInt(1), parts.getString(2), parts.getString(3))
    }.sortedWith(compareBy({ it.hostname }, { it.port }, { it.username }, { it.session }))

    fun save(profile: HostProfile) = replace(emptyList(), listOf(profile))

    fun remove(profile: HostProfile) {
        replace(listOf(profile), emptyList())
        allowHomeObservation(profile, true)
    }

    internal fun recordOpened(profile: HostProfile) {
        if (profile !in profiles() || profile == lastOpened()) return
        prefs.edit {
            putString("lastOpened", encode(profile))
            putString("widgetToken", java.util.UUID.randomUUID().toString())
        }
        AgentStatusWidget.update(app)
    }

    internal fun widgetToken(): String? = prefs.getString("widgetToken", null)

    internal fun lastOpened(token: String? = null): HostProfile? {
        if (token != null && token != widgetToken()) return null
        val saved = prefs.getString("lastOpened", null) ?: return null
        return profiles().firstOrNull { encode(it) == saved }
    }

    private fun clearLastOpenedIfRemoved() {
        if (prefs.contains("lastOpened") && lastOpened() == null) {
            prefs.edit { remove("lastOpened"); remove("widgetToken") }
            AgentStatusWidget.update(app)
        }
    }

    internal fun permitsHomeObservation(profile: HostProfile): Boolean =
        !prefs.getBoolean("manual/${encode(profile)}", false)

    internal fun allowHomeObservation(profile: HostProfile, allow: Boolean) = prefs.edit {
        val key = "manual/${encode(profile)}"
        if (allow) remove(key) else putBoolean(key, true)
    }

    /** Replaces saved sessions in one write so Home never lists both the old and new names. */
    fun replace(old: List<HostProfile>, new: List<HostProfile>) = synchronized(storageLock) {
        new.forEach { require(it.validationError() == null) { it.validationError().orEmpty() } }
        val profiles = prefs.getStringSet("profiles", emptySet()).orEmpty()
        val saved = profiles - old.map(::encode).toSet() + new.map(::encode)
        val remaining = profiles().filter { encode(it) in saved } + new
        prefs.edit {
            putStringSet("profiles", saved)
            for (account in old.map { it.account }.distinct()) {
                if (remaining.none { it.account == account }) remove("tags/$account")
            }
        }
        clearLastOpenedIfRemoved()
    }

    var selectedTag: String
        get() = prefs.getString("home-tag", "").orEmpty()
        set(value) = prefs.edit { putString("home-tag", value) }

    fun tags(account: String): List<String> = prefs.getString("tags/$account", "").orEmpty()
        .split(',').filter(String::isNotEmpty)

    fun tag(account: String, text: String) {
        val tags = text.split(',').map { field ->
            val words = field.trim().split(Regex("\\s+")).filter { word -> word.none(Char::isISOControl) }
            val tag = words.joinToString(" ")
            tag.substring(0, tag.offsetByCodePoints(0, minOf(24, tag.codePointCount(0, tag.length)))).trimEnd()
        }.filter(String::isNotEmpty).distinctBy { it.lowercase() }.take(8)
        prefs.edit {
            if (tags.isEmpty()) remove("tags/$account") else putString("tags/$account", tags.joinToString(","))
        }
    }

    fun availableTags(profiles: List<HostProfile>): List<String> =
        profiles.map { it.account }.distinct().flatMap(::tags).distinctBy { it.lowercase() }.sortedBy { it.lowercase() }

    fun filtered(tag: String, profiles: List<HostProfile> = profiles()): List<HostProfile> {
        if (tag.isBlank()) return profiles
        return profiles.filter { p -> tags(p.account).any { it.equals(tag, ignoreCase = true) } }.ifEmpty { profiles }
    }

    /** An optional label shown in place of the SSH destination; blank clears it. */
    fun name(account: String): String? = prefs.getString("name/$account", null)

    fun rename(account: String, name: String) = prefs.edit {
        if (name.isBlank()) remove("name/$account") else putString("name/$account", name.trim().take(64))
    }

    private fun encode(profile: HostProfile) =
        JSONArray().put(profile.hostname).put(profile.port).put(profile.username).put(profile.session).toString()

    internal fun pin(profile: HostProfile): String? = prefs.getString("pin/${profile.endpoint}", null)

    internal fun trust(profile: HostProfile, fingerprint: String): Boolean = synchronized(storageLock) {
        val existing = pin(profile)
        if (existing != null) return existing == fingerprint
        prefs.edit { putString("pin/${profile.endpoint}", fingerprint) }
        true
    }

    internal fun enrolled(profile: HostProfile): Boolean =
        pin(profile)?.let { it == prefs.getString("key/${profile.account}", null) } ?: false

    internal fun recordEnrollment(profile: HostProfile, fingerprint: String): Boolean = synchronized(storageLock) {
        if (pin(profile) != fingerprint) return false
        prefs.edit { putString("key/${profile.account}", fingerprint) }
        true
    }

    internal fun forgetPin(profile: HostProfile, expected: String): Boolean = synchronized(storageLock) {
        if (pin(profile) != expected) return false
        val invalidateWidget = lastOpened()?.endpoint == profile.endpoint
        prefs.edit {
            if (invalidateWidget) { remove("lastOpened"); remove("widgetToken") }
            remove("pin/${profile.endpoint}")
            prefs.all.keys.filter { it.startsWith("key/${profile.endpoint}/") }.forEach(::remove)
        }
        if (invalidateWidget) AgentStatusWidget.update(app)
        true
    }

    private companion object {
        val storageLock = Any()
    }
}
