// ABOUTME: Checks the packaged agent resources and the unknown-identity fallback.
// ABOUTME: Exercises the native parser used by the Home board and the live picker.
package ai.spaceport.mobile

import androidx.test.platform.app.InstrumentationRegistry
import org.junit.Assert.*
import org.junit.Test

class AgentBrandTest {
    @Test fun packagedMarksUseOnlyTheAgentNamespace() {
        val context = InstrumentationRegistry.getInstrumentation().targetContext
        assertNotNull(agentBrand(context, "claude"))
        assertNotNull(agentBrand(context, "codex"))
        assertNotNull(agentBrand(context, "omo"))
        assertNull(agentBrand(context, "../launcher"))
        assertNull(agentBrand(context, "future-agent"))
    }
}
