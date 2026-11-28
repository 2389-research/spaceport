// ABOUTME: Exercises storage and signing on a real Android Keystore provider.
// ABOUTME: Verifies the exported public point and persistent endpoint trust.
package ai.spaceport.mobile

import android.os.Build
import androidx.test.platform.app.InstrumentationRegistry
import java.security.KeyStore
import java.security.Signature
import java.util.UUID
import org.junit.Assert.*
import org.junit.Test
import uniffi.spaceport_mobile.enclavePublicKey

class DeviceIdentityTest {
    @Test fun signsWithTheSameNonExportableP256KeyAfterReopening() {
        val alias = "spaceport-test-${UUID.randomUUID()}"
        val store = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
        try {
            assertThrows(HardwareKeyUnavailable::class.java) { PhoneKey.open(alias, createIfMissing = false) }
            assertFalse(store.containsAlias(alias))
            if (Build.VERSION.SDK_INT < 28) {
                assertThrows(HardwareKeyUnavailable::class.java) { PhoneKey.open(alias) }
                assertFalse(store.containsAlias(alias))
                return
            }
            assertTrue(PhoneKey.parameters(alias).isUnlockedDeviceRequired)
            val first = try {
                PhoneKey.open(alias)
            } catch (e: HardwareKeyUnavailable) {
                // An emulator can lack secure hardware. No software signing route may escape.
                assertNull(store.getKey(alias, null)?.encoded)
                assertThrows(HardwareKeyUnavailable::class.java) { PhoneKey.open(alias) }
                return
            }
            val second = PhoneKey.open(alias)
            assertArrayEquals(first.point, second.point)
            assertTrue(enclavePublicKey(second.point).startsWith("ecdsa-sha2-nistp256 "))
            assertNull(store.getKey(alias, null).encoded)
            val message = "SSH authentication".toByteArray()
            val verifier = Signature.getInstance("SHA256withECDSA")
            verifier.initVerify(store.getCertificate(alias).publicKey)
            verifier.update(message)
            assertTrue(verifier.verify(second.sign(message)))
        } finally {
            store.deleteEntry(alias)
        }
    }

    @Test fun doesNotReplacePinsAndScopesEnrollmentToVerifiedHostKey() {
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        val store = HostStore(context)
        val p = HostProfile("${UUID.randomUUID()}.invalid", 2222, "demo", "default")
        val other = p.copy(session = "second")
        try {
            store.save(p)
            store.save(other)
            assertTrue(store.trust(p, "first"))
            assertFalse(store.trust(other, "unexpected"))
            assertTrue(store.recordEnrollment(p, "first"))
            val reopened = HostStore(context)
            assertEquals("first", reopened.pin(other))
            assertTrue(reopened.enrolled(other))
            assertFalse(reopened.enrolled(other.copy(username = "other")))
            reopened.remove(p)
            assertTrue(reopened.profiles().contains(other))
            assertEquals("first", reopened.pin(other))
            assertTrue(reopened.forgetPin(other, "first"))
            assertFalse(reopened.enrolled(other))
            assertFalse(reopened.recordEnrollment(p, "first"))
        } finally {
            store.remove(p)
            store.remove(other)
            store.forgetPin(p, "first")
        }
    }
}
