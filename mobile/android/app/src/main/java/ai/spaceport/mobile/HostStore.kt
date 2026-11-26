// ABOUTME: Saves SSH routes and host fingerprints in the app's private preferences.
// ABOUTME: Records device-key enrollment only for the matching account and pinned host.
package ai.spaceport.mobile

import android.content.Context
import androidx.core.content.edit
import org.json.JSONArray

class HostStore(context: Context) {
    private val prefs = context.applicationContext.getSharedPreferences("spaceport.hosts", Context.MODE_PRIVATE)

    fun profiles(): List<HostProfile> = prefs.getStringSet("profiles", emptySet()).orEmpty().map { encoded ->
        val parts = JSONArray(encoded)
        HostProfile(parts.getString(0), parts.getInt(1), parts.getString(2), parts.getString(3))
    }.sortedWith(compareBy({ it.hostname }, { it.port }, { it.username }, { it.session }))

    fun save(profile: HostProfile) {
        require(profile.validationError() == null) { profile.validationError().orEmpty() }
        editProfiles(profile, true)
    }

    fun remove(profile: HostProfile) = editProfiles(profile, false)

    /** Replaces saved sessions in one write so Home never lists both the old and new names. */
    fun replace(old: List<HostProfile>, new: List<HostProfile>) = synchronized(storageLock) {
        new.forEach { require(it.validationError() == null) { it.validationError().orEmpty() } }
        val profiles = prefs.getStringSet("profiles", emptySet()).orEmpty()
        prefs.edit { putStringSet("profiles", profiles - old.map(::encode).toSet() + new.map(::encode)) }
    }

    /** An optional label shown in place of the SSH destination; blank clears it. */
    fun name(account: String): String? = prefs.getString("name/$account", null)

    fun rename(account: String, name: String) = prefs.edit {
        if (name.isBlank()) remove("name/$account") else putString("name/$account", name.trim().take(64))
    }

    private fun editProfiles(profile: HostProfile, add: Boolean) = synchronized(storageLock) {
        val row = encode(profile)
        val profiles = prefs.getStringSet("profiles", emptySet()).orEmpty()
        prefs.edit { putStringSet("profiles", if (add) profiles + row else profiles - row) }
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
        prefs.edit {
            remove("pin/${profile.endpoint}")
            prefs.all.keys.filter { it.startsWith("key/${profile.endpoint}/") }.forEach(::remove)
        }
        true
    }

    private companion object {
        val storageLock = Any()
    }
}
