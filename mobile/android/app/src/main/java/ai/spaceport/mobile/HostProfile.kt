// ABOUTME: Identifies one SSH account and named Herdr session.
// ABOUTME: Keeps endpoint trust independent of the account and session.
package ai.spaceport.mobile

import java.util.Locale

data class HostProfile(val hostname: String, val port: Int, val username: String, val session: String) {
    internal val endpoint get() = "[${hostname.lowercase(Locale.ROOT)}]:$port"
    internal val account get() = "$endpoint/$username"

    fun validationError(): String? = when {
        hostname.isEmpty() || hostname.toByteArray().size > 255 ||
            hostname.any { it.isWhitespace() || it.isISOControl() || it in "/[]" } ->
            "Enter an SSH hostname or IP address."
        port !in 1..65535 -> "Enter an SSH port between 1 and 65535."
        username.isEmpty() || username.toByteArray().size > 255 ||
            username.any { it.isWhitespace() || it.isISOControl() } -> "Enter an SSH username."
        !SESSION.matches(session) || session == "." || session == ".." ->
            "Enter a Herdr session name of up to 64 letters, digits, periods, underscores, or hyphens."
        else -> null
    }

    private companion object {
        val SESSION = Regex("[a-zA-Z0-9._-]{1,64}")
    }
}
