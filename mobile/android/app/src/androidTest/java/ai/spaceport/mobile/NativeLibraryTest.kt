// ABOUTME: Exercises the packaged Rust library through the generated Kotlin bindings.
// ABOUTME: Verifies an optional value and its returned enum survive a native call on Android.
package ai.spaceport.mobile

import androidx.test.ext.junit.runners.AndroidJUnit4
import org.junit.Assert.assertEquals
import org.junit.Test
import org.junit.runner.RunWith
import uniffi.spaceport_mobile.IdleAgeGroup
import uniffi.spaceport_mobile.idleAgeGroup

@RunWith(AndroidJUnit4::class)
class NativeLibraryTest {
    @Test
    fun classifiesIdleAgeInTheSharedController() {
        assertEquals(IdleAgeGroup.UNPLACED, idleAgeGroup(null, false, 3_600_000uL))
        assertEquals(IdleAgeGroup.ONE_TO_FIVE_HOURS, idleAgeGroup(0uL, true, 3_600_000uL))
    }
}
