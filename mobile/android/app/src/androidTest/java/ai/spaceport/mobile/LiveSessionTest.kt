// ABOUTME: Connects the Android client to an explicitly supplied sandbox SSH and Herdr host.
// ABOUTME: Exercises host verification, password login, and retained-connection input fences.
package ai.spaceport.mobile

import androidx.test.platform.app.InstrumentationRegistry
import java.util.concurrent.CountDownLatch
import java.util.concurrent.TimeUnit
import org.junit.Assert.*
import org.junit.Assume.assumeTrue
import org.junit.Test
import uniffi.spaceport_mobile.InputTarget
import uniffi.spaceport_mobile.MobileSession
import uniffi.spaceport_mobile.MobileView

class LiveSessionTest {
    @Test fun verifiesTrustAndResumesTheOriginalSession() {
        val instrumentation = InstrumentationRegistry.getInstrumentation()
        val args = InstrumentationRegistry.getArguments()
        assumeTrue("Requires an explicitly selected, disposable SSH/Herdr host", args.containsKey("herdrHost"))
        val profile = HostProfile(
            args.getString("herdrHost")!!, args.getString("herdrPort", "22").toInt(),
            args.getString("herdrUser")!!, args.getString("herdrSession", "default")
        )
        val expected = requireNotNull(args.getString("herdrFingerprint"))
        val password = requireNotNull(args.getString("herdrPassword"))
        val store = HostStore(instrumentation.targetContext)
        assertNull("Use a fresh endpoint for the sandbox", store.pin(profile))
        val controller = SessionController(instrumentation.targetContext)
        fun await(predicate: () -> Boolean, action: () -> Unit) {
            val event = CountDownLatch(1)
            instrumentation.runOnMainSync {
                controller.changed = { if (predicate() || controller.phase == ConnectionPhase.Failed) event.countDown() }
                action()
            }
            assertTrue("Timed out in ${controller.phase}", event.await(40, TimeUnit.SECONDS))
            instrumentation.runOnMainSync { assertTrue("Phase ${controller.phase}: ${controller.message}", predicate()) }
        }
        try {
            await({ controller.publicKey != null || controller.keyHardwareBacked == false }) {
                controller.loadPublicKey()
            }
            assertNull(controller.profile)
            if (controller.keyHardwareBacked == true) {
                assertTrue(controller.publicKey!!.startsWith("ecdsa-sha2-nistp256 "))
            }
            store.trust(profile, "wrong-test-pin")
            await({ controller.keyChanged }) { controller.connect(profile) }
            assertEquals("wrong-test-pin", store.pin(profile))
            await({ controller.phase == ConnectionPhase.Trust }) { controller.forgetHostKey() }
            assertEquals(expected, controller.fingerprint)
            await({ controller.phase == ConnectionPhase.Password }) { controller.trustHost() }
            if (controller.keyHardwareBacked == false) {
                await({ controller.phase == ConnectionPhase.Failed }) { controller.loginWithKey() }
                assertFalse(controller.keyEnrolled)
                await({ controller.phase == ConnectionPhase.Password }) { controller.usePassword() }
            }
            await({ controller.view?.inputTarget != null }) { controller.login(password) }
            lateinit var snapshot: MobileView
            instrumentation.runOnMainSync { snapshot = requireNotNull(controller.view) }
            await({ controller.view !== snapshot && controller.view?.inputTarget?.let(controller::sessionFor) != null }) {
                controller.cancelGoTo()
                assertNull(controller.sessionFor(snapshot))
                assertNull(controller.sessionFor(requireNotNull(snapshot.inputTarget)))
            }
            lateinit var oldTarget: InputTarget
            lateinit var original: MobileSession
            instrumentation.runOnMainSync {
                oldTarget = requireNotNull(controller.view?.inputTarget)
                original = requireNotNull(controller.sessionFor(oldTarget))
                controller.park()
                assertNull(controller.sessionFor(oldTarget))
            }
            await({ controller.phase == ConnectionPhase.Connected && controller.view?.inputTarget != null }) {
                controller.resume()
            }
            instrumentation.runOnMainSync {
                val target = requireNotNull(controller.view?.inputTarget)
                assertSame(original, controller.sessionFor(target))
                assertNull(controller.sessionFor(oldTarget))
                controller.disconnect()
                assertNull(controller.sessionFor(target))
            }
            // Enrollment changes the host. Only a disposable SSH home may opt in.
            if (args.getString("herdrTestKeyEnrollment") == "true" && controller.keyHardwareBacked == true) {
                await({ controller.phase == ConnectionPhase.Password }) { controller.connect(profile) }
                await({ controller.canEnroll }) { controller.login(password) }
                await({ controller.keyEnrolled }) { controller.enrollKey() }
                instrumentation.runOnMainSync { controller.disconnect() }
                assertTrue(store.forgetPin(profile, expected))
                await({ controller.phase == ConnectionPhase.Trust }) { controller.connect(profile) }
                await({ controller.phase == ConnectionPhase.Password }) { controller.trustHost() }
                assertFalse(controller.keyEnrolled)
                await({ controller.phase == ConnectionPhase.Connected && controller.keyEnrolled }) {
                    controller.loginWithKey()
                }
            }
        } finally {
            instrumentation.runOnMainSync { controller.close() }
            store.forgetPin(profile, expected)
            store.forgetPin(profile, "wrong-test-pin")
        }
    }
}
