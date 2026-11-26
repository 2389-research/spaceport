// ABOUTME: Names the screens reached from Home and what those screens may ask of the activity.
// ABOUTME: Keeps Settings, key installation and host details out of the session activity.
package ai.spaceport.mobile

/** A screen above Home. The activity keeps a back stack of these across rotation. */
sealed interface Page {
    data object Settings : Page
    data object ToolbarKeys : Page
    data class InstallKey(val account: String) : Page
    data class HostDetail(val account: String) : Page
}

interface Pages {
    val hosts: HostStore

    fun go(page: Page)

    fun back()

    /** Redraws the current page, for example after saving a change. */
    fun redraw()

    fun open(profile: HostProfile)

    /** Closes live connections for these sessions and drops their saved observations. */
    fun forget(profiles: Collection<HostProfile>)

    /** Closes live connections that trusted this endpoint's pinned key. */
    fun closeEndpoint(endpoint: String)

    /** Runs [work] off the UI thread; [done] runs only if the same page is still showing. */
    fun <T> background(work: () -> T, done: (T) -> Unit)

    fun toast(text: String)
}

internal fun Pages.group(account: String): HostGroup? = HostGroup.of(hosts.profiles()).firstOrNull { it.account == account }

/** Saves [new] in place of [old]; false after telling the person why it was refused. */
internal fun Pages.addSession(old: List<HostProfile>, new: HostProfile): Boolean {
    new.validationError()?.let { toast(it); return false }
    if (new in hosts.profiles()) {
        toast("This host already has a session named ${new.session}.")
        return false
    }
    hosts.replace(old, listOf(new))
    forget(old)
    return true
}
