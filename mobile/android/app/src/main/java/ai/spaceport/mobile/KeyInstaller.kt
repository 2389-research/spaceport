// ABOUTME: Installs this phone's public key on an SSH account without opening Herdr.
// ABOUTME: Pins an unknown host only after explicit trust and records a verified key-only login.
package ai.spaceport.mobile

import uniffi.spaceport_mobile.SshException
import uniffi.spaceport_mobile.SshHost
import uniffi.spaceport_mobile.authorizeEnclavePhone

internal sealed interface InstallOutcome {
    data object Installed : InstallOutcome

    /** The host needs a fingerprint decision; [changed] keys must be forgotten in Host Detail first. */
    data class Challenge(val fingerprint: String, val changed: Boolean) : InstallOutcome

    data class Failed(val message: String) : InstallOutcome
}

internal class KeyInstaller(private val store: HostStore) {
    /** Blocks on SSH; call from a worker thread. [trusting] is a fingerprint the person just accepted. */
    fun install(profile: HostProfile, password: String, trusting: String? = null): InstallOutcome {
        if (trusting != null && !store.trust(profile, trusting))
            return InstallOutcome.Failed("A different fingerprint is already saved for this host.")
        val pin = store.pin(profile)
        return try {
            val key = PhoneKey.open()
            authorizeEnclavePhone(SshHost(profile.hostname, profile.port.toUShort(), profile.username, pin), password, key.point, key)
            if (pin != null && store.recordEnrollment(profile, pin)) InstallOutcome.Installed
            else InstallOutcome.Failed("The host accepted the key, but its saved fingerprint changed. Verify the host again.")
        } catch (e: SshException.UnknownHost) {
            InstallOutcome.Challenge(e.fingerprint, false)
        } catch (e: SshException.ChangedHost) {
            InstallOutcome.Challenge(e.fingerprint, true)
        } catch (e: HardwareKeyUnavailable) {
            InstallOutcome.Failed(e.message.orEmpty())
        } catch (e: SshException) {
            InstallOutcome.Failed(
                when (e) {
                    is SshException.Authentication -> "SSH refused password sign-in for this account."
                    is SshException.UnverifiedEnrollment -> "The key may have been installed. A fresh key-only login failed."
                    is SshException.Enrollment, is SshException.Channel ->
                        "SSH signed in, but the host refused to install the public key."
                    is SshException.Timeout -> "SSH timed out. Check the host address and network."
                    is SshException.Connection -> "Spaceport could not reach the SSH host."
                    else -> "Could not install the key over a verified SSH connection."
                }
            )
        }
    }
}
