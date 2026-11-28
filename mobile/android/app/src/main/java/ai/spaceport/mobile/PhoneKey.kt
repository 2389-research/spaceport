// ABOUTME: Uses an app-owned Android Keystore key for SSH authentication.
// ABOUTME: Exports only the uncompressed P-256 point and DER-encoded signatures.
package ai.spaceport.mobile

import android.os.Build
import android.security.keystore.KeyGenParameterSpec
import android.security.keystore.KeyInfo
import android.security.keystore.KeyProperties
import androidx.annotation.RequiresApi
import java.security.GeneralSecurityException
import java.security.KeyFactory
import java.security.KeyPairGenerator
import java.security.KeyStore
import java.security.PrivateKey
import java.security.ProviderException
import java.security.Signature
import java.security.interfaces.ECPublicKey
import java.security.spec.ECGenParameterSpec
import uniffi.spaceport_mobile.EnclaveSigner
import uniffi.spaceport_mobile.SshException

internal class HardwareKeyUnavailable(
    message: String = "This device cannot keep the SSH key in secure hardware. Use password sign-in.",
    cause: Throwable? = null,
) : GeneralSecurityException(message, cause)

internal class PhoneKey private constructor(
    val point: ByteArray,
    private val privateKey: PrivateKey,
) : EnclaveSigner {
    override fun sign(message: ByteArray): ByteArray = try {
        Signature.getInstance("SHA256withECDSA").run {
            initSign(privateKey)
            update(message)
            sign()
        }
    } catch (e: GeneralSecurityException) {
        throw SshException.Key()
    }

    companion object {
        @RequiresApi(28)
        fun parameters(alias: String): KeyGenParameterSpec =
            KeyGenParameterSpec.Builder(alias, KeyProperties.PURPOSE_SIGN)
                .setAlgorithmParameterSpec(ECGenParameterSpec("secp256r1"))
                .setDigests(KeyProperties.DIGEST_SHA256)
                .setUnlockedDeviceRequired(true)
                .build()

        @Synchronized
        fun open(alias: String = "spaceport.ssh.unlocked.p256", createIfMissing: Boolean = true): PhoneKey {
            if (Build.VERSION.SDK_INT < 28) {
                throw HardwareKeyUnavailable("Phone key requires Android 9 or later. Use password sign-in.")
            }
            val store = KeyStore.getInstance("AndroidKeyStore").apply { load(null) }
            if (!store.containsAlias(alias)) {
                if (!createIfMissing) throw HardwareKeyUnavailable("The enrolled phone key is unavailable.")
                try {
                    val generator = KeyPairGenerator.getInstance(KeyProperties.KEY_ALGORITHM_EC, "AndroidKeyStore")
                    generator.initialize(parameters(alias))
                    generator.generateKeyPair()
                } catch (e: ProviderException) {
                    throw HardwareKeyUnavailable("Keystore could not create the phone key. Check your screen lock or use password sign-in.", e)
                } catch (e: GeneralSecurityException) {
                    throw HardwareKeyUnavailable("Keystore could not create the phone key. Use password sign-in.", e)
                }
            }
            val entry = store.getEntry(alias, null) as KeyStore.PrivateKeyEntry
            val public = entry.certificate.publicKey as ECPublicKey
            val point = ByteArray(65)
            point[0] = 4
            for ((index, coordinate) in listOf(public.w.affineX, public.w.affineY).withIndex()) {
                require(coordinate.signum() >= 0 && coordinate.bitLength() <= 256)
                val bytes = coordinate.toByteArray()
                val length = minOf(32, bytes.size)
                bytes.copyInto(point, 1 + (index + 1) * 32 - length, bytes.size - length)
            }
            val info = KeyFactory.getInstance(KeyProperties.KEY_ALGORITHM_EC, "AndroidKeyStore")
                .getKeySpec(entry.privateKey, KeyInfo::class.java)
            @Suppress("DEPRECATION")
            val hardware = if (Build.VERSION.SDK_INT >= 31) {
                info.securityLevel == KeyProperties.SECURITY_LEVEL_TRUSTED_ENVIRONMENT ||
                    info.securityLevel == KeyProperties.SECURITY_LEVEL_STRONGBOX
            } else {
                info.isInsideSecureHardware
            }
            if (!hardware) throw HardwareKeyUnavailable()
            return PhoneKey(point, entry.privateKey)
        }
    }
}
